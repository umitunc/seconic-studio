use crate::traits::ScannerTransport;
use async_trait::async_trait;
use sekonic_schema::error::SekonicError;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::{DataBits, Parity, SerialPortBuilderExt, SerialStream, StopBits};

pub struct NativeSerialTransport {
    port: SerialStream,
}

/// Lists all available RS-232 / USB-Serial COM ports on the host OS.
pub fn list_serial_ports() -> Vec<String> {
    tokio_serial::available_ports()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.port_name)
        .collect()
}

impl NativeSerialTransport {
    /// Opens an RS-232 serial port matching `OMR_OpenDeviceSerial` DCB settings (8N1).
    pub fn open(port_name: &str, baud_rate: u32) -> Result<Self, SekonicError> {
        let port = tokio_serial::new(port_name, baud_rate)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .open_native_async()
            .map_err(|e| SekonicError::Transport(format!("serial open {port_name}: {e}")))?;
        Ok(Self { port })
    }
}

#[async_trait]
impl ScannerTransport for NativeSerialTransport {
    async fn write_control(&mut self, buf: &[u8], timeout: Duration) -> Result<usize, SekonicError> {
        tokio::time::timeout(timeout, self.port.write(buf))
            .await
            .map_err(|_| SekonicError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            })?
            .map_err(|e| SekonicError::Transport(format!("serial write: {e}")))
    }

    async fn read_control(
        &mut self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, SekonicError> {
        tokio::time::timeout(timeout, self.port.read(buf))
            .await
            .map_err(|_| SekonicError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            })?
            .map_err(|e| SekonicError::Transport(format!("serial read: {e}")))
    }

    async fn read_image_pipe(
        &mut self,
        _buf: &mut [u8],
        _timeout: Duration,
    ) -> Result<usize, SekonicError> {
        Err(SekonicError::Transport(
            "RS-232 serial does not support a secondary CIS image pipe".into(),
        ))
    }

    async fn purge(&mut self) -> Result<(), SekonicError> {
        Ok(())
    }
}
