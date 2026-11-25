use futures_util::StreamExt;
use sekonic_grader::analog_solver::BubbleState;
use sekonic_grader::field_decoder::grade_sheet;
use sekonic_grader::roster_validator::{RosterValidator, StackerDecision};
use sekonic_proto_omr::commands::StackerTarget;
use sekonic_proto_omr::driver::OmrScannerDriver;
use sekonic_schema::form_schema::{
    BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, FormOrientation,
    GridGeometry, OmrFormSchema, SheetSide, ThresholdPolicy,
};
use sekonic_sim::paper_generator::{generate_synthetic_sheet, SheetScenario};
use sekonic_sim::virtual_sr6500::VirtualSr6500;
use sekonic_sync::wal_queue::WalOutboxQueue;
use sekonic_sync::ws_client::flush_outbox_once;
use sekonic_transport::memory_channel::InMemoryTransport;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;

#[tokio::test]
async fn test_e2e_2000_sheets_erasure_substacker_and_network_outage() {
    let schema = OmrFormSchema {
        schema_version: 1,
        form_id: "E2E-2000".into(),
        form_name: "E2E Stress 2000".into(),
        duplex: false,
        dropout_color: DropoutColor::Red,
        orientation: FormOrientation::Straight,
        multi_mark_char: "*".into(),
        blank_char: " ".into(),
        grid: GridGeometry {
            front_rows: 16,
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
                allow_multiple_marks: false,
                hidden_from_output: false,
                route_to_sub_on_multi: false,
                route_to_sub_on_blank: false,
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
                allow_multiple_marks: false,
                hidden_from_output: false,
                route_to_sub_on_multi: false,
                route_to_sub_on_blank: false,
                items: vec![BubbleGroup {
                    index: 1,
                    options: (0..5)
                        .map(|c| BubbleCoord {
                            label: ((b'A' + c as u8) as char).to_string(),
                            row: 2,
                            col: c,
                        })
                        .collect(),
                }],
            },
        ],
    };

    // 2,000 sheets: 150 with erasures, 30 invalid student IDs, 15 multi-marks, rest clean
    let mut sheets = Vec::with_capacity(2000);
    for i in 1..=2000u64 {
        let scen = if i <= 30 {
            SheetScenario::InvalidStudentId
        } else if i <= 180 {
            SheetScenario::WithErasures
        } else if i <= 195 {
            SheetScenario::WithMultiMarks
        } else {
            SheetScenario::Clean
        };
        sheets.push(generate_synthetic_sheet(&schema, i, scen, 9999));
    }

    let (host_tx, dev_rx) = InMemoryTransport::pair(64);
    let sim_handle = tokio::spawn(VirtualSr6500::new(sheets).run(dev_rx));

    let db_path = std::env::temp_dir().join("sekonic_e2e_2000.db");
    let _ = std::fs::remove_file(&db_path);
    let queue = WalOutboxQueue::open(&db_path).expect("open wal");

    let mut driver = OmrScannerDriver::new(host_tx);
    driver.initialize(&schema, true).await.expect("init");

    let mut roster = RosterValidator::new(vec![], false);
    let mut erasure_recovered = 0usize;
    let mut processed = 0usize;

    while let Some(cap) = driver.feed_and_read_sheet(&schema, true).await.expect("feed") {
        processed += 1;
        let graded = grade_sheet(&schema, &cap);
        if let Some(details) = graded.answer_details.get("matematik") {
            if details
                .iter()
                .any(|d| d.state == BubbleState::MarkedWithErasure)
            {
                erasure_recovered += 1;
            }
        }
        let target = match roster.validate(&graded) {
            StackerDecision::RouteToMain => StackerTarget::Main,
            StackerDecision::RouteToSubStacker(_) => StackerTarget::Sub,
        };
        driver.eject_sheet(target).await.expect("eject");

        let payload = serde_json::to_string(&graded).expect("json");
        queue
            .enqueue("ses_e2e", cap.seq_no, &payload, cap.seq_no)
            .expect("enqueue");
    }
    drop(driver);

    let sim_res = sim_handle.await.expect("sim join");
    assert_eq!(processed, 2000);
    assert_eq!(sim_res.sub_stacker.len(), 45);
    assert_eq!(sim_res.main_stacker.len(), 1955);
    assert_eq!(erasure_recovered, 150);

    // Simulate network return & flush all 2,000 sheets over WebSocket
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    let rec = received.clone();
    let srv = tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            if let Ok(mut ws) = accept_async(stream).await {
                while let Some(Ok(msg)) = ws.next().await {
                    if msg.is_text() {
                        rec.lock().await.push(msg.to_string());
                    }
                }
            }
        }
    });

    let flushed = flush_outbox_once(&queue, &format!("ws://{addr}"), 5000)
        .await
        .expect("flush all");
    assert_eq!(flushed, 2000);
    let _ = srv.await;
    assert_eq!(received.lock().await.len(), 2000);
    assert_eq!(queue.stats().expect("stats").pending, 0);
    let _ = std::fs::remove_file(&db_path);
}
