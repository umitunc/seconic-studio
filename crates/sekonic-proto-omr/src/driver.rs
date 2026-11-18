use crate::codec::{decode_ack_nak, encode_frame};
use crate::commands::{
    cmd_eject, cmd_feed_sheet, cmd_get_barcode, cmd_get_mark_ad, cmd_init, cmd_machine_name,
    cmd_set_barcode_unit, cmd_set_columns, cmd_set_duplex, StackerTarget,
};
use crate::status_decoder::HardwareCondition;
use bytes::BytesMut;
use sekonic_schema::error::SekonicError;
use sekonic_schema::form_schema::OmrFormSchema;
use sekonic_schema::raw_capture::{Matrix2D, RawSheetCapture};
use sekonic_transport::traits::ScannerTransport;
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_millis(2000);

pub struct OmrScannerDriver<T: ScannerTransport> {
    transport: T,
    tx_buf: BytesMut,
    rx_buf: Vec<u8>,
    model_name: String,
    seq_counter: u64,
}

impl<T: ScannerTransport> OmrScannerDriver<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            tx_buf: BytesMut::with_capacity(4096),
            rx_buf: vec![0u8; 8192],
            model_name: String::new(),
            seq_counter: 0,
        }
    }

    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    async fn send_and_recv(&mut self, payload: &[u8]) -> Result<Vec<u8>, SekonicError> {
        encode_frame(payload, &mut self.tx_buf)?;
        self.transport
            .write_control(&self.tx_buf, DEFAULT_TIMEOUT)
            .await?;

        let ack_n = self
            .transport
            .read_control(&mut self.rx_buf[..2], DEFAULT_TIMEOUT)
            .await?;
        decode_ack_nak(&self.rx_buf[..ack_n])?;

        let resp_n = self
            .transport
            .read_control(&mut self.rx_buf, DEFAULT_TIMEOUT)
            .await?;
        Ok(self.rx_buf[..resp_n].to_vec())
    }

    pub async fn initialize(&mut self, schema: &OmrFormSchema, enable_barcode: bool) -> Result<(), SekonicError> {
        let _ = self.send_and_recv(cmd_init()).await?;
        let mn = self.send_and_recv(cmd_machine_name()).await?;
        if mn.len() > 6 {
            self.model_name = String::from_utf8_lossy(&mn[6..]).to_string();
        }
        let _ = self
            .send_and_recv(&cmd_set_columns(schema.grid.front_cols))
            .await?;
        let _ = self
            .send_and_recv(&cmd_set_duplex(true, schema.duplex))
            .await?;
        let _ = self
            .send_and_recv(cmd_set_barcode_unit(enable_barcode))
            .await?;
        Ok(())
    }

    /// Feeds one sheet and reads its 0-255 analog density matrix (front + optional back) and barcode.
    /// Returns `Ok(None)` when the hopper is empty.
    pub async fn feed_and_read_sheet(
        &mut self,
        schema: &OmrFormSchema,
        read_barcode: bool,
    ) -> Result<Option<RawSheetCapture>, SekonicError> {
        let sf = self.send_and_recv(cmd_feed_sheet()).await?;
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

        let front_resp = self.send_and_recv(cmd_get_mark_ad(false)).await?;
        if front_resp.len() < 2 {
            return Err(SekonicError::Protocol("AD0 response too short".into()));
        }

        let mut matrix = Matrix2D::new(schema.grid.front_rows, schema.grid.front_cols, schema.duplex);
        let front_bytes = &front_resp[2..];
        let copy_len = front_bytes.len().min(matrix.front.len());
        matrix.front[..copy_len].copy_from_slice(&front_bytes[..copy_len]);

        if schema.duplex {
            let back_resp = self.send_and_recv(cmd_get_mark_ad(true)).await?;
            if back_resp.len() >= 2 {
                if let Some(back_dst) = matrix.back.as_mut() {
                    let back_bytes = &back_resp[2..];
                    let blen = back_bytes.len().min(back_dst.len());
                    back_dst[..blen].copy_from_slice(&back_bytes[..blen]);
                }
            }
        }

        let barcode = if read_barcode {
            let bd = self.send_and_recv(cmd_get_barcode()).await?;
            if bd.len() > 6 {
                let s = String::from_utf8_lossy(&bd[6..]).trim().to_string();
                if s.is_empty() { None } else { Some(s) }
            } else {
                None
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
        let _ = self.send_and_recv(cmd_eject(target)).await?;
        Ok(())
    }
}
