use crate::traits::ScannerTransport;
use async_trait::async_trait;
use sekonic_schema::error::SekonicError;
use std::time::Duration;

/// Browser Web Serial adapter (wired via `packages/sekonic-web-sdk` in WP-10).
pub struct WebSerialTransport;

#[async_trait(?Send)]
impl ScannerTransport for WebSerialTransport {
    async fn write_control(&mut self, _buf: &[u8], _timeout: Duration) -> Result<usize, SekonicError> {
        Err(SekonicError::NotConnected)
    }

    async fn read_control(
        &mut self,
        _buf: &mut [u8],
        _timeout: Duration,
    ) -> Result<usize, SekonicError> {
        Err(SekonicError::NotConnected)
    }

    async fn read_image_pipe(
        &mut self,
        _buf: &mut [u8],
        _timeout: Duration,
    ) -> Result<usize, SekonicError> {
        Err(SekonicError::Transport(
            "Web Serial does not support secondary CIS image pipe".into(),
        ))
    }

    async fn purge(&mut self) -> Result<(), SekonicError> {
        Ok(())
    }
}
