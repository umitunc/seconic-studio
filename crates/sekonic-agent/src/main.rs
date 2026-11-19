use futures_util::{SinkExt, StreamExt};
use sekonic_grader::field_decoder::grade_sheet;
use sekonic_proto_omr::commands::StackerTarget;
use sekonic_proto_omr::driver::OmrScannerDriver;
use sekonic_schema::form_schema::{
    BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, GridGeometry,
    OmrFormSchema, SheetSide, ThresholdPolicy,
};
use sekonic_sim::paper_generator::{generate_synthetic_sheet, SheetScenario};
use sekonic_sim::virtual_sr6500::VirtualSr6500;
use sekonic_transport::memory_channel::InMemoryTransport;
use sekonic_transport::native_serial::{list_serial_ports, NativeSerialTransport};
use sekonic_transport::native_usb::{discover_usb_scanners, NativeUsbTransport};
use sekonic_transport::traits::ScannerTransport;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tokio_tungstenite::tungstenite::Message;

const DEFAULT_PORT: u16 = 17441;

#[derive(Debug, Clone, Serialize)]
pub struct PortOption {
    pub id: String,
    pub label: String,
    pub kind: String, // "serial" | "usb"
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientCommand {
    Ping {
        req_id: Option<String>,
    },
    ListPorts {
        req_id: Option<String>,
    },
    StartScan {
        req_id: Option<String>,
        port_id: String,
        #[serde(default = "default_baud_rate")]
        baud_rate: u32,
        #[serde(default)]
        simulator_mode: bool,
    },
    StopScan {
        req_id: Option<String>,
    },
}

fn default_baud_rate() -> u32 {
    38400
}

struct AgentState {
    scanning: AtomicBool,
    simulator_mode: AtomicBool,
    event_tx: broadcast::Sender<String>,
}

impl AgentState {
    fn new() -> Arc<Self> {
        let (event_tx, _) = broadcast::channel(256);
        Arc::new(Self {
            scanning: AtomicBool::new(false),
            simulator_mode: AtomicBool::new(false),
            event_tx,
        })
    }

    fn list_ports(&self) -> Vec<PortOption> {
        let mut out = Vec::new();
        for dev in discover_usb_scanners().unwrap_or_default() {
            out.push(PortOption {
                id: format!("usb:{:04x}", dev.product_id),
                label: format!("USB: {} (PID 0x{:04X})", dev.model_hint, dev.product_id),
                kind: "usb".into(),
            });
        }
        for com in list_serial_ports() {
            out.push(PortOption {
                id: format!("serial:{com}"),
                label: format!("COM / Seri Port: {com}"),
                kind: "serial".into(),
            });
        }
        out
    }

    fn broadcast_json(&self, value: serde_json::Value) {
        if let Ok(text) = serde_json::to_string(&value) {
            let _ = self.event_tx.send(text);
        }
    }
}

fn is_allowed_origin(origin: &str) -> bool {
    origin == "https://sekonic.dijios.com"
        || origin.starts_with("https://sekonic.dijios.com:")
        || origin == "http://localhost"
        || origin.starts_with("http://localhost:")
        || origin == "https://localhost"
        || origin.starts_with("https://localhost:")
        || origin == "http://127.0.0.1"
        || origin.starts_with("http://127.0.0.1:")
        || origin == "https://127.0.0.1"
        || origin.starts_with("https://127.0.0.1:")
        || origin.starts_with("tauri://")
}

fn default_runtime_schema() -> OmrFormSchema {
    OmrFormSchema {
        schema_version: 1,
        form_id: "RUNTIME-DEFAULT".into(),
        form_name: "Aktif Sınav Şeması".into(),
        duplex: false,
        dropout_color: DropoutColor::Red,
        grid: GridGeometry {
            front_rows: 24,
            front_cols: 10,
            back_rows: 0,
            back_cols: 0,
            width_mm: 210.0,
            height_mm: 297.0,
        },
        threshold_policy: ThresholdPolicy::default(),
        fields: vec![
            FieldDefinition {
                key: "student_id".into(),
                field_type: FieldType::StudentId,
                side: SheetSide::Front,
                required_for_valid_sheet: true,
                items: vec![BubbleGroup {
                    index: 1,
                    options: (0..5)
                        .map(|c| BubbleCoord {
                            label: c.to_string(),
                            row: 0,
                            col: c,
                        })
                        .collect(),
                }],
            },
            FieldDefinition {
                key: "matematik".into(),
                field_type: FieldType::SubjectQuestions,
                side: SheetSide::Front,
                required_for_valid_sheet: false,
                items: (0..10u16)
                    .map(|q| BubbleGroup {
                        index: q + 1,
                        options: (0..5u16)
                            .map(|c| BubbleCoord {
                                label: ((b'A' + c as u8) as char).to_string(),
                                row: q + 2,
                                col: c,
                            })
                            .collect(),
                    })
                    .collect(),
            },
        ],
    }
}

async fn run_driver_loop<T: ScannerTransport + 'static>(
    mut driver: OmrScannerDriver<T>,
    schema: OmrFormSchema,
    state: Arc<AgentState>,
) {
    if let Err(e) = driver.initialize(&schema, true).await {
        state.scanning.store(false, Ordering::Relaxed);
        state.broadcast_json(serde_json::json!({
            "type": "scanner:error",
            "message": e.to_string()
        }));
        state.broadcast_json(serde_json::json!({
            "type": "scanner:status",
            "scanning": false
        }));
        return;
    }

    state.broadcast_json(serde_json::json!({
        "type": "scanner:status",
        "scanning": true
    }));

    while state.scanning.load(Ordering::Relaxed) {
        match driver.feed_and_read_sheet(&schema, true).await {
            Ok(Some(cap)) => {
                let graded = grade_sheet(&schema, &cap);
                let target = if graded.missing_required_field {
                    StackerTarget::Sub
                } else {
                    StackerTarget::Main
                };
                let _ = driver.eject_sheet(target).await;
                state.broadcast_json(serde_json::json!({
                    "type": "sheet:scanned",
                    "payload": graded
                }));
            }
            Ok(None) => {
                state.scanning.store(false, Ordering::Relaxed);
                state.broadcast_json(serde_json::json!({
                    "type": "scanner:hopper_empty"
                }));
                state.broadcast_json(serde_json::json!({
                    "type": "scanner:status",
                    "scanning": false
                }));
                break;
            }
            Err(e) => {
                state.scanning.store(false, Ordering::Relaxed);
                state.broadcast_json(serde_json::json!({
                    "type": "scanner:error",
                    "message": e.to_string()
                }));
                state.broadcast_json(serde_json::json!({
                    "type": "scanner:status",
                    "scanning": false
                }));
                break;
            }
        }
    }
}

fn start_scan(
    state: &Arc<AgentState>,
    port_id: &str,
    baud_rate: u32,
    simulator_mode: bool,
) -> Result<(), String> {
    if state.scanning.load(Ordering::Relaxed) {
        return Err("Tarama zaten aktif.".into());
    }

    state.simulator_mode.store(simulator_mode, Ordering::Relaxed);
    state.scanning.store(true, Ordering::Relaxed);

    let st = state.clone();
    let schema = default_runtime_schema();

    if simulator_mode {
        let sheets: Vec<_> = (1..=50u64)
            .map(|i| {
                let scen = if i % 10 == 0 {
                    SheetScenario::InvalidStudentId
                } else if i % 4 == 1 {
                    SheetScenario::WithErasures
                } else {
                    SheetScenario::Clean
                };
                generate_synthetic_sheet(&schema, i, scen, 4242)
            })
            .collect();
        let (host_tx, dev_rx) = InMemoryTransport::pair(16);
        tokio::spawn(VirtualSr6500::new(sheets).run(dev_rx));
        let driver = OmrScannerDriver::new(host_tx);
        tokio::spawn(run_driver_loop(driver, schema, st));
        return Ok(());
    }

    if let Some(com_port) = port_id.strip_prefix("serial:") {
        let transport = NativeSerialTransport::open(com_port, baud_rate).map_err(|e| {
            state.scanning.store(false, Ordering::Relaxed);
            format!("Seri port ({com_port}) açılamadı: {e}")
        })?;
        let driver = OmrScannerDriver::new(transport);
        tokio::spawn(run_driver_loop(driver, schema, st));
        return Ok(());
    }

    if let Some(pid_hex) = port_id.strip_prefix("usb:") {
        let pid = u16::from_str_radix(pid_hex, 16).map_err(|e| {
            state.scanning.store(false, Ordering::Relaxed);
            format!("Geçersiz USB PID: {e}")
        })?;
        let transport = NativeUsbTransport::open_by_pid(pid).map_err(|e| {
            state.scanning.store(false, Ordering::Relaxed);
            format!("USB aygıtı (0x{pid:04X}) açılamadı: {e}")
        })?;
        let driver = OmrScannerDriver::new(transport);
        tokio::spawn(run_driver_loop(driver, schema, st));
        return Ok(());
    }

    state.scanning.store(false, Ordering::Relaxed);
    Err("Geçerli bir COM veya USB aygıtı seçilmedi.".into())
}

async fn handle_client(stream: tokio::net::TcpStream, state: Arc<AgentState>) {
    let callback = |req: &Request, resp: Response| -> Result<Response, ErrorResponse> {
        if let Some(origin) = req.headers().get("Origin").and_then(|v| v.to_str().ok()) {
            if !is_allowed_origin(origin) {
                eprintln!("[sekonic-agent] Rejected WebSocket Origin: {origin}");
                return Err(Response::builder()
                    .status(403)
                    .body(Some("Forbidden Origin".to_string()))
                    .unwrap_or_default());
            }
        }
        Ok(resp)
    };

    let Ok(ws_stream) = accept_hdr_async(stream, callback).await else {
        return;
    };

    let (mut ws_tx, mut ws_rx) = ws_stream.split();
    let mut broadcast_rx = state.event_tx.subscribe();

    // Send initial hello handshake
    let hello = serde_json::json!({
        "type": "hello",
        "version": env!("CARGO_PKG_VERSION"),
        "platform": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "scanning": state.scanning.load(Ordering::Relaxed),
    });
    if ws_tx.send(Message::Text(hello.to_string().into())).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            bcast = broadcast_rx.recv() => {
                match bcast {
                    Ok(payload) => {
                        if ws_tx.send(Message::Text(payload.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            incoming = ws_rx.next() => {
                let Some(Ok(msg)) = incoming else {
                    break;
                };
                match msg {
                    Message::Text(text) => {
                        let response = match serde_json::from_str::<ClientCommand>(&text) {
                            Ok(ClientCommand::Ping { req_id }) => serde_json::json!({
                                "type": "pong",
                                "req_id": req_id,
                                "scanning": state.scanning.load(Ordering::Relaxed),
                            }),
                            Ok(ClientCommand::ListPorts { req_id }) => {
                                let ports = state.list_ports();
                                serde_json::json!({
                                    "type": "ports_list",
                                    "req_id": req_id,
                                    "ports": ports,
                                })
                            }
                            Ok(ClientCommand::StartScan { req_id, port_id, baud_rate, simulator_mode }) => {
                                match start_scan(&state, &port_id, baud_rate, simulator_mode) {
                                    Ok(()) => serde_json::json!({
                                        "type": "scan_started",
                                        "req_id": req_id,
                                        "ok": true,
                                    }),
                                    Err(err) => serde_json::json!({
                                        "type": "error",
                                        "req_id": req_id,
                                        "message": err,
                                    }),
                                }
                            }
                            Ok(ClientCommand::StopScan { req_id }) => {
                                state.scanning.store(false, Ordering::Relaxed);
                                state.broadcast_json(serde_json::json!({
                                    "type": "scanner:status",
                                    "scanning": false
                                }));
                                serde_json::json!({
                                    "type": "scan_stopped",
                                    "req_id": req_id,
                                    "ok": true,
                                })
                            }
                            Err(err) => serde_json::json!({
                                "type": "error",
                                "message": format!("Geçersiz komut: {err}"),
                            }),
                        };
                        if ws_tx.send(Message::Text(response.to_string().into())).await.is_err() {
                            break;
                        }
                    }
                    Message::Ping(data) => {
                        if ws_tx.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("SEKONIC_AGENT_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let bind_addr = format!("127.0.0.1:{port}");
    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[sekonic-agent] {bind_addr} dinlenemedi (başka bir örnek çalışıyor olabilir): {e}");
            std::process::exit(1);
        }
    };

    println!(
        "Sekonic Local Bridge Agent v{} hazir -> ws://{bind_addr}",
        env!("CARGO_PKG_VERSION")
    );
    println!("Web arayuzunden (https://sekonic.dijios.com) otomatik baglanti bekleniyor...");

    let state = AgentState::new();

    loop {
        tokio::select! {
            accept_res = listener.accept() => {
                if let Ok((stream, _peer)) = accept_res {
                    let st = state.clone();
                    tokio::spawn(handle_client(stream, st));
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("\n[sekonic-agent] Kapatiliyor...");
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_allowlist_checks() {
        assert!(is_allowed_origin("https://sekonic.dijios.com"));
        assert!(is_allowed_origin("http://localhost:5173"));
        assert!(is_allowed_origin("http://127.0.0.1:1420"));
        assert!(is_allowed_origin("tauri://localhost"));
        assert!(!is_allowed_origin("https://evil.example.com"));
        assert!(!is_allowed_origin("https://sekonic.dijios.com.evil.com"));
    }

    #[test]
    fn command_deserialization() {
        let cmd: ClientCommand =
            serde_json::from_str(r#"{"type":"list_ports","req_id":"r1"}"#).unwrap();
        assert!(matches!(cmd, ClientCommand::ListPorts { req_id: Some(ref id) } if id == "r1"));

        let start: ClientCommand = serde_json::from_str(
            r#"{"type":"start_scan","port_id":"serial:COM3","baud_rate":38400,"simulator_mode":false}"#,
        )
        .unwrap();
        assert!(matches!(
            start,
            ClientCommand::StartScan {
                baud_rate: 38400,
                simulator_mode: false,
                ..
            }
        ));
    }
}

