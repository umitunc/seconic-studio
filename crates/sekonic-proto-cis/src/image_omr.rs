use crate::calibration::CisShadingProfile;
use crate::commands::{
    cmd_eject, cmd_init, cmd_scan_start, cmd_set_back_side, cmd_set_light_color,
    cmd_set_resolution,
};
use sekonic_schema::error::SekonicError;
use sekonic_schema::form_schema::{OmrFormSchema, SheetSide};
use sekonic_schema::raw_capture::{Matrix2D, RawSheetCapture};
use sekonic_transport::traits::ScannerTransport;
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(2500);

/// Converts an 8-bit grayscale CIS image (255 = white, 0 = dark mark) into a 0-255 `Matrix2D`
/// using left-edge timing mark detection and ROI luminance inversion.
pub fn extract_omr_matrix_from_image(
    img: &[u8],
    width_px: u16,
    height_px: u16,
    rows: u16,
    cols: u16,
    shading: Option<&CisShadingProfile>,
) -> Result<Matrix2D, SekonicError> {
    let w = usize::from(width_px);
    let h = usize::from(height_px);
    if img.len() < w * h {
        return Err(SekonicError::Protocol(format!(
            "CIS image too small: got {}, expected {}",
            img.len(),
            w * h
        )));
    }

    let mut work = img[..w * h].to_vec();
    if let Some(prof) = shading {
        for row_slice in work.chunks_exact_mut(w) {
            prof.normalize_scanline(row_slice);
        }
    }

    // Detect timing-mark y centers along x = 2..10
    let mut timing_ys = Vec::with_capacity(usize::from(rows));
    let mut in_mark = false;
    let mut start_y = 0usize;
    for y in 0..h {
        let mut dark_sum = 0u32;
        for x in 2..10.min(w) {
            dark_sum += u32::from(255u8.saturating_sub(work[y * w + x]));
        }
        let is_dark = (dark_sum / 8) > 100;
        if is_dark && !in_mark {
            in_mark = true;
            start_y = y;
        } else if !is_dark && in_mark {
            in_mark = false;
            timing_ys.push((start_y + y) / 2);
        }
    }
    if in_mark {
        timing_ys.push((start_y + h) / 2);
    }

    let mut matrix = Matrix2D::new(rows, cols, false);
    let row_step = h / usize::from(rows.max(1));
    let col_step = w / usize::from((cols + 2).max(1));

    for r in 0..rows {
        let cy = timing_ys
            .get(usize::from(r))
            .copied()
            .unwrap_or_else(|| usize::from(r) * row_step + row_step / 2);
        for c in 0..cols {
            let cx = (usize::from(c) + 1) * col_step + col_step / 2;
            let mut sum = 0u32;
            let mut cnt = 0u32;
            for dy in -1i32..=2 {
                let yy = (cy as i32 + dy).clamp(0, (h - 1) as i32) as usize;
                for dx in -1i32..=2 {
                    let xx = (cx as i32 + dx).clamp(0, (w - 1) as i32) as usize;
                    sum += u32::from(255u8.saturating_sub(work[yy * w + xx]));
                    cnt += 1;
                }
            }
            let density = (sum / cnt.max(1)).min(255) as u8;
            matrix.set(SheetSide::Front, r, c, density);
        }
    }

    Ok(matrix)
}

pub struct CisScannerDriver<T: ScannerTransport> {
    transport: T,
    width_px: u16,
    height_px: u16,
    seq_counter: u64,
}

impl<T: ScannerTransport> CisScannerDriver<T> {
    pub fn new(transport: T, width_px: u16, height_px: u16) -> Self {
        Self {
            transport,
            width_px,
            height_px,
            seq_counter: 0,
        }
    }

    async fn send_ctrl(&mut self, cmd: &[u8]) -> Result<Vec<u8>, SekonicError> {
        self.transport.write_control(cmd, DEFAULT_TIMEOUT).await?;
        let mut ack = [0u8; 4];
        let n = self.transport.read_control(&mut ack, DEFAULT_TIMEOUT).await?;
        if n == 0 || ack[0] != 0x06 {
            return Err(SekonicError::Protocol("CIS missing ACK".into()));
        }
        let mut resp = [0u8; 64];
        let rn = self
            .transport
            .read_control(&mut resp, DEFAULT_TIMEOUT)
            .await?;
        Ok(resp[..rn].to_vec())
    }

    pub async fn initialize(&mut self, schema: &OmrFormSchema) -> Result<(), SekonicError> {
        let _ = self.send_ctrl(&cmd_init(0)).await?;
        let _ = self.send_ctrl(&cmd_set_back_side(schema.duplex)).await?;
        let _ = self
            .send_ctrl(&cmd_set_light_color(0, schema.dropout_color))
            .await?;
        let _ = self.send_ctrl(&cmd_set_resolution(0, true)).await?;
        Ok(())
    }

    pub async fn scan_sheet(
        &mut self,
        schema: &OmrFormSchema,
    ) -> Result<Option<RawSheetCapture>, SekonicError> {
        let resp = self.send_ctrl(cmd_scan_start()).await?;
        if resp.get(2..4) == Some(b"01") {
            return Ok(None);
        }
        let expected_len = usize::from(self.width_px) * usize::from(self.height_px);
        let mut img_buf = vec![0u8; expected_len];
        let mut read_total = 0usize;
        while read_total < expected_len {
            let n = self
                .transport
                .read_image_pipe(&mut img_buf[read_total..], DEFAULT_TIMEOUT)
                .await?;
            if n == 0 {
                break;
            }
            read_total += n;
        }

        let matrix = extract_omr_matrix_from_image(
            &img_buf[..read_total],
            self.width_px,
            self.height_px,
            schema.grid.front_rows,
            schema.grid.front_cols,
            None,
        )?;
        self.seq_counter += 1;
        Ok(Some(RawSheetCapture {
            seq_no: self.seq_counter,
            timestamp_ms: 0,
            barcode: None,
            matrix,
            raw_wire_sample: None,
            observed_front_rows: None,
            right_edge_timing_detected: false,
        }))
    }

    pub async fn eject(&mut self, sub_stacker: bool) -> Result<(), SekonicError> {
        let _ = self.send_ctrl(&cmd_eject(sub_stacker)).await?;
        Ok(())
    }
}
