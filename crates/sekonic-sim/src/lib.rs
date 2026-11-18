#![forbid(unsafe_code)]

//! Virtual Sekonic hardware simulator (SR-6500 & SR-55D) and synthetic sheet generator.

pub mod paper_generator;
pub mod virtual_sr55d;
pub mod virtual_sr6500;

#[cfg(test)]
mod tests {
    use super::paper_generator::{generate_synthetic_sheet, SheetScenario};
    use super::virtual_sr6500::crc16_omr;
    use sekonic_schema::form_schema::{
        BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, GridGeometry,
        OmrFormSchema, SheetSide, ThresholdPolicy,
    };

    #[test]
    fn test_crc16_deterministic() {
        let frame = [0x02, 0x00, 0x02, b'M', b'N'];
        let c1 = crc16_omr(&frame);
        let c2 = crc16_omr(&frame);
        assert_eq!(c1, c2);
        assert_ne!(c1, 0);
    }

    #[test]
    fn test_synthetic_erasure_generation() {
        let schema = OmrFormSchema {
            schema_version: 1,
            form_id: "TEST-1".into(),
            form_name: "Test".into(),
            duplex: false,
            dropout_color: DropoutColor::Red,
            grid: GridGeometry {
                front_rows: 10,
                front_cols: 10,
                back_rows: 0,
                back_cols: 0,
                width_mm: 210.0,
                height_mm: 297.0,
            },
            threshold_policy: ThresholdPolicy::default(),
            fields: vec![FieldDefinition {
                key: "q".into(),
                field_type: FieldType::SubjectQuestions,
                side: SheetSide::Front,
                required_for_valid_sheet: false,
                items: vec![BubbleGroup {
                    index: 1,
                    options: vec![
                        BubbleCoord { label: "A".into(), row: 0, col: 0 },
                        BubbleCoord { label: "B".into(), row: 0, col: 1 },
                    ],
                }],
            }],
        };
        let s = generate_synthetic_sheet(&schema, 1, SheetScenario::WithErasures, 42);
        let d0 = s.matrix.get(SheetSide::Front, 0, 0);
        let d1 = s.matrix.get(SheetSide::Front, 0, 1);
        assert!(d0.max(d1) >= 195);
        assert!(d0.min(d1) >= 60 && d0.min(d1) <= 74);
    }
}
