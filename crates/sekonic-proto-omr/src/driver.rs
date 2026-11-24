use crate::codec::{decode_ack_nak, encode_frame, STX};
use crate::commands::{
    cmd_cancel_error, cmd_eject, cmd_feed_sheet, cmd_get_barcode, cmd_get_mark_ad,
    cmd_get_marks_md, cmd_init, cmd_machine_name, cmd_move_hopper, cmd_reset, cmd_set_back_sensor,
    cmd_set_barcode_unit, cmd_set_columns, cmd_set_duplex, cmd_set_reading_method_16level,
    StackerTarget,
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
const MD_CONTINUATION_TIMEOUT: Duration = Duration::from_millis(60);

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
    rm_16level: bool,
    init_log: Vec<String>,
}

impl<T: ScannerTransport> OmrScannerDriver<T> {
    pub fn new(transport: T) -> Self {
        let is_usb = !transport.uses_serial_framing();
        Self {
            transport,
            tx_buf: BytesMut::with_capacity(4096),
            rx_buf: vec![0u8; 32768],
            model_name: String::new(),
            seq_counter: 0,
            barcode_supported: false,
            // Real USB Family A scanners (SR-1800/3500/6000/6500) use MD0/MD1 directly.
            use_md_fallback: is_usb,
            rm_16level: false,
            init_log: Vec::new(),
        }
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    pub fn init_log(&self) -> &[String] {
        &self.init_log
    }

    pub fn is_rm_16level(&self) -> bool {
        self.rm_16level
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
        self.init_log.clear();
        let _ = self.transport.purge().await;

        let fmt_res = |cmd: &str, r: &Result<Vec<u8>, SekonicError>| -> String {
            match r {
                Ok(b) => {
                    let s = String::from_utf8_lossy(b);
                    let clean: String = s.chars().filter(|c| !c.is_control()).take(64).collect();
                    format!("{cmd} -> OK ({}B: \"{clean}\")", b.len())
                }
                Err(e) => format!("{cmd} -> ERR ({e})"),
            }
        };

        // 1. Clear any sticky hardware/protocol error (`OMR_CancelError` -> `CE`)
        let r_ce = self.send_and_recv(cmd_cancel_error()).await;
        self.init_log.push(fmt_res("CE", &r_ce));

        // 2. Reset / Initialize scanner (`SekOku.exe` uses `OMR_Reset` -> `SR`; fallback to `IS`)
        let r_sr = self.send_and_recv(cmd_reset()).await;
        self.init_log.push(fmt_res("SR", &r_sr));
        if r_sr.is_err() {
            let r_is = self.send_and_recv(cmd_init()).await;
            self.init_log.push(fmt_res("IS", &r_is));
            r_is?;
        }

        // 3. Query machine model (`MN` -> `"MN0000SR-6500"`)
        let r_mn = self.send_and_recv(cmd_machine_name()).await;
        self.init_log.push(fmt_res("MN", &r_mn));
        if let Ok(mn) = r_mn {
            if mn.len() > 6 {
                self.model_name = String::from_utf8_lossy(&mn[6..])
                    .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                    .to_string();
            }
        }

        // 4. Select 16-level density mode (`SekOku.exe @ 0x4b6da4`: `OMR_SetReadingMethod(3, 0)` -> `RM1,3`)
        let r_rm = self.send_and_recv(cmd_set_reading_method_16level()).await;
        self.init_log.push(fmt_res("RM1,3", &r_rm));
        self.rm_16level = r_rm.is_ok();

        // 5. Configure all 48 physical sensor channels (`SekOku.exe @ 0x4b6deb`: `OMR_SetNumberOfColumnsToRead(48)`)
        let r_nc = self.send_and_recv(&cmd_set_columns(48)).await;
        self.init_log.push(fmt_res("NC1,48", &r_nc));

        // 6. Configure back sensor (`SekOku.exe`: `OMR_SetBackSensorUnit` -> `BR1,0`/`BR1,1`; fallback `WB1`)
        let r_br = self.send_and_recv(cmd_set_back_sensor(schema.duplex)).await;
        self.init_log.push(fmt_res("BR1", &r_br));
        if r_br.is_err() {
            let r_wb = self
                .send_and_recv(&cmd_set_duplex(true, schema.duplex))
                .await;
            self.init_log.push(fmt_res("WB1", &r_wb));
        }

        // 7. Optional barcode unit
        if enable_barcode {
            let r_bc = self.send_and_recv(cmd_set_barcode_unit(true)).await;
            self.init_log.push(fmt_res("BC1", &r_bc));
            self.barcode_supported = r_bc.is_ok();
        } else {
            self.barcode_supported = false;
        }

        // 8. Raise motorized hopper table (`OMR_MoveHopper(1)` -> `HU1`) on SR-3500/6000/6500
        let r_hu = self
            .send_and_recv_timeout(cmd_move_hopper(true), FEED_TIMEOUT)
            .await;
        self.init_log.push(fmt_res("HU1", &r_hu));

        Ok(())
    }

    /// Lowers motorized hopper table (`HU0`) cleanly when scanning finishes or stops.
    pub async fn finish_scan(&mut self) {
        let _ = self.send_and_recv(cmd_move_hopper(false)).await;
    }

    /// Decodes a single ASCII mark level character from `MD0`/`MD1` (`OMR_GetMarks @ 0x10004130`).
    /// Real SR-1800/3500/6000/6500 hardware in `RM1,3` mode executes `sub dl, 0x30` (`[ebp] = 0x10`),
    /// producing values `0..=16` encoded as `'0'..='@'` (`0x30..=0x40`), where `'@'` (`16`) is the
    /// darkest mark! Also supports hex `'A'..='F'` -> `10..=15` for simulator/legacy variants.
    #[inline]
    fn decode_md_level(ch: u8) -> u8 {
        match ch {
            b'0'..=b'@' => ch - b'0',
            b'A'..=b'F' => ch - b'A' + 10,
            b'a'..=b'f' => ch - b'a' + 10,
            _ => 0,
        }
    }

    /// Parses ASCII `MD0`/`MD1` mark matrix (`OMR_GetMarks @ 0x10003ff0` & `SekOku.exe @ 0x4aef74`).
    /// Response format: 6-byte header `"MD00@@"`, followed by comma-separated timing-mark rows
    /// (`num_tokens = rows`, `token_len = 48` physical sensor columns).
    /// Returns `(observed_rows, right_edge_timing_detected)`.
    pub(crate) fn decode_md_into_slice(
        resp: &[u8],
        dst: &mut [u8],
        rows: u16,
        cols: u16,
        force_16level: bool,
    ) -> (u16, bool) {
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
        let is_binary = !force_16level && max_level <= 1;
        // Matches `SekOku.exe @ 0x4aefd0..0x4af042` + `sekonic.ini` (`Üst Sınır = 9`):
        // - `0..=4`: `0..=32` (< default `min_mark_density = 48`; `lvl = 4` -> `32 >= 30` when `.fmt` uses `03`)
        // - `5..=8` (light marks / pencil erasures): `64, 80, 96, 112` (< `saturation_density = 135`)
        // - `9..=16` (normal/dark marks `>= Üst Sınır 9`): `140..=238` (>= `saturation_density = 135`)
        let to_density = |ch: u8| -> u8 {
            let lvl = Self::decode_md_level(ch);
            if is_binary {
                if lvl >= 1 { 220 } else { 0 }
            } else if lvl <= 4 {
                lvl * 8
            } else if lvl <= 8 {
                64 + (lvl - 5) * 16
            } else {
                140u8.saturating_add((lvl - 9).saturating_mul(14))
            }
        };

        if body.contains(&b',') {
            let tokens: Vec<&[u8]> = body.split(|&b| b == b',').filter(|t| !t.is_empty()).collect();
            let observed_rows = u16::try_from(tokens.len()).unwrap_or(u16::MAX);

            // On SR-6000 (`NC1,48`), 210mm A4 paper covers channels `0..44`; uncovered rightmost
            // channels (`45..47`) sit over the black scanner platen outside the right edge of the
            // paper and read dark (`12..16`) on every single row with zero white-paper rows.
            // Scan strictly inward from channel 47 and stop at the first channel that sees paper.
            let mut platen_mask = [false; 48];
            if tokens.len() >= 15 {
                for ch_idx in (43..48).rev() {
                    let mut white_rows = 0usize;
                    let mut dark_rows = 0usize;
                    for row in &tokens {
                        if let Some(&b) = row.get(ch_idx) {
                            let lvl = Self::decode_md_level(b);
                            if lvl <= 4 {
                                white_rows += 1;
                            } else if lvl >= 11 {
                                dark_rows += 1;
                            }
                        }
                    }
                    if white_rows == 0 && dark_rows * 100 >= tokens.len() * 92 {
                        platen_mask[ch_idx] = true;
                    } else {
                        break;
                    }
                }
            }

            for (r, row_bits) in tokens.iter().enumerate() {
                if r < r_max {
                    for (c, &ch) in row_bits.iter().take(c_max).enumerate() {
                        dst[r * c_max + c] = if c < 48 && platen_mask[c] {
                            0
                        } else {
                            to_density(ch)
                        };
                    }
                }
            }
            (observed_rows, false)
        } else {
            let src_stride = if c_max < 48 && body.len() >= r_max * 48 {
                48
            } else {
                c_max.max(1)
            };
            let observed_rows = if body.is_empty() {
                0
            } else {
                u16::try_from(body.len() / src_stride).unwrap_or(rows)
            };
            for r in 0..r_max {
                let row_start = r * src_stride;
                if row_start >= body.len() {
                    break;
                }
                let row_end = (row_start + src_stride).min(body.len());
                for (c, &ch) in body[row_start..row_end].iter().take(c_max).enumerate() {
                    dst[r * c_max + c] = to_density(ch);
                }
            }
            (observed_rows, false)
        }
    }

    async fn read_side_matrix(
        &mut self,
        back_side: bool,
        dst: &mut [u8],
        rows: u16,
        cols: u16,
    ) -> Result<(String, Option<u16>, bool), SekonicError> {
        if !self.use_md_fallback {
            match self
                .send_and_recv_timeout(cmd_get_mark_ad(back_side), DATA_TIMEOUT)
                .await
            {
                Ok(resp) if resp.len() >= 2 => {
                    let bytes = &resp[2..];
                    let copy_len = bytes.len().min(dst.len());
                    dst[..copy_len].copy_from_slice(&bytes[..copy_len]);
                    let min_v = bytes.iter().copied().min().unwrap_or(0);
                    let max_v = bytes.iter().copied().max().unwrap_or(0);
                    return Ok((
                        format!(
                            "AD{} len={} min={} max={}",
                            u8::from(back_side),
                            bytes.len(),
                            min_v,
                            max_v
                        ),
                        None,
                        false,
                    ));
                }
                Err(_) | Ok(_) => {
                    self.use_md_fallback = true;
                }
            }
        }

        let mut md_resp = self
            .send_and_recv_timeout(cmd_get_marks_md(back_side), DATA_TIMEOUT)
            .await?;

        // Reassemble multi-chunk USB / WDM (`UsbSkOmr`) reads if a driver split on a 64/512/4096B boundary
        // before all expected rows arrived. Short-packet terminated USB transfers skip this immediately (0ms).
        if !self.transport.uses_serial_framing() {
            let expected_tokens = usize::from(rows);
            while md_resp.len() >= 64 && md_resp.len() % 64 == 0 {
                let current_tokens = md_resp.iter().filter(|&&b| b == b',').count() + 1;
                if current_tokens >= expected_tokens {
                    break;
                }
                match self
                    .transport
                    .read_control(&mut self.rx_buf, MD_CONTINUATION_TIMEOUT)
                    .await
                {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        md_resp.extend_from_slice(&self.rx_buf[..n]);
                        if n % 64 != 0 {
                            break;
                        }
                    }
                }
            }
        }

        let (obs_rows, right_timing) =
            Self::decode_md_into_slice(&md_resp, dst, rows, cols, self.rm_16level);
        Ok((
            String::from_utf8_lossy(&md_resp)
                .trim_end_matches('\0')
                .to_string(),
            Some(obs_rows),
            right_timing,
        ))
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
        let (front_wire, observed_front_rows, right_edge_timing_detected) = self
            .read_side_matrix(
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
            raw_wire_sample: Some(front_wire),
            observed_front_rows,
            right_edge_timing_detected,
        }))
    }

    pub async fn eject_sheet(&mut self, target: StackerTarget) -> Result<(), SekonicError> {
        let _ = self.send_and_recv_timeout(cmd_eject(target), FEED_TIMEOUT).await?;
        Ok(())
    }
}
