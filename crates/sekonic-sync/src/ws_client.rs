use crate::wal_queue::WalOutboxQueue;
use futures_util::SinkExt;
use sekonic_schema::error::SekonicError;
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// Flushes pending outbox items over WebSocket to the assessment backend.
/// Returns the number of items successfully flushed in this pass.
pub async fn flush_outbox_once(
    queue: &WalOutboxQueue,
    ws_url: &str,
    batch_size: usize,
) -> Result<usize, SekonicError> {
    let pending = queue.fetch_pending(batch_size)?;
    if pending.is_empty() {
        return Ok(0);
    }

    let (mut ws, _) = tokio::time::timeout(Duration::from_secs(3), connect_async(ws_url))
        .await
        .map_err(|_| SekonicError::Timeout { timeout_ms: 3000 })?
        .map_err(|e| SekonicError::Sync(format!("ws connect: {e}")))?;

    let mut flushed = 0usize;
    for item in pending {
        ws.send(Message::Text(item.payload_json.into()))
            .await
            .map_err(|e| SekonicError::Sync(format!("ws send: {e}")))?;
        queue.mark_synced(item.id)?;
        flushed += 1;
    }
    let _ = ws.close(None).await;
    Ok(flushed)
}
