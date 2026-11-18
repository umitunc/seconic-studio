#![forbid(unsafe_code)]

//! Unified USB Bulk / Serial / In-Memory transport layer for Sekonic scanners.

pub mod memory_channel;
pub mod traits;
pub mod usb_ids;

#[cfg(not(target_arch = "wasm32"))]
pub mod native_serial;
#[cfg(not(target_arch = "wasm32"))]
pub mod native_usb;

#[cfg(target_arch = "wasm32")]
pub mod wasm_serial;
#[cfg(target_arch = "wasm32")]
pub mod wasm_usb;

#[cfg(test)]
mod tests {
    use super::memory_channel::InMemoryTransport;
    use super::traits::{DeviceFamily, ScannerTransport};
    use super::usb_ids::{classify_pid, PID_SR55D, PID_SR3500_6000_6500};
    use std::time::Duration;

    #[test]
    fn test_pid_classification() {
        assert_eq!(
            classify_pid(PID_SR3500_6000_6500).map(|x| x.0),
            Some(DeviceFamily::FamilyAOmrGrid)
        );
        assert_eq!(
            classify_pid(PID_SR55D).map(|x| x.0),
            Some(DeviceFamily::FamilyBCisImage)
        );
    }

    #[tokio::test]
    async fn test_in_memory_transport_duplex() {
        let (mut host, dev) = InMemoryTransport::pair(8);
        host.write_control(b"MN", Duration::from_millis(100))
            .await
            .expect("write");
        let req = dev.recv_from_host().await.expect("recv");
        assert_eq!(req, b"MN");

        dev.send_to_host(b"SR-6500".to_vec()).await.expect("reply");
        let mut buf = [0u8; 32];
        let n = host
            .read_control(&mut buf, Duration::from_millis(100))
            .await
            .expect("read");
        assert_eq!(&buf[..n], b"SR-6500");
    }
}
