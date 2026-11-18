#![forbid(unsafe_code)]

//! SQLite WAL offline outbox queue and WebSocket/REST sync client.

pub mod wal_queue;
pub mod ws_client;

#[cfg(test)]
mod tests {
    use super::wal_queue::WalOutboxQueue;
    use super::ws_client::flush_outbox_once;
    use futures_util::StreamExt;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::sync::Mutex;
    use tokio_tungstenite::accept_async;

    #[tokio::test]
    async fn test_offline_queue_100_sheets_reconnect_sync() {
        let dir = std::env::temp_dir().join("sekonic_sync_test.db");
        let _ = std::fs::remove_file(&dir);
        let queue = WalOutboxQueue::open(&dir).expect("open wal");

        // Enqueue first 50 sheets
        for i in 1..=50u64 {
            queue
                .enqueue("ses_1", i, &format!("{{\"seq\":{i}}}"), 1000 + i)
                .expect("enqueue");
        }

        // Start mock WS server #1 for first 50
        let received = Arc::new(Mutex::new(Vec::<String>::new()));
        let listener1 = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr1 = listener1.local_addr().expect("addr");
        let rec1 = received.clone();
        let srv1 = tokio::spawn(async move {
            if let Ok((stream, _)) = listener1.accept().await {
                if let Ok(mut ws) = accept_async(stream).await {
                    while let Some(Ok(msg)) = ws.next().await {
                        if msg.is_text() {
                            rec1.lock().await.push(msg.to_string());
                        }
                    }
                }
            }
        });

        let flushed1 = flush_outbox_once(&queue, &format!("ws://{addr1}"), 100)
            .await
            .expect("flush1");
        assert_eq!(flushed1, 50);
        let _ = srv1.await;

        // Simulate network outage: enqueue 51..100 while offline
        for i in 51..=100u64 {
            queue
                .enqueue("ses_1", i, &format!("{{\"seq\":{i}}}"), 1000 + i)
                .expect("enqueue offline");
        }
        assert!(flush_outbox_once(&queue, "ws://127.0.0.1:1", 100).await.is_err());
        assert_eq!(queue.stats().expect("stats").pending, 50);

        // Network returns: start mock WS server #2
        let listener2 = TcpListener::bind("127.0.0.1:0").await.expect("bind2");
        let addr2 = listener2.local_addr().expect("addr2");
        let rec2 = received.clone();
        let srv2 = tokio::spawn(async move {
            if let Ok((stream, _)) = listener2.accept().await {
                if let Ok(mut ws) = accept_async(stream).await {
                    while let Some(Ok(msg)) = ws.next().await {
                        if msg.is_text() {
                            rec2.lock().await.push(msg.to_string());
                        }
                    }
                }
            }
        });

        let flushed2 = flush_outbox_once(&queue, &format!("ws://{addr2}"), 100)
            .await
            .expect("flush2");
        assert_eq!(flushed2, 50);
        let _ = srv2.await;

        let stats = queue.stats().expect("stats");
        assert_eq!(stats.pending, 0);
        assert_eq!(stats.synced, 100);
        assert_eq!(received.lock().await.len(), 100);
        let _ = std::fs::remove_file(&dir);
    }
}
