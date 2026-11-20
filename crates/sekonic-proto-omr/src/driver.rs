use crate::codec::{decode_ack_nak, encode_frame, STX};
use crate::commands::{
    cmd_cancel_error, cmd_eject, cmd_feed_sheet, cmd_get_barcode, cmd_get_mark_ad,
    cmd_get_marks_md, cmd_init, cmd_machine_name, cmd_move_hopper, cmd_reset, cmd_set_back_sensor,
    cmd_set_barcode_unit, cmd_set_columns, cmd_set_duplex, StackerTarget,
};
use crate::status_decoder::HardwareCondition;
use bytes::BytesMut;
use sekonic_schema::error::SekonicError;
use sekonic_schema::form_schema::OmrFormSchema;
use sekonic_schema::raw_capture::{Matrix2D, RawSheetCapture};
use sekonic_transport::traits::ScannerTransport;
use std::time::Duration;

const CONTROL_TIMEOUT: Duration = Duration::from_millis(5000);
const FEED_TIMEOUT: Duration = Duration::from_millis(20000);
const DATA_TIMEOUT: Duration = Duration::from_millis(10000);

/// Matches `OMRAPI.dll::_OMR_ReceiveACKorNAK` (`0x200` = 512 bytes) so USB Bulk IN never overflows.
const ACK_READ_BUF_LEN: usize = 512;

pub struct OmrScannerDriver<T: ScannerTransport> {
    transport: T,
    tx_buf: BytesMut,
    rx_buf: Vec<u8>,
    model_name: String,
    seq_counter: u64,
    barcode_supported: bool,
    use_md_fallback: bool,
}

impl<T: ScannerTransport> OmrScannerDriver<T> {
    pub fn new(transport: T) -> Self {
        let is_usb = !transport.uses_serial_framing();
        Self {
            transport,
            tx_buf: BytesMut::with_capacity(4096),
            rx_buf: vec![0u8; 16384],
            model_name: String::new(),
            seq_counter: 0,
            barcode_supported: false,
            // Real USB Family A scanners (SR-1800/3500/6000/6500) use MD0/MD1 directly.
            use_md_fallback: is_usb,
        }
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    /// Strips optional Serial `[STX][LEN_HI][LEN_LO][PAYLOAD][CRC_HI][CRC_LO]` wrapper if present.
    fn unwrap_response(raw: &[u8]) -> &[u8] {
        if raw.len() >= 5 && raw[0] == STX {
            let payload_len = usize::from(u16::from_be_bytes([raw[1], raw[2]]));
            if raw.len() >= 3 + payload_len {
                return &raw[3..3 + payload_len];
            }
        }
        raw
    }

    async fn send_and_recv_timeout(
        &mut self,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<Vec<u8>, SekonicError> {
        let cmd_label = String::from_utf8_lossy(payload).into_owned();

        // CRITICAL (`OMRAPI.dll @ 0x10003453`):
        // Mode 2 (RS-232 Serial) frames commands with `[STX][LEN_BE][PAYLOAD][CRC16_BE]`.
        // Mode 1 (USB Bulk) sends plain ASCII command bytes directly without STX/LEN/CRC16!
        let wire_bytes: &[u8] = if self.transport.uses_serial_framing() {
            encode_frame(payload, &mut self.tx_buf)?;
            &self.tx_buf
        } else {
            payload
        };

        self.transport
            .write_control(wire_bytes, timeout)
            .await
            .map_err(|e| match e {
                SekonicError::Timeout { timeout_ms } => SekonicError::Protocol(format!(
                    "[{cmd_label}] Yazma zaman aşımı ({timeout_ms} ms)"
                )),
                other => other,
            })?;

        let ack_n = self
            .transport
            .read_control(&mut self.rx_buf[..ACK_READ_BUF_LEN], timeout)
            .await
            .map_err(|e| match e {
                SekonicError::Timeout { timeout_ms } => SekonicError::Protocol(format!(
                    "[{cmd_label}] ACK/NAK beklenirken zaman aşımı ({timeout_ms} ms)"
                )),
                other => other,
            })?;

        if let Err(e) = decode_ack_nak(&self.rx_buf[..ack_n]) {
            return Err(match e {
                SekonicError::Nak { status } => SekonicError::Protocol(format!(
                    "[{cmd_label}] Cihaz komutu reddetti (NAK 0x{status:02X})"
                )),
                other => SekonicError::Protocol(format!("[{cmd_label}] {other}")),
            });
        }

        // Handle coalesced ACK + response in a single USB packet if it ever occurs.
        if ack_n > 1 {
            return Ok(Self::unwrap_response(&self.rx_buf[1..ack_n]).to_vec());
        }

        let resp_n = self
            .transport
            .read_control(&mut self.rx_buf, timeout)
            .await
            .map_err(|e| match e {
                SekonicError::Timeout { timeout_ms } => SekonicError::Protocol(format!(
                    "[{cmd_label}] Yanıt beklenirken zaman aşımı ({timeout_ms} ms)"
                )),
                other => other,
            })?;
        Ok(Self::unwrap_response(&self.rx_buf[..resp_n]).to_vec())
    }

    async fn send_and_recv(&mut self, payload: &[u8]) -> Result<Vec<u8>, SekonicError> {
        self.send_and_recv_timeout(payload, CONTROL_TIMEOUT).await
    }

    pub async fn initialize(
        &mut self,
        schema: &OmrFormSchema,
        enable_barcode: bool,
    ) -> Result<(), SekonicError> {
        let _ = self.transport.purge().await;

        // 1. Clear any sticky hardware/protocol error (`OMR_CancelError` -> `CE`)
        let _ = self.send_and_recv(cmd_cancel_error()).await;

        // 2. Reset / Initialize scanner (`SekOku.exe` uses `OMR_Reset` -> `SR`; fallback to `IS`)
        if self.send_and_recv(cmd_reset()).await.is_err() {
            self.send_and_recv(cmd_init()).await?;
        }

        // 3. Query machine model (`MN` -> `"MN0000SR-6500"`)
        if let Ok(mn) = self.send_and_recv(cmd_machine_name()).await {
            if mn.len() > 6 {
                self.model_name = String::from_utf8_lossy(&mn[6..])
                    .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                    .to_string();
            }
        }

        // 4. Configure number of columns (`OMR_SetNumberOfColumnsToRead`: `1..=48`)
        let cols = schema.grid.front_cols.clamp(1, 48);
        let _ = self.send_and_recv(&cmd_set_columns(cols)).await;

        // 5. Configure back sensor / duplex (`BR1,0`/`BR1,1` + `WB1`)
        let _ = self.send_and_recv(cmd_set_back_sensor(schema.duplex)).await;
        let _ = self
            .send_and_recv(&cmd_set_duplex(true, schema.duplex))
            .await;

        // 6. Optional barcode unit
        if enable_barcode {
            self.barcode_supported = self
                .send_and_recv(cmd_set_barcode_unit(true))
                .await
                .is_ok();
        } else {
            self.barcode_supported = false;
        }

        // 7. Raise motorized hopper table (`OMR_MoveHopper(1)` -> `HU1`) on SR-3500/6000/6500
        let _ = self.send_and_recv_timeout(cmd_move_hopper(true), FEED_TIMEOUT).await;

        Ok(())
    }

    /// Lowers motorized hopper table (`HU0`) cleanly when scanning finishes or stops.
    pub async fn finish_scan(&mut self) {
        let _ = self.send_and_recv(cmd_move_hopper(false)).await;
    }

    /// Decodes a single ASCII mark level character from `MD0`/`MD1` (`OMR_GetMarks @ 0x10003ff0`).
    /// Supports both `sub dl, 0x30` (`'0'..='?'` -> `0..=15`) and ASCII hex (`'A'..='F'` -> `10..=15`).
    #[inline]
    fn decode_md_level(ch: u8) -> u8 {
        match ch {
            b'0'..=b'?' => ch - b'0',
            b'A'..=b'F' => ch - b'A' + 10,
            b'a'..=b'f' => ch - b'a' + 10,
            _ => 0,
        }
    }

    /// Parses ASCII `MD0`/`MD1` mark matrix (`OMR_GetMarks @ 0x10003ff0`).
    /// Response format: 6-byte header `"MD0000"` (`[0x1001dd60] = rx + 6`), followed by
    /// comma-separated mark-level digits (`'0'/'1'` binary or `'0'..'?'` / `'0'..'F'` 16-level density).
    /// Automatically handles both row-major and column-major layouts.
    pub(crate) fn decode_md_into_slice(resp: &[u8], dst: &mut [u8], rows: u16, cols: u16) {
        let mut body = resp;
        if body.len() >= 6 && &body[..2] == b"MD" {
            body = &body[6..];
        } else if body.starts_with(b"MD") {
            body = &body[2..];
        }
        while let Some(&last) = body.last() {
            if last == 0 || last == b'\r' || last == b'\n' || last == b' ' {
                body = &body[..body.len() - 1];
            } else {
                break;
            }
        }

        let r_max = usize::from(rows);
        let c_max = usize::from(cols);

        let max_level = body
            .iter()
            .filter(|&&b| b != b',')
            .map(|&b| Self::decode_md_level(b))
            .max()
            .unwrap_or(0);
        let is_binary = max_level <= 1;
        let to_density = |ch: u8| -> u8 {
            let lvl = Self::decode_md_level(ch);
            if is_binary {
                if lvl >= 1 { 220 } else { 0 }
            } else {
                lvl.saturating_mul(17)
            }
        };

        if body.contains(&b',') {
            let tokens: Vec<&[u8]> = body.split(|&b| b == b',').filter(|t| !t.is_empty()).collect();
            let tok_count = tokens.len();
            let first_len = tokens.first().map(|t| t.len()).unwrap_or(0);

            // Detect whether each comma-separated token is a sensor column (length ~= rows, count ~= cols)
            // or a timing-mark row (length ~= cols, count ~= rows).
            let is_col_major = tok_count != r_max
                && (tok_count == c_max
                    || (first_len >= r_max && tok_count <= 48 && r_max > 48));

            if is_col_major {
                for (c, col_bits) in tokens.iter().enumerate() {
                    if c >= c_max {
                        break;
                    }
                    for (r, &ch) in col_bits.iter().enumerate() {
                        if r >= r_max {
                            break;
                        }
                        dst[r * c_max + c] = to_density(ch);
                    }
                }
            } else {
                for (r, row_bits) in tokens.iter().enumerate() {
                    if r >= r_max {
                        break;
                    }
                    for (c, &ch) in row_bits.iter().enumerate() {
                        if c >= c_max {
                            break;
                        }
                        dst[r * c_max + c] = to_density(ch);
                    }
                }
            }
        } else {
            for (i, &b) in body.iter().take(dst.len()).enumerate() {
                dst[i] = to_density(b);
            }
        }
    }

    async fn read_side_matrix(
        &mut self,
        back_side: bool,
        dst: &mut [u8],
        rows: u16,
        cols: u16,
    ) -> Result<(), SekonicError> {
        if !self.use_md_fallback {
            match self
                .send_and_recv_timeout(cmd_get_mark_ad(back_side), DATA_TIMEOUT)
                .await
            {
                Ok(resp) if resp.len() >= 2 => {
                    let bytes = &resp[2..];
                    let copy_len = bytes.len().min(dst.len());
                    dst[..copy_len].copy_from_slice(&bytes[..copy_len]);
                    return Ok(());
                }
                Err(_) | Ok(_) => {
                    self.use_md_fallback = true;
                }
            }
        }

        let md_resp = self
            .send_and_recv_timeout(cmd_get_marks_md(back_side), DATA_TIMEOUT)
            .await?;
        Self::decode_md_into_slice(&md_resp, dst, rows, cols);
        Ok(())
    }

    /// Feeds one sheet and reads its mark matrix (front + optional back) and barcode.
    /// Returns `Ok(None)` when the hopper is empty.
    pub async fn feed_and_read_sheet(
        &mut self,
        schema: &OmrFormSchema,
        read_barcode: bool,
    ) -> Result<Option<RawSheetCapture>, SekonicError> {
        let sf = self
            .send_and_recv_timeout(cmd_feed_sheet(), FEED_TIMEOUT)
            .await?;

        if sf.len() >= 4 {
            let status_str = std::str::from_utf8(&sf[2..4]).unwrap_or("99");
            let cond = HardwareCondition::from_status_code(status_str);
            match cond {
                HardwareCondition::Ready => {}
                HardwareCondition::HopperEmpty => return Ok(None),
                fault => {
                    return Err(SekonicError::HardwareFault {
                        code: status_str.to_string(),
                        message: fault.message_tr().to_string(),
                    });
                }
            }
        }

        let mut matrix = Matrix2D::new(schema.grid.front_rows, schema.grid.front_cols, schema.duplex);
        self.read_side_matrix(
            false,
            &mut matrix.front,
            schema.grid.front_rows,
            schema.grid.front_cols,
        )
        .await?;

        if schema.duplex {
            if let Some(back_dst) = matrix.back.as_mut() {
                let br = if schema.grid.back_rows > 0 {
                    schema.grid.back_rows
                } else {
                    schema.grid.front_rows
                };
                let bc = if schema.grid.back_cols > 0 {
                    schema.grid.back_cols
                } else {
                    schema.grid.front_cols
                };
                let _ = self.read_side_matrix(true, back_dst, br, bc).await;
            }
        }

        let barcode = if read_barcode && self.barcode_supported {
            match self.send_and_recv(cmd_get_barcode()).await {
                Ok(bd) if bd.len() > 6 => {
                    let s = String::from_utf8_lossy(&bd[6..]).trim().to_string();
                    if s.is_empty() { None } else { Some(s) }
                }
                _ => None,
            }
        } else {
            None
        };

        self.seq_counter += 1;
        Ok(Some(RawSheetCapture {
            seq_no: self.seq_counter,
            timestamp_ms: 0,
            barcode,
            matrix,
        }))
    }

    pub async fn eject_sheet(&mut self, target: StackerTarget) -> Result<(), SekonicError> {
        let _ = self.send_and_recv_timeout(cmd_eject(target), FEED_TIMEOUT).await?;
        Ok(())
    }
}
