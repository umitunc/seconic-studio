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
        BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, FormOrientation,
        GridGeometry, OmrFormSchema, SheetSide, ThresholdPolicy,
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
            orientation: FormOrientation::Straight,
            multi_mark_char: "*".into(),
            blank_char: " ".into(),
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
                    allow_multiple_marks: false,
                    hidden_from_output: false,
                    route_to_sub_on_multi: false,
                    route_to_sub_on_blank: false,
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
                    allow_multiple_marks: false,
                    hidden_from_output: false,
                    route_to_sub_on_multi: false,
                    route_to_sub_on_blank: false,
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

    #[test]
    fn test_sekoku_saturation_multi_char_and_c2_multi_select() {
        // 1. Verify `SekOku.exe` `Üst Sınır = 9` (`saturation_density = 135`) behavior:
        //    - Two intentional pencil marks (`lvl 11 -> 168` and `lvl 16 -> 238`) both saturate at 135 -> MultiMark (`*`)!
        //    - One erasure (`lvl 6 -> 80 < 135`) + one real mark (`lvl 15 -> 224 >= 135`) -> MarkedWithErasure!
        let mut m = Matrix2D::new(4, 6, false);
        m.set(SheetSide::Front, 0, 0, 168); // lvl 11
        m.set(SheetSide::Front, 0, 3, 238); // lvl 16 ('@')
        let grp_q1 = BubbleGroup {
            index: 1,
            options: (0..5)
                .map(|c| BubbleCoord {
                    label: ((b'A' + c as u8) as char).to_string(),
                    row: 0,
                    col: c,
                })
                .collect(),
        };
        let dec_dual = solve_bubble_group(&m, SheetSide::Front, &grp_q1, 0, &ThresholdPolicy::default());
        assert_eq!(dec_dual.state, BubbleState::MultiMark);

        m.set(SheetSide::Front, 1, 1, 80);  // lvl 6 erasure on B
        m.set(SheetSide::Front, 1, 2, 224); // lvl 15 mark on C
        let grp_q2 = BubbleGroup {
            index: 2,
            options: (0..5)
                .map(|c| BubbleCoord {
                    label: ((b'A' + c as u8) as char).to_string(),
                    row: 1,
                    col: c,
                })
                .collect(),
        };
        let dec_erase = solve_bubble_group(&m, SheetSide::Front, &grp_q2, 0, &ThresholdPolicy::default());
        assert_eq!(dec_erase.state, BubbleState::MarkedWithErasure);
        assert_eq!(dec_erase.value.as_deref(), Some("C"));

        // 2. Verify `C2` (`allow_multiple_marks = true`) & `~` multi-char token width (`01~02~03`):
        m.set(SheetSide::Front, 2, 0, 196); // A
        m.set(SheetSide::Front, 2, 3, 210); // D
        m.set(SheetSide::Front, 3, 1, 210); // "10"
        let schema = OmrFormSchema {
            schema_version: 1,
            form_id: "FMT-C2".into(),
            form_name: "FMT C2".into(),
            duplex: false,
            dropout_color: DropoutColor::Red,
            orientation: FormOrientation::Reversed180,
            multi_mark_char: "*".into(),
            blank_char: " ".into(),
            grid: GridGeometry {
                front_rows: 4,
                front_cols: 6,
                back_rows: 0,
                back_cols: 0,
                width_mm: 210.0,
                height_mm: 297.0,
            },
            threshold_policy: ThresholdPolicy::default(),
            fields: vec![
                FieldDefinition {
                    key: "sinif".into(),
                    field_type: FieldType::StudentId,
                    side: SheetSide::Front,
                    required_for_valid_sheet: false,
                    allow_multiple_marks: false,
                    hidden_from_output: false,
                    route_to_sub_on_multi: false,
                    route_to_sub_on_blank: false,
                    items: vec![BubbleGroup {
                        index: 1,
                        options: vec![
                            BubbleCoord { label: "09".into(), row: 3, col: 0 },
                            BubbleCoord { label: "10".into(), row: 3, col: 1 },
                            BubbleCoord { label: "11".into(), row: 3, col: 2 },
                        ],
                    }],
                },
                FieldDefinition {
                    key: "cevaplar".into(),
                    field_type: FieldType::SubjectQuestions,
                    side: SheetSide::Front,
                    required_for_valid_sheet: false,
                    allow_multiple_marks: true,
                    hidden_from_output: false,
                    route_to_sub_on_multi: false,
                    route_to_sub_on_blank: false,
                    items: vec![BubbleGroup {
                        index: 1,
                        options: (0..6)
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
        let cap = RawSheetCapture {
            seq_no: 1,
            timestamp_ms: 0,
            barcode: None,
            matrix: m,
            raw_wire_sample: None,
            observed_front_rows: Some(4),
            right_edge_timing_detected: false,
        };
        let graded = grade_sheet(&schema, &cap);
        assert_eq!(graded.decoded_fields.get("sinif").map(String::as_str), Some("10"));
        assert_eq!(graded.answers.get("cevaplar").map(String::as_str), Some("AD    "));
        assert!(!graded.needs_operator_review);
    }

    #[test]
    fn test_x2_blank_digits_and_light_erasure_parity() {
        // 1. Medium-light mark (`lvl 7 -> 96 < t_strong 115`) vs faint erasure (`lvl 5 -> 64`):
        //    SekOku.exe picks the darker mark (`96 > 64`, ratio 0.667 <= 0.82) rather than MultiMark (`*`).
        let mut m = Matrix2D::new(4, 6, false);
        m.set(SheetSide::Front, 0, 0, 64); // lvl 5 erasure on A
        m.set(SheetSide::Front, 0, 2, 96); // lvl 7 real mark on C
        let grp = BubbleGroup {
            index: 1,
            options: (0..5)
                .map(|c| BubbleCoord {
                    label: ((b'A' + c as u8) as char).to_string(),
                    row: 0,
                    col: c,
                })
                .collect(),
        };
        let dec = solve_bubble_group(&m, SheetSide::Front, &grp, 0, &ThresholdPolicy::default());
        assert_eq!(dec.state, BubbleState::MarkedWithErasure);
        assert_eq!(dec.value.as_deref(), Some("C"));

        // 2. X2 mode (`required_for_valid_sheet = false, route_to_sub_on_blank = false`):
        //    Partially blank student ID (e.g. " 2075") must NOT route to Sub-Stacker!
        let mut m2 = Matrix2D::new(10, 5, false);
        m2.set(SheetSide::Front, 2, 1, 210); // col 1 = '2'
        m2.set(SheetSide::Front, 0, 2, 210); // col 2 = '0'
        m2.set(SheetSide::Front, 7, 3, 210); // col 3 = '7'
        m2.set(SheetSide::Front, 5, 4, 210); // col 4 = '5'
        let x2_schema = OmrFormSchema {
            schema_version: 1,
            form_id: "X2-BLANK".into(),
            form_name: "X2 Blank Test".into(),
            duplex: false,
            dropout_color: DropoutColor::Red,
            orientation: FormOrientation::Straight,
            multi_mark_char: "*".into(),
            blank_char: " ".into(),
            grid: GridGeometry {
                front_rows: 10,
                front_cols: 5,
                back_rows: 0,
                back_cols: 0,
                width_mm: 210.0,
                height_mm: 297.0,
            },
            threshold_policy: ThresholdPolicy::default(),
            fields: vec![FieldDefinition {
                key: "ogrenci_no".into(),
                field_type: FieldType::StudentId,
                side: SheetSide::Front,
                required_for_valid_sheet: false,
                allow_multiple_marks: false,
                hidden_from_output: false,
                route_to_sub_on_multi: false,
                route_to_sub_on_blank: false,
                items: (0..5u16)
                    .map(|c| BubbleGroup {
                        index: c + 1,
                        options: (0..10u16)
                            .map(|r| BubbleCoord {
                                label: r.to_string(),
                                row: r,
                                col: c,
                            })
                            .collect(),
                    })
                    .collect(),
            }],
        };
        let cap2 = RawSheetCapture {
            seq_no: 1,
            timestamp_ms: 0,
            barcode: None,
            matrix: m2,
            raw_wire_sample: None,
            observed_front_rows: Some(10),
            right_edge_timing_detected: false,
        };
        let g2 = grade_sheet(&x2_schema, &cap2);
        assert_eq!(g2.decoded_fields.get("ogrenci_no").map(String::as_str), Some(" 2075"));
        assert!(!g2.missing_required_field);

        // 3. S/U mode (`route_to_sub_on_blank = true`) MUST route to Sub-Stacker on blank!
        let mut s_schema = x2_schema;
        s_schema.fields[0].route_to_sub_on_blank = true;
        let g3 = grade_sheet(&s_schema, &cap2);
        assert!(g3.missing_required_field);
    }
}
