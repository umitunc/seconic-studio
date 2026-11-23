mod tray_ui;
mod updater;

use futures_util::{SinkExt, StreamExt};
use sekonic_grader::field_decoder::{grade_sheet, GradedSheet};
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
use sekonic_transport::native_usbskomr::{discover_usbskomr_devices, LegacyUsbSkOmrTransport};
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

/// Windows 7 / 8 / 8.1 compatibility shim:
/// Rust 1.78+ `std` imports `bcryptprimitives.dll!ProcessPrng` (Windows 10+ only).
/// Providing a strong `__imp_` definition backed by `advapi32.dll!SystemFunction036` (`RtlGenRandom`)
/// eliminates the `bcryptprimitives.dll` import completely so the binary runs on Windows 7 / 8.1 / 10 / 11.
#[cfg(all(windows, target_env = "msvc"))]
mod win7_compat {
    use std::ffi::c_void;

    #[link(name = "advapi32")]
    extern "system" {
        fn SystemFunction036(buf: *mut c_void, len: u32) -> u8;
    }

    pub unsafe extern "system" fn fallback_process_prng(mut data: *mut u8, mut len: usize) -> i32 {
        while len > 0 {
            let chunk = len.min(0x7FFF_FFFF) as u32;
            if unsafe { SystemFunction036(data.cast(), chunk) } == 0 {
                return 0;
            }
            data = unsafe { data.add(chunk as usize) };
            len -= chunk as usize;
        }
        1
    }
}

#[cfg(all(windows, target_env = "msvc", target_arch = "x86"))]
std::arch::global_asm!(
    ".section .rdata,\"dr\"",
    ".globl __imp__ProcessPrng@8",
    ".p2align 2",
    "__imp__ProcessPrng@8:",
    ".long {fn_sym}",
    fn_sym = sym win7_compat::fallback_process_prng,
);

#[cfg(all(windows, target_env = "msvc", target_arch = "x86_64"))]
std::arch::global_asm!(
    ".section .rdata,\"dr\"",
    ".globl __imp_ProcessPrng",
    ".p2align 3",
    "__imp_ProcessPrng:",
    ".quad {fn_sym}",
    fn_sym = sym win7_compat::fallback_process_prng,
);

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
        #[serde(default)]
        enable_barcode: bool,
        #[serde(default)]
        schema: Option<OmrFormSchema>,
    },
    StopScan {
        req_id: Option<String>,
    },
    GetDiagnostics {
        req_id: Option<String>,
    },
    CheckUpdate {
        req_id: Option<String>,
        manifest_url: Option<String>,
    },
    UpdateAgent {
        req_id: Option<String>,
        download_url: Option<String>,
    },
}

fn default_baud_rate() -> u32 {
    38400
}

#[derive(Default, Clone, Serialize)]
struct AgentDiagSnapshot {
    model_name: String,
    rm_16level: bool,
    init_log: Vec<String>,
    last_error: Option<String>,
    total_scanned: u64,
    recent_sheets: Vec<GradedSheet>,
}

struct AgentState {
    scanning: AtomicBool,
    simulator_mode: AtomicBool,
    event_tx: broadcast::Sender<String>,
    diag: std::sync::Mutex<AgentDiagSnapshot>,
    tray: Arc<tray_ui::TraySharedState>,
    ui_cmd_tx: tokio::sync::mpsc::UnboundedSender<tray_ui::UiCommand>,
}

impl AgentState {
    fn new(
        tray: Arc<tray_ui::TraySharedState>,
        ui_cmd_tx: tokio::sync::mpsc::UnboundedSender<tray_ui::UiCommand>,
    ) -> Arc<Self> {
        let (event_tx, _) = broadcast::channel(256);
        Arc::new(Self {
            scanning: AtomicBool::new(false),
            simulator_mode: AtomicBool::new(false),
            event_tx,
            diag: std::sync::Mutex::new(AgentDiagSnapshot::default()),
            tray,
            ui_cmd_tx,
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
        for idx in discover_usbskomr_devices() {
            out.push(PortOption {
                id: format!("usbskomr:{idx}"),
                label: format!("USB: SEKONIC SR-2300/3500/6000/6500 (UsbSkOmr{idx})"),
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
        if value.get("type").and_then(|v| v.as_str()) == Some("scanner:status") {
            if let Some(scanning) = value.get("scanning").and_then(|v| v.as_bool()) {
                self.tray.scanning.store(scanning, Ordering::Relaxed);
                if !self.tray.updating.load(Ordering::Relaxed) {
                    if let Ok(mut st) = self.tray.status_line.lock() {
                        *st = if scanning {
                            "Tarama Aktif...".to_string()
                        } else {
                            format!("Hazır — ws://127.0.0.1:{}", self.tray.port)
                        };
                    }
                }
                self.tray.notify_refresh();
            }
        }
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
    enable_barcode: bool,
    state: Arc<AgentState>,
) {
    println!(
        "[sekonic-agent] Tarama baslatiliyor: form='{}' ({}x{}), barcode={}",
        schema.form_name, schema.grid.front_rows, schema.grid.front_cols, enable_barcode
    );
    state.broadcast_json(serde_json::json!({
        "type": "scanner:status",
        "phase": "initializing",
        "scanning": false
    }));
    if let Err(e) = driver.initialize(&schema, enable_barcode).await {
        eprintln!("[sekonic-agent] initialize hatasi: {e}");
        if let Ok(mut d) = state.diag.lock() {
            d.init_log = driver.init_log().to_vec();
            d.last_error = Some(e.to_string());
        }
        state.scanning.store(false, Ordering::Relaxed);
        state.broadcast_json(serde_json::json!({
            "type": "scanner:error",
            "message": e.to_string()
        }));
        state.broadcast_json(serde_json::json!({
            "type": "scanner:status",
            "phase": "idle",
            "scanning": false,
            "init_log": driver.init_log()
        }));
        return;
    }

    if let Ok(mut d) = state.diag.lock() {
        d.model_name = driver.model_name().to_string();
        d.rm_16level = driver.is_rm_16level();
        d.init_log = driver.init_log().to_vec();
        d.last_error = None;
        d.total_scanned = 0;
        d.recent_sheets.clear();
    }

    println!(
        "[sekonic-agent] Cihaz hazir (model='{}', rm16={}), init={:?}",
        driver.model_name(),
        driver.is_rm_16level(),
        driver.init_log()
    );

    state.broadcast_json(serde_json::json!({
        "type": "scanner:status",
        "phase": "scanning",
        "scanning": true,
        "model_name": driver.model_name(),
        "rm_16level": driver.is_rm_16level(),
        "init_log": driver.init_log()
    }));

    while state.scanning.load(Ordering::Relaxed) {
        match driver.feed_and_read_sheet(&schema, enable_barcode).await {
            Ok(Some(cap)) => {
                let graded = grade_sheet(&schema, &cap);
                if let Ok(mut d) = state.diag.lock() {
                    d.total_scanned += 1;
                    if d.recent_sheets.len() < 3 {
                        d.recent_sheets.push(graded.clone());
                    }
                }
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
                println!("[sekonic-agent] Hopper bos, tarama tamamlandi.");
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
                eprintln!("[sekonic-agent] Okuma hatasi: {e}");
                if let Ok(mut d) = state.diag.lock() {
                    d.last_error = Some(e.to_string());
                }
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
    driver.finish_scan().await;
}

fn start_scan(
    state: &Arc<AgentState>,
    port_id: &str,
    baud_rate: u32,
    simulator_mode: bool,
    enable_barcode: bool,
    custom_schema: Option<OmrFormSchema>,
) -> Result<(), String> {
    if state.scanning.load(Ordering::Relaxed) {
        return Err("Tarama zaten aktif.".into());
    }

    let schema = match custom_schema {
        Some(s) => {
            s.validate().map_err(|e| format!("Geçersiz form şeması: {e}"))?;
            s
        }
        None => default_runtime_schema(),
    };

    state.simulator_mode.store(simulator_mode, Ordering::Relaxed);
    state.scanning.store(true, Ordering::Relaxed);

    let st = state.clone();

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
        tokio::spawn(run_driver_loop(driver, schema, enable_barcode, st));
        return Ok(());
    }

    if let Some(com_port) = port_id.strip_prefix("serial:") {
        let transport = NativeSerialTransport::open(com_port, baud_rate).map_err(|e| {
            state.scanning.store(false, Ordering::Relaxed);
            format!("Seri port ({com_port}) açılamadı: {e}")
        })?;
        let driver = OmrScannerDriver::new(transport);
        tokio::spawn(run_driver_loop(driver, schema, enable_barcode, st));
        return Ok(());
    }

    if let Some(idx_str) = port_id.strip_prefix("usbskomr:") {
        let idx = idx_str.parse::<u8>().map_err(|e| {
            state.scanning.store(false, Ordering::Relaxed);
            format!("Geçersiz UsbSkOmr indeksi: {e}")
        })?;
        let transport = LegacyUsbSkOmrTransport::open(idx).map_err(|e| {
            state.scanning.store(false, Ordering::Relaxed);
            format!(
                "USB aygıtı (UsbSkOmr{idx}) açılamadı (SekOku açıksa kapatıp tekrar deneyin): {e}"
            )
        })?;
        let driver = OmrScannerDriver::new(transport);
        tokio::spawn(run_driver_loop(driver, schema, enable_barcode, st));
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
        tokio::spawn(run_driver_loop(driver, schema, enable_barcode, st));
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
                            Ok(ClientCommand::StartScan {
                                req_id,
                                port_id,
                                baud_rate,
                                simulator_mode,
                                enable_barcode,
                                schema,
                            }) => {
                                match start_scan(&state, &port_id, baud_rate, simulator_mode, enable_barcode, schema) {
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
                            Ok(ClientCommand::GetDiagnostics { req_id }) => {
                                let snap = state.diag.lock().map(|d| d.clone()).unwrap_or_default();
                                serde_json::json!({
                                    "type": "diagnostics",
                                    "req_id": req_id,
                                    "snapshot": snap,
                                })
                            }
                            Ok(ClientCommand::CheckUpdate { req_id, manifest_url }) => {
                                match tokio::task::spawn_blocking(move || {
                                    updater::check_for_update_blocking(manifest_url.as_deref())
                                })
                                .await
                                {
                                    Ok(Ok(res)) => {
                                        apply_update_check_to_tray(&state, &res);
                                        serde_json::json!({
                                            "type": "update_status",
                                            "req_id": req_id,
                                            "current_version": res.current_version,
                                            "latest_version": res.latest_version,
                                            "download_url": res.download_url,
                                            "update_available": res.update_available,
                                        })
                                    }
                                    Ok(Err(err)) => serde_json::json!({
                                        "type": "error",
                                        "req_id": req_id,
                                        "message": err,
                                    }),
                                    Err(err) => serde_json::json!({
                                        "type": "error",
                                        "req_id": req_id,
                                        "message": format!("Güncelleme kontrol görevi hatası: {err}"),
                                    }),
                                }
                            }
                            Ok(ClientCommand::UpdateAgent { req_id, download_url }) => {
                                if state.tray.updating.load(Ordering::Relaxed) {
                                    serde_json::json!({
                                        "type": "update_started",
                                        "req_id": req_id,
                                        "ok": true,
                                    })
                                } else {
                                    let _ = state.ui_cmd_tx.send(tray_ui::UiCommand::PerformUpdate { download_url });
                                    serde_json::json!({
                                        "type": "update_started",
                                        "req_id": req_id,
                                        "ok": true,
                                    })
                                }
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

fn apply_update_check_to_tray(state: &Arc<AgentState>, res: &updater::UpdateCheckResult) {
    state.tray.update_available.store(res.update_available, Ordering::Relaxed);
    if let Ok(mut lv) = state.tray.latest_version.lock() {
        *lv = Some(res.latest_version.clone());
    }
    if let Ok(mut dl) = state.tray.download_url.lock() {
        *dl = Some(res.download_url.clone());
    }
    if let Ok(mut line) = state.tray.update_line.lock() {
        *line = if res.update_available {
            format!("Yeni sürüm mevcut: v{} (mevcut: v{})", res.latest_version, res.current_version)
        } else {
            format!("Güncel (v{})", res.current_version)
        };
    }
    state.tray.notify_refresh();
}

async fn run_update_flow(state: Arc<AgentState>, download_url: Option<String>) {
    if state.tray.updating.swap(true, Ordering::AcqRel) {
        return;
    }
    let dl_url = download_url
        .or_else(|| state.tray.download_url.lock().ok().and_then(|g| g.clone()))
        .unwrap_or_else(|| updater::DEFAULT_DOWNLOAD_URL.to_string());

    if let Ok(mut st) = state.tray.status_line.lock() {
        *st = "Güncelleme indiriliyor ve kuruluyor...".to_string();
    }
    state.tray.notify_refresh();
    state.tray.show_balloon(
        "Sekonic Ajan Güncelleniyor",
        "Yeni sürüm indiriliyor, mevcut ajan kapatılıp güncellenecek...",
    );
    state.broadcast_json(serde_json::json!({
        "type": "agent:updating",
        "download_url": dl_url,
    }));

    let res = tokio::task::spawn_blocking(move || updater::download_and_spawn_updater(&dl_url)).await;
    match res {
        Ok(Ok(_staged)) => {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            state.tray.request_exit();
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            std::process::exit(0);
        }
        Ok(Err(err)) => {
            state.tray.updating.store(false, Ordering::Relaxed);
            if let Ok(mut st) = state.tray.status_line.lock() {
                *st = format!("Hazır — ws://127.0.0.1:{}", state.tray.port);
            }
            if let Ok(mut ul) = state.tray.update_line.lock() {
                *ul = format!("Güncelleme hatası: {err}");
            }
            state.tray.notify_refresh();
            state.tray.show_balloon("Güncelleme Hatası", &err);
            state.broadcast_json(serde_json::json!({
                "type": "agent:update_error",
                "message": err,
            }));
        }
        Err(err) => {
            state.tray.updating.store(false, Ordering::Relaxed);
            state.tray.notify_refresh();
            state.broadcast_json(serde_json::json!({
                "type": "agent:update_error",
                "message": format!("Güncelleme görevi başarısız: {err}"),
            }));
        }
    }
}

async fn run_check_flow(state: Arc<AgentState>, manual: bool) {
    let res = tokio::task::spawn_blocking(|| updater::check_for_update_blocking(None)).await;
    match res {
        Ok(Ok(info)) => {
            apply_update_check_to_tray(&state, &info);
            if info.update_available {
                state.broadcast_json(serde_json::json!({
                    "type": "agent:update_available",
                    "current_version": info.current_version,
                    "latest_version": info.latest_version,
                    "download_url": info.download_url,
                }));
                let auto = state.tray.auto_update.load(Ordering::Relaxed);
                let busy = state.scanning.load(Ordering::Relaxed);
                if auto && !busy {
                    run_update_flow(state, Some(info.download_url)).await;
                } else {
                    state.tray.show_balloon(
                        "Yeni Sürüm Mevcut",
                        &format!("Sekonic Ajan v{} hazır. Güncellemek için tıklayın.", info.latest_version),
                    );
                }
            } else if manual {
                state.tray.show_balloon(
                    "Sekonic Ajan Güncel",
                    &format!("En son sürüm (v{}) kullanılıyor.", info.current_version),
                );
            }
        }
        Ok(Err(err)) => {
            if let Ok(mut ul) = state.tray.update_line.lock() {
                *ul = format!("Sürüm kontrolü yapılamadı (v{})", state.tray.current_version);
            }
            state.tray.notify_refresh();
            if manual {
                state.tray.show_balloon("Güncelleme Kontrolü", &err);
            }
        }
        Err(_) => {}
    }
}

async fn run_agent_runtime(
    port: u16,
    tray: Arc<tray_ui::TraySharedState>,
    ui_cmd_tx: tokio::sync::mpsc::UnboundedSender<tray_ui::UiCommand>,
    mut ui_cmd_rx: tokio::sync::mpsc::UnboundedReceiver<tray_ui::UiCommand>,
) {
    let bind_addr = format!("127.0.0.1:{port}");
    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            if let Ok(mut st) = tray.status_line.lock() {
                *st = format!("Port hatası ({bind_addr}): {e}");
            }
            tray.notify_refresh();
            tray.show_balloon(
                "Bağlantı Hatası",
                &format!("{bind_addr} dinlenemedi (başka bir örnek çalışıyor olabilir)."),
            );
            return;
        }
    };

    let state = AgentState::new(tray.clone(), ui_cmd_tx);

    // Periodic auto-update check (startup + every 30 min)
    let update_state = state.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        run_check_flow(update_state.clone(), false).await;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1800));
        interval.tick().await; // skip immediate tick
        loop {
            interval.tick().await;
            run_check_flow(update_state.clone(), false).await;
        }
    });

    loop {
        tokio::select! {
            accept_res = listener.accept() => {
                if let Ok((stream, _peer)) = accept_res {
                    let st = state.clone();
                    tokio::spawn(handle_client(stream, st));
                }
            }
            Some(cmd) = ui_cmd_rx.recv() => {
                match cmd {
                    tray_ui::UiCommand::CheckUpdate { manual } => {
                        let st = state.clone();
                        tokio::spawn(async move { run_check_flow(st, manual).await; });
                    }
                    tray_ui::UiCommand::PerformUpdate { download_url } => {
                        let st = state.clone();
                        tokio::spawn(async move { run_update_flow(st, download_url).await; });
                    }
                    tray_ui::UiCommand::ToggleAutoUpdate(enabled) => {
                        let mut cfg = updater::load_config();
                        cfg.auto_update = enabled;
                        updater::save_config(&cfg);
                    }
                    tray_ui::UiCommand::Shutdown => {
                        break;
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tray.request_exit();
                break;
            }
        }
    }
}

fn main() {
    if tray_ui::try_focus_existing_instance() {
        return;
    }

    let port: u16 = std::env::var("SEKONIC_AGENT_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let cfg = updater::load_config();
    let tray = tray_ui::TraySharedState::new(port, cfg.auto_update);
    let (ui_cmd_tx, ui_cmd_rx) = tokio::sync::mpsc::unbounded_channel();

    let runtime_tray = tray.clone();
    let runtime_tx = ui_cmd_tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Tokio runtime failed");
        rt.block_on(run_agent_runtime(port, runtime_tray, runtime_tx, ui_cmd_rx));
    });

    tray_ui::run_tray_ui_blocking(tray, ui_cmd_tx);
    std::process::exit(0);
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

