use async_trait::async_trait;
use sekonic_schema::error::SekonicError;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceFamily {
    /// SR-1800, SR-1800EX, SR-2300, SR-3500, SR-5500, SR-6000, SR-6500
    FamilyAOmrGrid,
    /// SR-55D and CIS / Image-based Hybrid models
    FamilyBCisImage,
}

#[derive(Debug, Clone)]
pub struct DiscoveredScanner {
    pub id: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub model_hint: String,
    pub family: DeviceFamily,
    pub transport_kind: TransportKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    UsbBulk,
    SerialPort,
    VirtualSimulator,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ScannerTransport: Send + Sync {
    async fn write_control(&mut self, buf: &[u8], timeout: Duration) -> Result<usize, SekonicError>;
    async fn read_control(
        &mut self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, SekonicError>;
    async fn read_image_pipe(
        &mut self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, SekonicError>;
    async fn purge(&mut self) -> Result<(), SekonicError>;
}
