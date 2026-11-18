use crate::paper_generator::{render_cis_grayscale_image, GeneratedSheet};
use sekonic_schema::form_schema::SheetSide;
use sekonic_transport::memory_channel::InMemoryDeviceEnd;
use std::collections::VecDeque;

const ACK: u8 = 0x06;

/// Virtual SR-55D CIS scanner emulator with dual-pipe control + 8-bit grayscale image streaming.
pub struct VirtualSr55d {
    pub hopper: VecDeque<GeneratedSheet>,
    pub active_sheet: Option<GeneratedSheet>,
    pub width_px: u16,
    pub height_px: u16,
}

impl VirtualSr55d {
    pub fn new(sheets: Vec<GeneratedSheet>, width_px: u16, height_px: u16) -> Self {
        Self {
            hopper: VecDeque::from(sheets),
            active_sheet: None,
            width_px,
            height_px,
        }
    }

    pub async fn run(mut self, dev: InMemoryDeviceEnd) -> Self {
        while let Some(frame) = dev.recv_from_host().await {
            let cmd = String::from_utf8_lossy(&frame)
                .trim_matches(|c| c == '\x02' || c == '\x03')
                .to_string();

            let _ = dev.send_to_host(vec![ACK]).await;

            if cmd == "SF1" {
                if let Some(sheet) = self.hopper.pop_front() {
                    let img = render_cis_grayscale_image(
                        &sheet.matrix,
                        SheetSide::Front,
                        self.width_px,
                        self.height_px,
                    );
                    self.active_sheet = Some(sheet);
                    let _ = dev.send_to_host(b"SF0000".to_vec()).await;
                    let _ = dev.send_image_to_host(img).await;
                } else {
                    let _ = dev.send_to_host(b"SF0100".to_vec()).await;
                }
            } else if cmd.starts_with("ER") {
                self.active_sheet = None;
                let _ = dev.send_to_host(b"ER0000".to_vec()).await;
            } else {
                let prefix = cmd.get(..2).unwrap_or("OK");
                let _ = dev.send_to_host(format!("{prefix}0000").into_bytes()).await;
            }
        }
        self
    }
}
