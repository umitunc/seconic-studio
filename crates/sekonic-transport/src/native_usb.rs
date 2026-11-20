use crate::traits::{DiscoveredScanner, ScannerTransport, TransportKind};
use crate::usb_ids::{classify_pid, VID_SEKONIC};
use async_trait::async_trait;
use nusb::transfer::RequestBuffer;
use sekonic_schema::error::SekonicError;
use std::time::Duration;

/// Scans connected USB buses for Sekonic scanners (`VID == 0x0A41`).
pub fn discover_usb_scanners() -> Result<Vec<DiscoveredScanner>, SekonicError> {
    let devices = nusb::list_devices().map_err(|e| SekonicError::Transport(e.to_string()))?;
    let mut found = Vec::new();
    for dev in devices {
        if dev.vendor_id() == VID_SEKONIC {
            let (family, model) = classify_pid(dev.product_id())
                .unwrap_or((crate::traits::DeviceFamily::FamilyAOmrGrid, "SEKONIC OMR"));
            found.push(DiscoveredScanner {
                id: format!("usb:{:04x}:{:04x}:{}", dev.vendor_id(), dev.product_id(), dev.device_address()),
                vendor_id: dev.vendor_id(),
                product_id: dev.product_id(),
                model_hint: model.to_string(),
                family,
                transport_kind: TransportKind::UsbBulk,
            });
        }
    }
    Ok(found)
}

pub struct NativeUsbTransport {
    interface: nusb::Interface,
    ep_out: u8,
    ep_in_ctrl: u8,
    ep_in_image: Option<u8>,
}

impl NativeUsbTransport {
    pub fn open_by_pid(pid: u16) -> Result<Self, SekonicError> {
        let dev_info = nusb::list_devices()
            .map_err(|e| SekonicError::Transport(e.to_string()))?
            .find(|d| d.vendor_id() == VID_SEKONIC && d.product_id() == pid)
            .ok_or(SekonicError::NotConnected)?;

        let device = dev_info
            .open()
            .map_err(|e| SekonicError::Transport(format!("usb open failed: {e}")))?;

        let cfg = device
            .active_configuration()
            .map_err(|e| SekonicError::Transport(format!("usb config error: {e}")))?;

        let mut ep_out = None;
        let mut ep_in_ctrl = None;
        let mut ep_in_image = None;
        let mut iface_num = 0u8;

        if let Some(iface) = cfg.interfaces().next() {
            iface_num = iface.interface_number();
            if let Some(alt) = iface.alt_settings().next() {
                for ep in alt.endpoints() {
                    if ep.transfer_type() == nusb::transfer::EndpointType::Bulk {
                        match ep.direction() {
                            nusb::transfer::Direction::Out if ep_out.is_none() => {
                                ep_out = Some(ep.address());
                            }
                            nusb::transfer::Direction::In if ep_in_ctrl.is_none() => {
                                ep_in_ctrl = Some(ep.address());
                            }
                            nusb::transfer::Direction::In if ep_in_image.is_none() => {
                                ep_in_image = Some(ep.address());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        let interface = device
            .claim_interface(iface_num)
            .map_err(|e| SekonicError::Transport(format!("claim interface failed: {e}")))?;

        Ok(Self {
            interface,
            ep_out: ep_out.unwrap_or(0x01),
            ep_in_ctrl: ep_in_ctrl.unwrap_or(0x81),
            ep_in_image,
        })
    }
}

#[async_trait]
impl ScannerTransport for NativeUsbTransport {
    async fn write_control(&mut self, buf: &[u8], timeout: Duration) -> Result<usize, SekonicError> {
        let fut = self.interface.bulk_out(self.ep_out, buf.to_vec());
        let comp = tokio::time::timeout(timeout, fut)
            .await
            .map_err(|_| SekonicError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            })?;
        comp.status
            .map_err(|e| SekonicError::Transport(format!("bulk out error: {e}")))?;
        Ok(comp.data.actual_length())
    }

    async fn read_control(
        &mut self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, SekonicError> {
        let fut = self
            .interface
            .bulk_in(self.ep_in_ctrl, RequestBuffer::new(buf.len()));
        let comp = tokio::time::timeout(timeout, fut)
            .await
            .map_err(|_| SekonicError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            })?;
        comp.status
            .map_err(|e| SekonicError::Transport(format!("bulk in error: {e}")))?;
        let n = comp.data.len().min(buf.len());
        buf[..n].copy_from_slice(&comp.data[..n]);
        Ok(n)
    }

    async fn read_image_pipe(
        &mut self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, SekonicError> {
        let ep = self
            .ep_in_image
            .ok_or_else(|| SekonicError::Transport("device has no secondary image pipe".into()))?;
        let fut = self.interface.bulk_in(ep, RequestBuffer::new(buf.len()));
        let comp = tokio::time::timeout(timeout, fut)
            .await
            .map_err(|_| SekonicError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            })?;
        comp.status
            .map_err(|e| SekonicError::Transport(format!("image pipe error: {e}")))?;
        let n = comp.data.len().min(buf.len());
        buf[..n].copy_from_slice(&comp.data[..n]);
        Ok(n)
    }

    async fn purge(&mut self) -> Result<(), SekonicError> {
        Ok(())
    }

    fn uses_serial_framing(&self) -> bool {
        false
    }
}
