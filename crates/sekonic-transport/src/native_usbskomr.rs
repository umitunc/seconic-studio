use crate::traits::ScannerTransport;
use async_trait::async_trait;
use sekonic_schema::error::SekonicError;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Probes Windows legacy Sekonic WDM driver device nodes (`\\.\UsbSkOmr0` .. `\\.\UsbSkOmr7`)
/// created by `SkSr6500.sys` / `UsbSkOmr.sys` for SR-2300, SR-3500, SR-5500, SR-6000, SR-6500.
pub fn discover_usbskomr_devices() -> Vec<u8> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::OpenOptionsExt;
        let mut found = Vec::new();
        for idx in 0..8u8 {
            let path = format!(r"\\.\UsbSkOmr{idx}");
            match OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(3) // FILE_SHARE_READ | FILE_SHARE_WRITE (matches OMRAPI.DLL @ 0x100053e6)
                .open(&path)
            {
                Ok(_) => found.push(idx),
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => found.push(idx),
                Err(_) => {}
            }
        }
        found
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

/// Transport for Sekonic's legacy Windows WDM driver (`\\.\UsbSkOmr%d`).
pub struct LegacyUsbSkOmrTransport {
    file: Arc<Mutex<File>>,
}

impl LegacyUsbSkOmrTransport {
    pub fn open(index: u8) -> Result<Self, SekonicError> {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::fs::OpenOptionsExt;
            let path = format!(r"\\.\UsbSkOmr{index}");
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(3)
                .open(&path)
                .map_err(|e| SekonicError::Transport(format!("open {path} failed: {e}")))?;
            Ok(Self {
                file: Arc::new(Mutex::new(file)),
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = index;
            Err(SekonicError::Transport(
                "UsbSkOmr legacy WDM driver is only supported on Windows".into(),
            ))
        }
    }
}

#[async_trait]
impl ScannerTransport for LegacyUsbSkOmrTransport {
    async fn write_control(&mut self, buf: &[u8], timeout: Duration) -> Result<usize, SekonicError> {
        let data = buf.to_vec();
        let file = Arc::clone(&self.file);
        let fut = tokio::task::spawn_blocking(move || -> Result<usize, SekonicError> {
            let mut guard = file
                .lock()
                .map_err(|_| SekonicError::Transport("UsbSkOmr mutex poisoned".into()))?;
            guard
                .write(&data)
                .map_err(|e| SekonicError::Transport(format!("UsbSkOmr write error: {e}")))
        });
        tokio::time::timeout(timeout, fut)
            .await
            .map_err(|_| SekonicError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            })?
            .map_err(|e| SekonicError::Transport(format!("UsbSkOmr task join error: {e}")))?
    }

    async fn read_control(
        &mut self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, SekonicError> {
        let max_len = buf.len();
        let file = Arc::clone(&self.file);
        let fut = tokio::task::spawn_blocking(move || -> Result<Vec<u8>, SekonicError> {
            let mut tmp = vec![0u8; max_len];
            let mut guard = file
                .lock()
                .map_err(|_| SekonicError::Transport("UsbSkOmr mutex poisoned".into()))?;
            let n = guard
                .read(&mut tmp)
                .map_err(|e| SekonicError::Transport(format!("UsbSkOmr read error: {e}")))?;
            tmp.truncate(n);
            Ok(tmp)
        });
        let data = tokio::time::timeout(timeout, fut)
            .await
            .map_err(|_| SekonicError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            })?
            .map_err(|e| SekonicError::Transport(format!("UsbSkOmr task join error: {e}")))??;
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
            "UsbSkOmr does not support a secondary CIS image pipe".into(),
        ))
    }

    async fn purge(&mut self) -> Result<(), SekonicError> {
        Ok(())
    }

    fn uses_serial_framing(&self) -> bool {
        // Verified from 90K OMRAPI.DLL::_OMR_SendCommand @ 0x10003ff6:
        // Mode 1 (`UsbSkOmr`) sends plain ASCII commands without STX/LEN/CRC16.
        false
    }
}

