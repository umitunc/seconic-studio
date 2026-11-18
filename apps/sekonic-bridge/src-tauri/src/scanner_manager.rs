use sekonic_transport::native_usb::discover_usb_scanners;
use sekonic_transport::traits::DiscoveredScanner;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;

pub struct ScannerManagerState {
    pub simulator_mode: AtomicBool,
    pub scanning: AtomicBool,
    pub event_tx: broadcast::Sender<String>,
}

impl ScannerManagerState {
    pub fn new() -> Arc<Self> {
        let (event_tx, _) = broadcast::channel(256);
        Arc::new(Self {
            simulator_mode: AtomicBool::new(true),
            scanning: AtomicBool::new(false),
            event_tx,
        })
    }

    pub fn list_devices(&self) -> Vec<DiscoveredScanner> {
        discover_usb_scanners().unwrap_or_default()
    }

    pub fn is_simulator_mode(&self) -> bool {
        self.simulator_mode.load(Ordering::Relaxed)
    }

    pub fn set_simulator_mode(&self, enabled: bool) {
        self.simulator_mode.store(enabled, Ordering::Relaxed);
    }
}
