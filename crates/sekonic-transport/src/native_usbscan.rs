#![allow(unsafe_code)]

use crate::traits::ScannerTransport;
use crate::usb_ids::VID_SEKONIC;
use async_trait::async_trait;
use sekonic_schema::error::SekonicError;
use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr::{null, null_mut};
use std::sync::Arc;
use std::time::Duration;
use windows_sys::core::GUID;
use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW,
    SetupDiGetDeviceInterfaceDetailW, DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, HDEVINFO,
    SP_DEVICE_INTERFACE_DATA, SP_DEVICE_INTERFACE_DETAIL_DATA_W,
};
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_INSUFFICIENT_BUFFER, ERROR_IO_PENDING,
    ERROR_NO_MORE_ITEMS, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
    WAIT_TIMEOUT,
};
use windows_sys::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, FILE_FLAG_OVERLAPPED, FILE_SHARE_READ, FILE_SHARE_WRITE,
    OPEN_EXISTING,
};
use windows_sys::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows_sys::Win32::System::IO::{CancelIoEx, DeviceIoControl, GetOverlappedResult, OVERLAPPED};

/// `GUID_DEVINTERFACE_IMAGE` (`{6bdd1fc6-810f-11d0-bec7-08002be2092f}`) used by `usbscan.sys`.
const GUID_DEVINTERFACE_IMAGE: GUID = GUID {
    data1: 0x6bdd_1fc6,
    data2: 0x810f,
    data3: 0x11d0,
    data4: [0xbe, 0xc7, 0x08, 0x00, 0x2b, 0xe2, 0x09, 0x2f],
};

/// `GUID_CLASS_OMR` (`{a43d14b6-add8-11d3-99f0-005004d38a59}`) used by Sekonic OMR INF files.
const GUID_CLASS_OMR: GUID = GUID {
    data1: 0xa43d_14b6,
    data2: 0xadd8,
    data3: 0x11d3,
    data4: [0x99, 0xf0, 0x00, 0x50, 0x04, 0xd3, 0x8a, 0x59],
};

/// `CTL_CODE(FILE_DEVICE_USB_SCAN (0x8000), 0x806, METHOD_BUFFERED, FILE_ANY_ACCESS)`
const IOCTL_GET_DEVICE_DESCRIPTOR: u32 = 0x8000_2018;

#[repr(C)]
#[derive(Default)]
struct UsbscanDeviceDescriptor {
    vendor_id: u16,
    product_id: u16,
    bcd_device: u16,
    language_id: u16,
}

struct DevInfoGuard(HDEVINFO);

impl Drop for DevInfoGuard {
    fn drop(&mut self) {
        if self.0 != INVALID_HANDLE_VALUE && self.0 != 0 {
            unsafe {
                SetupDiDestroyDeviceInfoList(self.0);
            }
        }
    }
}

struct HandleInner(HANDLE);

// SAFETY: A Win32 file handle opened with FILE_FLAG_OVERLAPPED supports concurrent overlapped I/O
// across threads, and each operation uses its own stack-local OVERLAPPED and event handle.
unsafe impl Send for HandleInner {}
unsafe impl Sync for HandleInner {}

impl Drop for HandleInner {
    fn drop(&mut self) {
        if self.0 != INVALID_HANDLE_VALUE && self.0 != 0 {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

struct EventGuard(HANDLE);

impl Drop for EventGuard {
    fn drop(&mut self) {
        if self.0 != 0 && self.0 != INVALID_HANDLE_VALUE {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn enumerate_interface_paths(guid: &GUID) -> Vec<String> {
    let mut paths = Vec::new();
    let dev_info = unsafe {
        SetupDiGetClassDevsW(
            guid,
            null(),
            0,
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        )
    };
    if dev_info == INVALID_HANDLE_VALUE || dev_info == 0 {
        return paths;
    }
    let _guard = DevInfoGuard(dev_info);

    let mut index: u32 = 0;
    loop {
        let mut iface_data: SP_DEVICE_INTERFACE_DATA = unsafe { zeroed() };
        iface_data.cbSize = size_of::<SP_DEVICE_INTERFACE_DATA>() as u32;

        let ok = unsafe {
            SetupDiEnumDeviceInterfaces(dev_info, null(), guid, index, &mut iface_data)
        };
        if ok == 0 {
            let err = unsafe { GetLastError() };
            if err == ERROR_NO_MORE_ITEMS {
                break;
            }
            break;
        }
        index += 1;

        let mut required_size: u32 = 0;
        let _ = unsafe {
            SetupDiGetDeviceInterfaceDetailW(
                dev_info,
                &iface_data,
                null_mut(),
                0,
                &mut required_size,
                null_mut(),
            )
        };
        let err = unsafe { GetLastError() };
        if err != ERROR_INSUFFICIENT_BUFFER || required_size == 0 {
            continue;
        }

        let mut buf = vec![0u8; required_size as usize];
        let detail = buf.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
        unsafe {
            (*detail).cbSize = size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;
        }

        let ok = unsafe {
            SetupDiGetDeviceInterfaceDetailW(
                dev_info,
                &iface_data,
                detail,
                required_size,
                null_mut(),
                null_mut(),
            )
        };
        if ok == 0 {
            continue;
        }

        let max_u16 = (required_size as usize).saturating_sub(4) / 2;
        let path_base = unsafe { buf.as_ptr().add(4) as *const u16 };
        let mut utf16 = Vec::with_capacity(max_u16);
        for i in 0..max_u16 {
            let ch = unsafe { std::ptr::read_unaligned(path_base.add(i)) };
            if ch == 0 {
                break;
            }
            utf16.push(ch);
        }
        if let Ok(path) = String::from_utf16(&utf16) {
            paths.push(path);
        }
    }

    paths
}

fn path_matches_vid_pid(path: &str, vid: u16, pid: u16) -> bool {
    let lower = path.to_ascii_lowercase();
    let vid_pat = format!("vid_{vid:04x}");
    let pid_pat = format!("pid_{pid:04x}");
    lower.contains(&vid_pat) && lower.contains(&pid_pat)
}

fn open_overlapped_handle(path: &str) -> Result<HandleInner, SekonicError> {
    let wide = to_wide_null(path);
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null(),
            OPEN_EXISTING,
            FILE_FLAG_OVERLAPPED,
            0,
        )
    };
    if handle == INVALID_HANDLE_VALUE || handle == 0 {
        let err = std::io::Error::last_os_error();
        return Err(SekonicError::Transport(format!(
            "usbscan open '{path}' failed: {err}"
        )));
    }
    Ok(HandleInner(handle))
}

fn query_usbscan_vid_pid(handle: HANDLE) -> Option<(u16, u16)> {
    let mut desc = UsbscanDeviceDescriptor::default();
    let mut bytes_returned: u32 = 0;
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_GET_DEVICE_DESCRIPTOR,
            null(),
            0,
            &mut desc as *mut UsbscanDeviceDescriptor as *mut c_void,
            size_of::<UsbscanDeviceDescriptor>() as u32,
            &mut bytes_returned,
            null_mut(),
        )
    };
    if ok != 0 && bytes_returned >= 4 {
        Some((desc.vendor_id, desc.product_id))
    } else {
        None
    }
}

/// Transport for Sekonic USB scanners bound to Windows `usbscan.sys` (`Optical Mark Reader` / Imaging devices).
pub struct WindowsUsbscanTransport {
    handle: Arc<HandleInner>,
}

impl WindowsUsbscanTransport {
    pub fn open_by_pid(pid: u16) -> Result<Self, SekonicError> {
        // 1. Try GUID_DEVINTERFACE_IMAGE and GUID_CLASS_OMR via SetupDi (path contains vid_0a41&pid_xxxx).
        for guid in [&GUID_DEVINTERFACE_IMAGE, &GUID_CLASS_OMR] {
            for path in enumerate_interface_paths(guid) {
                if path_matches_vid_pid(&path, VID_SEKONIC, pid) {
                    if let Ok(handle) = open_overlapped_handle(&path) {
                        return Ok(Self {
                            handle: Arc::new(handle),
                        });
                    }
                }
            }
        }

        // 2. Fallback: probe \\.\Usbscan0..15 and verify VID/PID via IOCTL_GET_DEVICE_DESCRIPTOR.
        for idx in 0..16u8 {
            let path = format!(r"\\.\Usbscan{idx}");
            if let Ok(handle) = open_overlapped_handle(&path) {
                if let Some((vid, dev_pid)) = query_usbscan_vid_pid(handle.0) {
                    if vid == VID_SEKONIC && dev_pid == pid {
                        return Ok(Self {
                            handle: Arc::new(handle),
                        });
                    }
                }
            }
        }

        Err(SekonicError::Transport(format!(
            "usbscan device VID_{VID_SEKONIC:04X}&PID_{pid:04X} could not be opened"
        )))
    }
}

fn wait_overlapped(
    handle: HANDLE,
    ov: &mut OVERLAPPED,
    event: HANDLE,
    timeout_ms: u64,
    op_name: &str,
) -> Result<usize, SekonicError> {
    let wait_ms = u32::try_from(timeout_ms).unwrap_or(u32::MAX - 1);
    let wait_res = unsafe { WaitForSingleObject(event, wait_ms) };
    match wait_res {
        WAIT_OBJECT_0 => {
            let mut transferred: u32 = 0;
            let ok = unsafe { GetOverlappedResult(handle, ov, &mut transferred, 0) };
            if ok == 0 {
                let err = std::io::Error::last_os_error();
                return Err(SekonicError::Transport(format!(
                    "usbscan {op_name} overlapped result failed: {err}"
                )));
            }
            Ok(transferred as usize)
        }
        WAIT_TIMEOUT => {
            unsafe {
                CancelIoEx(handle, ov);
                let mut transferred: u32 = 0;
                let _ = GetOverlappedResult(handle, ov, &mut transferred, 1);
            }
            Err(SekonicError::Timeout { timeout_ms })
        }
        _ => {
            unsafe {
                CancelIoEx(handle, ov);
                let mut transferred: u32 = 0;
                let _ = GetOverlappedResult(handle, ov, &mut transferred, 1);
            }
            let err = std::io::Error::last_os_error();
            Err(SekonicError::Transport(format!(
                "usbscan {op_name} wait failed: {err}"
            )))
        }
    }
}

#[async_trait]
impl ScannerTransport for WindowsUsbscanTransport {
    async fn write_control(&mut self, buf: &[u8], timeout: Duration) -> Result<usize, SekonicError> {
        let data = buf.to_vec();
        let handle = Arc::clone(&self.handle);
        let timeout_ms = timeout.as_millis() as u64;

        tokio::task::spawn_blocking(move || -> Result<usize, SekonicError> {
            let event = unsafe { CreateEventW(null(), 1, 0, null()) };
            if event == 0 {
                return Err(SekonicError::Transport(format!(
                    "usbscan CreateEventW failed: {}",
                    std::io::Error::last_os_error()
                )));
            }
            let _event_guard = EventGuard(event);

            let mut ov: OVERLAPPED = unsafe { zeroed() };
            ov.hEvent = event;
            let mut written: u32 = 0;
            let len = u32::try_from(data.len()).unwrap_or(u32::MAX);

            let ok = unsafe {
                WriteFile(
                    handle.0,
                    data.as_ptr(),
                    len,
                    &mut written,
                    &mut ov,
                )
            };
            if ok != 0 {
                return Ok(written as usize);
            }
            let err = unsafe { GetLastError() };
            if err == ERROR_IO_PENDING {
                wait_overlapped(handle.0, &mut ov, event, timeout_ms, "write")
            } else {
                Err(SekonicError::Transport(format!(
                    "usbscan WriteFile failed: {}",
                    std::io::Error::from_raw_os_error(err as i32)
                )))
            }
        })
        .await
        .map_err(|e| SekonicError::Transport(format!("usbscan write task join error: {e}")))?
    }

    async fn read_control(
        &mut self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, SekonicError> {
        let max_len = buf.len();
        let handle = Arc::clone(&self.handle);
        let timeout_ms = timeout.as_millis() as u64;

        let data = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, SekonicError> {
            let event = unsafe { CreateEventW(null(), 1, 0, null()) };
            if event == 0 {
                return Err(SekonicError::Transport(format!(
                    "usbscan CreateEventW failed: {}",
                    std::io::Error::last_os_error()
                )));
            }
            let _event_guard = EventGuard(event);

            let mut tmp = vec![0u8; max_len];
            let mut ov: OVERLAPPED = unsafe { zeroed() };
            ov.hEvent = event;
            let mut read_bytes: u32 = 0;
            let len = u32::try_from(tmp.len()).unwrap_or(u32::MAX);

            let ok = unsafe {
                ReadFile(
                    handle.0,
                    tmp.as_mut_ptr(),
                    len,
                    &mut read_bytes,
                    &mut ov,
                )
            };
            if ok != 0 {
                tmp.truncate(read_bytes as usize);
                return Ok(tmp);
            }
            let err = unsafe { GetLastError() };
            if err == ERROR_IO_PENDING {
                let n = wait_overlapped(handle.0, &mut ov, event, timeout_ms, "read")?;
                tmp.truncate(n);
                Ok(tmp)
            } else {
                Err(SekonicError::Transport(format!(
                    "usbscan ReadFile failed: {}",
                    std::io::Error::from_raw_os_error(err as i32)
                )))
            }
        })
        .await
        .map_err(|e| SekonicError::Transport(format!("usbscan read task join error: {e}")))??;

        let n = data.len().min(buf.len());
        buf[..n].copy_from_slice(&data[..n]);
        Ok(n)
    }

    async fn read_image_pipe(
        &mut self,
        _buf: &mut [u8],
        _timeout: Duration,
    ) -> Result<usize, SekonicError> {
        Err(SekonicError::Transport(
            "usbscan device has no secondary image pipe".into(),
        ))
    }

    async fn purge(&mut self) -> Result<(), SekonicError> {
        unsafe {
            CancelIoEx(self.handle.0, null());
        }
        Ok(())
    }

    fn uses_serial_framing(&self) -> bool {
        false
    }
}
