use crate::analog_solver::{estimate_sheet_baseline, solve_bubble_group, BubbleDecision, BubbleState};
use base64::Engine;
use sekonic_schema::form_schema::{FieldType, OmrFormSchema};
use sekonic_schema::raw_capture::RawSheetCapture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradedSheet {
    pub seq_no: u64,
    pub barcode: Option<String>,
    pub decoded_fields: HashMap<String, String>,
    pub answers: HashMap<String, String>,
    pub answer_details: HashMap<String, Vec<BubbleDecision>>,
    pub warnings: Vec<String>,
    pub needs_operator_review: bool,
    pub missing_required_field: bool,
    pub raw_matrix_b64: String,
}

pub fn grade_sheet(schema: &OmrFormSchema, capture: &RawSheetCapture) -> GradedSheet {
    let baseline = estimate_sheet_baseline(&capture.matrix);
    let mut decoded_fields = HashMap::new();
    let mut answers = HashMap::new();
    let mut answer_details = HashMap::new();
    let mut warnings = Vec::new();
    let mut needs_operator_review = false;
    let mut missing_required_field = false;

    for field in &schema.fields {
        let mut decisions = Vec::with_capacity(field.items.len());
        let mut compact = String::with_capacity(field.items.len());
        let mut field_valid = true;

        for group in &field.items {
            let dec = solve_bubble_group(
                &capture.matrix,
                field.side,
                group,
                baseline,
                &schema.threshold_policy,
            );
            if dec.confidence < schema.threshold_policy.review_confidence_floor {
                needs_operator_review = true;
            }
            match dec.state {
                BubbleState::Blank => {
                    compact.push(' ');
                    if field.required_for_valid_sheet {
                        field_valid = false;
                    }
                }
                BubbleState::Marked => {
                    if let Some(v) = &dec.value {
                        compact.push_str(v);
                    }
                }
                BubbleState::MarkedWithErasure => {
                    if let Some(v) = &dec.value {
                        compact.push_str(v);
                    }
                    warnings.push(format!("{}_q{}_erasure_recovered", field.key, group.index));
                }
                BubbleState::MultiMark => {
                    compact.push('*');
                    needs_operator_review = true;
                    warnings.push(format!("{}_q{}_multi_mark", field.key, group.index));
                    if field.required_for_valid_sheet {
                        field_valid = false;
                    }
                }
            }
            decisions.push(dec);
        }

        if field.required_for_valid_sheet && !field_valid {
            missing_required_field = true;
        }

        match field.field_type {
            FieldType::SubjectQuestions => {
                answers.insert(field.key.clone(), compact);
                answer_details.insert(field.key.clone(), decisions);
            }
            _ => {
                decoded_fields.insert(field.key.clone(), compact);
                answer_details.insert(field.key.clone(), decisions);
            }
        }
    }

    let raw_matrix_b64 = base64::engine::general_purpose::STANDARD.encode(&capture.matrix.front);

    GradedSheet {
        seq_no: capture.seq_no,
        barcode: capture.barcode.clone(),
        decoded_fields,
        answers,
        answer_details,
        warnings,
        needs_operator_review,
        missing_required_field,
        raw_matrix_b64,
    }
}
