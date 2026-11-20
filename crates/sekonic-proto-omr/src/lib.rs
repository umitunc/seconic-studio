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
        BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, GridGeometry,
        OmrFormSchema, SheetSide, ThresholdPolicy,
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
        // 16-level density (`RM1,3` -> `'0'..='?'` / `'A'..='F'`):
        // SekOku `cl > 4` rule: levels 0..=4 stay below 48 (`0..=32`), levels 5..=15 map to `85..=255`.
        // Also verifies 48-sensor rows truncate cleanly to 46-column schema without shifting!
        let mut dst = vec![0u8; 6];
        OmrScannerDriver::<InMemoryTransport>::decode_md_into_slice(
            b"MD0000045XX,9?FYY",
            &mut dst,
            2,
            3,
            true,
        );
        assert_eq!(dst, vec![0, 32, 85, 153, 255, 255]);

        // Blank sheet in 16-level mode (`max_level == 1` noise) must NOT turn into binary 220!
        let mut dst_blank = vec![0u8; 4];
        OmrScannerDriver::<InMemoryTransport>::decode_md_into_slice(
            b"MD000001,10",
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
}
