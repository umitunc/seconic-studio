#![forbid(unsafe_code)]

//! Analog 0-255 erasure-aware OMR solver, field decoder, and roster validator.

pub mod analog_solver;
pub mod field_decoder;
pub mod roster_validator;

#[cfg(test)]
mod tests {
    use super::analog_solver::{solve_bubble_group, BubbleState};
    use super::field_decoder::grade_sheet;
    use super::roster_validator::{RosterValidator, StackerDecision, SubStackerReason};
    use sekonic_schema::form_schema::{
        BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, GridGeometry,
        OmrFormSchema, SheetSide, ThresholdPolicy,
    };
    use sekonic_schema::raw_capture::{Matrix2D, RawSheetCapture};
    use std::time::Instant;

    fn sample_schema() -> OmrFormSchema {
        let mut q_items = Vec::new();
        for q in 0..20u16 {
            let options = ['A', 'B', 'C', 'D', 'E']
                .iter()
                .enumerate()
                .map(|(c, ch)| BubbleCoord {
                    label: ch.to_string(),
                    row: q + 2,
                    col: c as u16,
                })
                .collect();
            q_items.push(BubbleGroup {
                index: q + 1,
                options,
            });
        }
        OmrFormSchema {
            schema_version: 1,
            form_id: "LGS-TEST".into(),
            form_name: "LGS Test".into(),
            duplex: false,
            dropout_color: DropoutColor::Red,
            grid: GridGeometry {
                front_rows: 32,
                front_cols: 16,
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
                        options: (0..4)
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
                    items: q_items,
                },
            ],
        }
    }

    #[test]
    fn test_erasure_recovery_vs_multimark() {
        let mut m = Matrix2D::new(8, 8, false);
        m.front.fill(10);
        // Q1: A erased (62 -> net 52 >= 48), C marked (210 -> net 200) -> should recover C
        m.set(SheetSide::Front, 0, 0, 62);
        m.set(SheetSide::Front, 0, 2, 210);
        let group = BubbleGroup {
            index: 1,
            options: vec![
                BubbleCoord { label: "A".into(), row: 0, col: 0 },
                BubbleCoord { label: "B".into(), row: 0, col: 1 },
                BubbleCoord { label: "C".into(), row: 0, col: 2 },
            ],
        };
        let dec = solve_bubble_group(&m, SheetSide::Front, &group, 10, &ThresholdPolicy::default());
        assert_eq!(dec.state, BubbleState::MarkedWithErasure);
        assert_eq!(dec.value.as_deref(), Some("C"));
        assert_eq!(dec.erased_label.as_deref(), Some("A"));

        // True multi-mark: A=188, B=182
        m.set(SheetSide::Front, 0, 0, 188);
        m.set(SheetSide::Front, 0, 1, 182);
        m.set(SheetSide::Front, 0, 2, 10);
        let dec2 = solve_bubble_group(&m, SheetSide::Front, &group, 10, &ThresholdPolicy::default());
        assert_eq!(dec2.state, BubbleState::MultiMark);
    }

    #[test]
    fn test_roster_duplicate_and_speed_10k_sheets() {
        let schema = sample_schema();
        let mut m = Matrix2D::new(32, 16, false);
        m.front.fill(10);
        m.set(SheetSide::Front, 0, 1, 215); // student_id = "1"
        for q in 0..20u16 {
            m.set(SheetSide::Front, q + 2, q % 5, 210);
        }
        let cap = RawSheetCapture {
            seq_no: 1,
            timestamp_ms: 0,
            barcode: None,
            matrix: m,
            raw_wire_sample: None,
            observed_front_rows: None,
            right_edge_timing_detected: false,
        };

        let mut roster = RosterValidator::new(vec!["1".into()], true);
        let g1 = grade_sheet(&schema, &cap);
        assert_eq!(roster.validate(&g1), StackerDecision::RouteToMain);
        assert_eq!(
            roster.validate(&g1),
            StackerDecision::RouteToSubStacker(SubStackerReason::DuplicateStudentSheet)
        );

        let start = Instant::now();
        for _ in 0..10_000 {
            let _ = grade_sheet(&schema, &cap);
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 500,
            "10k sheets took {} ms (expected < 500 ms)",
            elapsed.as_millis()
        );
    }
}
