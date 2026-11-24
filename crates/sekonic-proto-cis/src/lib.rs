#![forbid(unsafe_code)]

//! Family B (SR-55D & Hybrid CIS) image-based OMR protocol and calibration engine.

pub mod calibration;
pub mod commands;
pub mod image_omr;

#[cfg(test)]
mod tests {
    use super::image_omr::CisScannerDriver;
    use sekonic_schema::form_schema::{
        BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, FormOrientation,
        GridGeometry, OmrFormSchema, SheetSide, ThresholdPolicy,
    };
    use sekonic_sim::paper_generator::{generate_synthetic_sheet, SheetScenario};
    use sekonic_sim::virtual_sr55d::VirtualSr55d;
    use sekonic_transport::memory_channel::InMemoryTransport;

    #[tokio::test]
    async fn test_sr55d_cis_image_to_matrix_roundtrip() {
        let schema = OmrFormSchema {
            schema_version: 1,
            form_id: "CIS-1".into(),
            form_name: "CIS Test".into(),
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
            fields: vec![FieldDefinition {
                key: "q1".into(),
                field_type: FieldType::SubjectQuestions,
                side: SheetSide::Front,
                required_for_valid_sheet: true,
                allow_multiple_marks: false,
                hidden_from_output: false,
                route_to_sub_on_multi: false,
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
            }],
        };

        let sheet = generate_synthetic_sheet(&schema, 1, SheetScenario::WithErasures, 777);
        let expected_densities: Vec<u8> = (0..5)
            .map(|c| sheet.matrix.get(SheetSide::Front, 2, c))
            .collect();

        let (host_tx, dev_rx) = InMemoryTransport::pair(8);
        let sim_handle = tokio::spawn(VirtualSr55d::new(vec![sheet], 240, 320).run(dev_rx));

        let mut driver = CisScannerDriver::new(host_tx, 240, 320);
        driver.initialize(&schema).await.expect("init");
        let cap = driver
            .scan_sheet(&schema)
            .await
            .expect("scan")
            .expect("some sheet");
        driver.eject(false).await.expect("eject");

        for c in 0..5u16 {
            let got = cap.matrix.get(SheetSide::Front, 2, c);
            let exp = expected_densities[usize::from(c)];
            if exp >= 195 {
                assert!(got >= 150, "expected strong mark at col {c}, got {got}");
            } else if exp >= 45 {
                assert!(got >= 30 && got <= 90, "expected erasure at col {c}, got {got}");
            }
        }
        drop(driver);
        let _ = sim_handle.await;
    }
}
