use crate::paper_generator::GeneratedSheet;
use sekonic_schema::form_schema::SheetSide;
use sekonic_transport::memory_channel::InMemoryDeviceEnd;
use std::collections::VecDeque;

const STX: u8 = 0x02;
const ACK: u8 = 0x06;
const NAK: u8 = 0x15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackerRoute {
    Main,
    Sub,
}

/// Reflected CRC-16/CCITT (poly 0x8408, init 0xFFFF, xorout 0xFFFF) matching `OMRAPI.dll @ 0x1010`.
pub fn crc16_omr(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= u16::from(b);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0x8408;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF
}

/// Virtual SR-6500 hardware emulator running over `InMemoryDeviceEnd`.
pub struct VirtualSr6500 {
    pub hopper: VecDeque<GeneratedSheet>,
    pub active_sheet: Option<GeneratedSheet>,
    pub main_stacker: Vec<u64>,
    pub sub_stacker: Vec<u64>,
    pub model_name: &'static str,
}

impl VirtualSr6500 {
    pub fn new(sheets: Vec<GeneratedSheet>) -> Self {
        Self {
            hopper: VecDeque::from(sheets),
            active_sheet: None,
            main_stacker: Vec::new(),
            sub_stacker: Vec::new(),
            model_name: "SR-6500",
        }
    }

    pub async fn run(mut self, dev: InMemoryDeviceEnd) -> Self {
        while let Some(frame) = dev.recv_from_host().await {
            if frame.len() < 5 || frame[0] != STX {
                let _ = dev.send_to_host(vec![NAK, 0x01]).await;
                continue;
            }
            let payload_len = usize::from(u16::from_be_bytes([frame[1], frame[2]]));
            if frame.len() != payload_len + 5 {
                let _ = dev.send_to_host(vec![NAK, 0x02]).await;
                continue;
            }
            let expected_crc = crc16_omr(&frame[..payload_len + 3]);
            let got_crc = u16::from_be_bytes([frame[payload_len + 3], frame[payload_len + 4]]);
            if expected_crc != got_crc {
                let _ = dev.send_to_host(vec![NAK, 0x03]).await;
                continue;
            }

            let cmd = String::from_utf8_lossy(&frame[3..3 + payload_len]).to_string();
            let _ = dev.send_to_host(vec![ACK]).await;

            if let Some(resp) = self.handle_command(&cmd) {
                let _ = dev.send_to_host(resp).await;
            }
        }
        self
    }

    fn handle_command(&mut self, cmd: &str) -> Option<Vec<u8>> {
        if cmd == "IS" || cmd.starts_with("NC1") || cmd.starts_with("WB1") || cmd.starts_with("BC1") || cmd.starts_with("PR1") {
            return Some(format!("{}0000", &cmd[..2]).into_bytes());
        }
        if cmd == "MN" {
            return Some(format!("MN0000{}", self.model_name).into_bytes());
        }
        if cmd == "DI" {
            return Some(b"DI0000SEKONIC-SIM-V1".to_vec());
        }
        if cmd == "ST" {
            let hopper_empty = if self.hopper.is_empty() && self.active_sheet.is_none() {
                "01"
            } else {
                "00"
            };
            return Some(format!("ST{hopper_empty}00").into_bytes());
        }
        if cmd == "SF" {
            if let Some(sheet) = self.hopper.pop_front() {
                self.active_sheet = Some(sheet);
                return Some(b"SF0000".to_vec());
            } else {
                // Sheet Empty status (0x01)
                return Some(b"SF0100".to_vec());
            }
        }
        if cmd == "AD0" || cmd == "AD1" {
            let side = if cmd == "AD1" { SheetSide::Back } else { SheetSide::Front };
            if let Some(sheet) = &self.active_sheet {
                let mut out = b"AD".to_vec();
                match side {
                    SheetSide::Front => out.extend_from_slice(&sheet.matrix.front),
                    SheetSide::Back => {
                        if let Some(back) = &sheet.matrix.back {
                            out.extend_from_slice(back);
                        } else {
                            out.resize(2 + sheet.matrix.front.len(), 0);
                        }
                    }
                }
                return Some(out);
            }
            return Some(b"AD".to_vec());
        }
        if cmd == "BD1" {
            let bc = self
                .active_sheet
                .as_ref()
                .and_then(|s| s.barcode.as_deref())
                .unwrap_or("");
            return Some(format!("BD0000{bc}").into_bytes());
        }
        if cmd == "ER0" || cmd == "ER1" {
            if let Some(sheet) = self.active_sheet.take() {
                if cmd == "ER1" {
                    self.sub_stacker.push(sheet.seq_no);
                } else {
                    self.main_stacker.push(sheet.seq_no);
                }
            }
            return Some(b"ER0000".to_vec());
        }
        Some(format!("{}0000", cmd.get(..2).unwrap_or("OK")).into_bytes())
    }
}
