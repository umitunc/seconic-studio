use futures_util::SinkExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::Message;

/// Starts the local WebSocket bridge on `127.0.0.1:17441` with Origin allow-list checks.
pub async fn run_local_ws_bridge(mut tx_rx: broadcast::Receiver<String>) {
    let Ok(listener) = TcpListener::bind("127.0.0.1:17441").await else {
        return;
    };

    loop {
        let Ok((stream, _)) = listener.accept().await else {
            continue;
        };
        let mut client_rx = tx_rx.resubscribe();
        tokio::spawn(async move {
            let callback = |req: &Request, resp: Response| -> Result<Response, ErrorResponse> {
                if let Some(origin) = req.headers().get("Origin").and_then(|v| v.to_str().ok()) {
                    let allowed = origin == "https://sekonic.dijios.com"
                        || origin.starts_with("https://sekonic.dijios.com:")
                        || origin == "http://localhost"
                        || origin.starts_with("http://localhost:")
                        || origin == "https://localhost"
                        || origin.starts_with("https://localhost:")
                        || origin == "http://127.0.0.1"
                        || origin.starts_with("http://127.0.0.1:")
                        || origin == "https://127.0.0.1"
                        || origin.starts_with("https://127.0.0.1:")
                        || origin.starts_with("tauri://");
                    if !allowed {
                        return Err(Response::builder()
                            .status(403)
                            .body(Some("Forbidden Origin".to_string()))
                            .unwrap_or_default());
                    }
                }
                Ok(resp)
            };

            if let Ok(mut ws) = accept_hdr_async(stream, callback).await {
                while let Ok(payload) = client_rx.recv().await {
                    if ws.send(Message::Text(payload.into())).await.is_err() {
                        break;
                    }
                }
            }
        });
    }
}
