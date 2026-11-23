use crate::form_schema::SheetSide;
use serde::{Deserialize, Serialize};

/// Raw 0-255 analog optical density matrix captured from either Family A (AD) or Family B (CIS).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RawSheetCapture {
    pub seq_no: u64,
    pub timestamp_ms: u64,
    pub barcode: Option<String>,
    pub matrix: Matrix2D,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_wire_sample: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_front_rows: Option<u16>,
    #[serde(default)]
    pub right_edge_timing_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Matrix2D {
    pub front: Vec<u8>,
    pub back: Option<Vec<u8>>,
    pub rows: u16,
    pub cols: u16,
}

impl Matrix2D {
    pub fn new(rows: u16, cols: u16, duplex: bool) -> Self {
        let len = usize::from(rows) * usize::from(cols);
        Self {
            front: vec![0; len],
            back: if duplex { Some(vec![0; len]) } else { None },
            rows,
            cols,
        }
    }

    #[inline]
    pub fn get(&self, side: SheetSide, row: u16, col: u16) -> u8 {
        if row >= self.rows || col >= self.cols {
            return 0;
        }
        let idx = usize::from(row) * usize::from(self.cols) + usize::from(col);
        match side {
            SheetSide::Front => self.front.get(idx).copied().unwrap_or(0),
            SheetSide::Back => self
                .back
                .as_ref()
                .and_then(|b| b.get(idx).copied())
                .unwrap_or(0),
        }
    }

    #[inline]
    pub fn set(&mut self, side: SheetSide, row: u16, col: u16, val: u8) {
        if row >= self.rows || col >= self.cols {
            return;
        }
        let idx = usize::from(row) * usize::from(self.cols) + usize::from(col);
        match side {
            SheetSide::Front => {
                if let Some(cell) = self.front.get_mut(idx) {
                    *cell = val;
                }
            }
            SheetSide::Back => {
                if let Some(back) = self.back.as_mut() {
                    if let Some(cell) = back.get_mut(idx) {
                        *cell = val;
                    }
                }
            }
        }
    }
}
