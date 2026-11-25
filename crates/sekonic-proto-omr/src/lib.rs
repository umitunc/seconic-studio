#![forbid(unsafe_code)]

//! Family A (SR-1800EX, SR-3500, SR-6000, SR-6500) STX/ETX OMR protocol driver.

pub mod codec;
pub mod commands;
pub mod driver;
pub mod status_decoder;

#[cfg(test)]
mod tests {
    use super::commands::StackerTarget;
    use super::driver::OmrScannerDriver;
    use sekonic_schema::form_schema::{
        BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, FormOrientation,
        GridGeometry, OmrFormSchema, SheetSide, ThresholdPolicy,
    };
    use sekonic_sim::paper_generator::{generate_synthetic_sheet, SheetScenario};
    use sekonic_sim::virtual_sr6500::VirtualSr6500;
    use sekonic_transport::memory_channel::InMemoryTransport;

    #[tokio::test]
    async fn test_500_sheets_with_sub_stacker_routing() {
        let schema = OmrFormSchema {
            schema_version: 1,
            form_id: "SIM-500".into(),
            form_name: "Sim 500".into(),
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
            }],
        };

        let mut sheets = Vec::with_capacity(500);
        for i in 1..=500u64 {
            let scen = if i % 10 == 0 {
                SheetScenario::InvalidStudentId
            } else if i % 5 == 0 {
                SheetScenario::WithErasures
            } else {
                SheetScenario::Clean
            };
            sheets.push(generate_synthetic_sheet(&schema, i, scen, 12345));
        }

        let (host_tx, dev_rx) = InMemoryTransport::pair(32);
        let sim_handle = tokio::spawn(VirtualSr6500::new(sheets).run(dev_rx));

        let mut driver = OmrScannerDriver::new(host_tx);
        driver.initialize(&schema, true).await.expect("init");
        assert_eq!(driver.model_name(), "SR-6500");

        let mut count = 0usize;
        while let Some(capture) = driver.feed_and_read_sheet(&schema, true).await.expect("feed") {
            count += 1;
            // Check student_id row 0 cols 0..5: if max < min_mark_density -> Sub-Stacker
            let max_d = (0..5u16)
                .map(|c| capture.matrix.get(SheetSide::Front, 0, c))
                .max()
                .unwrap_or(0);
            let target = if max_d < schema.threshold_policy.min_mark_density {
                StackerTarget::Sub
            } else {
                StackerTarget::Main
            };
            driver.eject_sheet(target).await.expect("eject");
        }
        assert_eq!(count, 500);
        drop(driver);

        let final_sim = sim_handle.await.expect("sim join");
        assert_eq!(final_sim.sub_stacker.len(), 50);
        assert_eq!(final_sim.main_stacker.len(), 450);
    }

    #[test]
    fn test_decode_md_16_level_and_binary() {
        // 17-value density (`RM1,3` -> `'0'..='@'` -> `0..=16`, plus `'A'..='F'` -> `10..=15`):
        // SekOku `cl > 4` rule: levels 0..=4 stay below 48 (`0..=32`), levels 5..=16 map strictly
        // monotonically to `80..=245` (`'@'` = 16 -> 245 > `'?'` = 15 -> 230).
        // Also verifies 48-sensor rows truncate cleanly to schema cols without shifting!
        let mut dst = vec![0u8; 6];
        let (obs_rows, right_timing) = OmrScannerDriver::<InMemoryTransport>::decode_md_into_slice(
            b"MD00@@045XX,9?@YY",
            &mut dst,
            2,
            3,
            true,
        );
        assert_eq!(obs_rows, 2);
        assert!(!right_timing);
        assert_eq!(dst, vec![0, 32, 64, 140, 224, 238]);

        // Blank sheet in 16-level mode (`max_level == 1` noise) must NOT turn into binary 220!
        let mut dst_blank = vec![0u8; 4];
        OmrScannerDriver::<InMemoryTransport>::decode_md_into_slice(
            b"MD00@@01,10",
            &mut dst_blank,
            2,
            2,
            true,
        );
        assert_eq!(dst_blank, vec![0, 8, 8, 0]);

        // Pure binary `'0'`/`'1'` when `force_16level == false` still maps `'1'` -> 220
        let mut dst_bin = vec![0u8; 4];
        OmrScannerDriver::<InMemoryTransport>::decode_md_into_slice(
            b"MD000001,10",
            &mut dst_bin,
            2,
            2,
            false,
        );
        assert_eq!(dst_bin, vec![0, 220, 220, 0]);
    }

    #[test]
    fn test_sr6000_status_codes_and_reversed_sheet_detection() {
        use super::status_decoder::HardwareCondition;

        // Verified against real SR-6000 capture (`SFQ1@@`, `BRP1@@`, `HUP1@@`, `CE00@@`)
        assert_eq!(HardwareCondition::from_status_code("00"), HardwareCondition::Ready);
        assert_eq!(HardwareCondition::from_status_code("@@"), HardwareCondition::Ready);
        assert_eq!(HardwareCondition::from_status_code("P1"), HardwareCondition::Ready);
        assert_eq!(HardwareCondition::from_status_code("Q1"), HardwareCondition::HopperEmpty);
        assert_eq!(HardwareCondition::from_status_code("Q2"), HardwareCondition::DoubleFeed);
        assert_eq!(HardwareCondition::from_status_code("Q3"), HardwareCondition::SkewError);
        assert_eq!(HardwareCondition::from_status_code("G1"), HardwareCondition::CoverOpen);
        assert_eq!(HardwareCondition::from_status_code("H2"), HardwareCondition::PaperJam);
        assert_eq!(HardwareCondition::from_status_code("R4"), HardwareCondition::TimingMarkError);

        // Simulate SR-6000 narrow A4 sheet where channels 45..47 sit over the black platen (`@@@`)
        // on all 20 rows: channel 45 (`0`-based) must be masked to 0 (not flood option `E`),
        // and `obs_rows` (20 vs expected 61) must be reported accurately without false right_timing.
        let row_str = format!("{}@@@", "0".repeat(45));
        let wire = format!("MD00@@{}", vec![row_str.as_str(); 20].join(","));
        let mut dst = vec![0u8; 61 * 46];
        let (obs_rows, right_timing) = OmrScannerDriver::<InMemoryTransport>::decode_md_into_slice(
            wire.as_bytes(),
            &mut dst,
            61,
            46,
            true,
        );
        assert_eq!(obs_rows, 20);
        assert!(!right_timing);
        assert_eq!(dst[45], 0);
    }
}
