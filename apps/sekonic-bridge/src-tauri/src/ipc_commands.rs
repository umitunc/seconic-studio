use crate::scanner_manager::ScannerManagerState;
use sekonic_grader::field_decoder::grade_sheet;
use sekonic_proto_omr::commands::StackerTarget;
use sekonic_proto_omr::driver::OmrScannerDriver;
use sekonic_schema::form_schema::{
    BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, GridGeometry,
    OmrFormSchema, SheetSide, ThresholdPolicy,
};
use sekonic_schema::legacy_bldm::parse_bldm;
use sekonic_sim::paper_generator::{generate_synthetic_sheet, SheetScenario};
use sekonic_sim::virtual_sr6500::VirtualSr6500;
use sekonic_transport::memory_channel::InMemoryTransport;
use sekonic_transport::native_serial::{list_serial_ports, NativeSerialTransport};
use sekonic_transport::native_usb::NativeUsbTransport;
use sekonic_transport::traits::ScannerTransport;
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize)]
pub struct PortOption {
    pub id: String,
    pub label: String,
    pub kind: String, // "serial" | "usb"
}

#[derive(Serialize)]
pub struct BridgeStatus {
    pub simulator_mode: bool,
    pub scanning: bool,
    pub usb_device_count: usize,
}

#[tauri::command]
pub fn list_available_ports(state: State<'_, Arc<ScannerManagerState>>) -> Vec<PortOption> {
    let mut out = Vec::new();
    for dev in state.list_devices() {
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

#[tauri::command]
pub fn get_bridge_status(state: State<'_, Arc<ScannerManagerState>>) -> BridgeStatus {
    BridgeStatus {
        simulator_mode: state.is_simulator_mode(),
        scanning: state.scanning.load(Ordering::Relaxed),
        usb_device_count: state.list_devices().len(),
    }
}

#[tauri::command]
pub fn toggle_simulator_mode(state: State<'_, Arc<ScannerManagerState>>, enabled: bool) -> bool {
    state.set_simulator_mode(enabled);
    enabled
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
    state: Arc<ScannerManagerState>,
    app: AppHandle,
) {
    if let Err(e) = driver.initialize(&schema, true).await {
        let _ = app.emit("scanner:error", e.to_string());
        state.scanning.store(false, Ordering::Relaxed);
        return;
    }

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
                if let Ok(json) = serde_json::to_string(&graded) {
                    let _ = state.event_tx.send(json.clone());
                    let _ = app.emit("sheet:scanned", json);
                }
            }
            Ok(None) => {
                state.scanning.store(false, Ordering::Relaxed);
                let _ = app.emit("scanner:hopper_empty", true);
                break;
            }
            Err(e) => {
                state.scanning.store(false, Ordering::Relaxed);
                let _ = app.emit("scanner:error", e.to_string());
                break;
            }
        }
    }
}

#[tauri::command]
pub fn start_scan(
    app: AppHandle,
    state: State<'_, Arc<ScannerManagerState>>,
    port_id: String,
    baud_rate: u32,
    simulator_mode: bool,
) -> Result<bool, String> {
    state.set_simulator_mode(simulator_mode);
    state.scanning.store(true, Ordering::Relaxed);

    let st = state.inner().clone();
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
        tauri::async_runtime::spawn(VirtualSr6500::new(sheets).run(dev_rx));
        let driver = OmrScannerDriver::new(host_tx);
        tauri::async_runtime::spawn(run_driver_loop(driver, schema, st, app));
        return Ok(true);
    }

    if let Some(com_port) = port_id.strip_prefix("serial:") {
        let transport = NativeSerialTransport::open(com_port, baud_rate).map_err(|e| {
            state.scanning.store(false, Ordering::Relaxed);
            e.to_string()
        })?;
        let driver = OmrScannerDriver::new(transport);
        tauri::async_runtime::spawn(run_driver_loop(driver, schema, st, app));
        return Ok(true);
    }

    if let Some(pid_hex) = port_id.strip_prefix("usb:") {
        let pid = u16::from_str_radix(pid_hex, 16).map_err(|e| e.to_string())?;
        let transport = NativeUsbTransport::open_by_pid(pid).map_err(|e| {
            state.scanning.store(false, Ordering::Relaxed);
            e.to_string()
        })?;
        let driver = OmrScannerDriver::new(transport);
        tauri::async_runtime::spawn(run_driver_loop(driver, schema, st, app));
        return Ok(true);
    }

    state.scanning.store(false, Ordering::Relaxed);
    Err("Geçerli bir COM veya USB aygıtı seçilmedi.".into())
}

#[tauri::command]
pub fn stop_scan(state: State<'_, Arc<ScannerManagerState>>) -> bool {
    state.scanning.store(false, Ordering::Relaxed);
    false
}

#[tauri::command]
pub fn convert_legacy_bldm(form_id: String, content: String) -> Result<OmrFormSchema, String> {
    parse_bldm(&form_id, &content).map_err(|e| e.to_string())
}
