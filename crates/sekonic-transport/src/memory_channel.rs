use crate::traits::ScannerTransport;
use async_trait::async_trait;
use sekonic_schema::error::SekonicError;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

/// In-memory full-duplex transport channel for deterministic simulator & unit testing.
pub struct InMemoryTransport {
    tx: mpsc::Sender<Vec<u8>>,
    rx: mpsc::Receiver<Vec<u8>>,
    rx_buf: VecDeque<u8>,
    image_rx: Option<mpsc::Receiver<Vec<u8>>>,
    image_buf: VecDeque<u8>,
}

impl InMemoryTransport {
    /// Creates a `(host_end, device_end)` pair sharing control and image pipes.
    pub fn pair(buffer_depth: usize) -> (Self, InMemoryDeviceEnd) {
        let (h2d_tx, h2d_rx) = mpsc::channel(buffer_depth);
        let (d2h_tx, d2h_rx) = mpsc::channel(buffer_depth);
        let (img_tx, img_rx) = mpsc::channel(buffer_depth);

        let host = Self {
            tx: h2d_tx,
            rx: d2h_rx,
            rx_buf: VecDeque::new(),
            image_rx: Some(img_rx),
            image_buf: VecDeque::new(),
        };
        let device = InMemoryDeviceEnd {
            rx: Arc::new(Mutex::new(h2d_rx)),
            tx: d2h_tx,
            image_tx: img_tx,
        };
        (host, device)
    }
}

pub struct InMemoryDeviceEnd {
    pub rx: Arc<Mutex<mpsc::Receiver<Vec<u8>>>>,
    pub tx: mpsc::Sender<Vec<u8>>,
    pub image_tx: mpsc::Sender<Vec<u8>>,
}

impl InMemoryDeviceEnd {
    pub async fn recv_from_host(&self) -> Option<Vec<u8>> {
        self.rx.lock().await.recv().await
    }

    pub async fn send_to_host(&self, data: Vec<u8>) -> Result<(), SekonicError> {
        self.tx
            .send(data)
            .await
            .map_err(|_| SekonicError::Transport("host channel closed".into()))
    }

    pub async fn send_image_to_host(&self, data: Vec<u8>) -> Result<(), SekonicError> {
        self.image_tx
            .send(data)
            .await
            .map_err(|_| SekonicError::Transport("host image channel closed".into()))
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ScannerTransport for InMemoryTransport {
    async fn write_control(&mut self, buf: &[u8], timeout: Duration) -> Result<usize, SekonicError> {
        let send_fut = self.tx.send(buf.to_vec());
        match tokio::time::timeout(timeout, send_fut).await {
            Ok(Ok(())) => Ok(buf.len()),
            Ok(Err(_)) => Err(SekonicError::Transport("device channel closed".into())),
            Err(_) => Err(SekonicError::Timeout {
                timeout_ms: timeout.as_millis() as u64,
            }),
        }
    }

    async fn read_control(
        &mut self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, SekonicError> {
        if self.rx_buf.is_empty() {
            match tokio::time::timeout(timeout, self.rx.recv()).await {
                Ok(Some(chunk)) => self.rx_buf.extend(chunk),
                Ok(None) => return Err(SekonicError::Transport("device channel closed".into())),
                Err(_) => {
                    return Err(SekonicError::Timeout {
                        timeout_ms: timeout.as_millis() as u64,
                    })
                }
            }
        }
        let n = buf.len().min(self.rx_buf.len());
        for slot in buf.iter_mut().take(n) {
            if let Some(b) = self.rx_buf.pop_front() {
                *slot = b;
            }
        }
        Ok(n)
    }

    async fn read_image_pipe(
        &mut self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, SekonicError> {
        if self.image_buf.is_empty() {
            let rx = self
                .image_rx
                .as_mut()
                .ok_or_else(|| SekonicError::Transport("no image pipe configured".into()))?;
            match tokio::time::timeout(timeout, rx.recv()).await {
                Ok(Some(chunk)) => self.image_buf.extend(chunk),
                Ok(None) => return Err(SekonicError::Transport("image pipe closed".into())),
                Err(_) => {
                    return Err(SekonicError::Timeout {
                        timeout_ms: timeout.as_millis() as u64,
                    })
                }
            }
        }
        let n = buf.len().min(self.image_buf.len());
        for slot in buf.iter_mut().take(n) {
            if let Some(b) = self.image_buf.pop_front() {
                *slot = b;
            }
        }
        Ok(n)
    }

    async fn purge(&mut self) -> Result<(), SekonicError> {
        self.rx_buf.clear();
        self.image_buf.clear();
        while self.rx.try_recv().is_ok() {}
        if let Some(rx) = self.image_rx.as_mut() {
            while rx.try_recv().is_ok() {}
        }
        Ok(())
    }
}
