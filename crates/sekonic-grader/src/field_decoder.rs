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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_wire_sample: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_front_rows: Option<u16>,
}

fn pad_to_chars(s: &str, width: usize, pad_ch: char) -> String {
    let count = s.chars().count();
    if count >= width {
        s.chars().take(width).collect()
    } else {
        let mut out = String::with_capacity(s.len() + (width - count));
        out.push_str(s);
        for _ in 0..(width - count) {
            out.push(pad_ch);
        }
        out
    }
}

pub fn grade_sheet(schema: &OmrFormSchema, capture: &RawSheetCapture) -> GradedSheet {
    let baseline = estimate_sheet_baseline(&capture.matrix);
    let mut decoded_fields = HashMap::new();
    let mut answers = HashMap::new();
    let mut answer_details = HashMap::new();
    let mut warnings = Vec::new();
    let mut needs_operator_review = false;
    let mut missing_required_field = false;

    let blank_ch = schema.blank_char.chars().next().unwrap_or(' ');
    let multi_str = if schema.multi_mark_char.is_empty() {
        "*"
    } else {
        schema.multi_mark_char.as_str()
    };

    if capture.right_edge_timing_detected {
        needs_operator_review = true;
        missing_required_field = true;
        warnings.push("kağıt_ters_yön_sağ_referans".to_string());
    }
    if let Some(obs_rows) = capture.observed_front_rows {
        if obs_rows != schema.grid.front_rows {
            needs_operator_review = true;
            missing_required_field = true;
            warnings.push(format!(
                "okuma_çizgisi_uyumsuz:{obs_rows}/{}",
                schema.grid.front_rows
            ));
        }
    }

    for field in &schema.fields {
        let mut decisions = Vec::with_capacity(field.items.len());
        let mut compact = String::with_capacity(field.items.len());
        let mut field_valid = true;

        for group in &field.items {
            let token_width = group
                .options
                .iter()
                .map(|o| o.label.chars().count())
                .max()
                .unwrap_or(1)
                .max(1);

            if field.allow_multiple_marks {
                // `C` / `M` modes (`SekOku.exe @ 0x4b1140` / `0x4b7cc0`): allow multiple marked bubbles
                // in the same item, concatenating marked labels in option order padded to option count.
                let mut raw_densities = Vec::with_capacity(group.options.len());
                let mut marked_labels = Vec::new();
                for opt in &group.options {
                    let raw = capture.matrix.get(field.side, opt.row, opt.col);
                    raw_densities.push(raw);
                    if opt.label != "`"
                        && raw.saturating_sub(baseline) >= schema.threshold_policy.min_mark_density
                    {
                        marked_labels.push(opt.label.clone());
                    }
                }
                let joined = marked_labels.join("");
                let slot_width = group.options.len().max(token_width);
                compact.push_str(&pad_to_chars(&joined, slot_width, blank_ch));
                decisions.push(BubbleDecision {
                    index: group.index,
                    state: if marked_labels.is_empty() {
                        BubbleState::Blank
                    } else {
                        BubbleState::Marked
                    },
                    value: if marked_labels.is_empty() {
                        None
                    } else {
                        Some(joined)
                    },
                    multi_values: marked_labels,
                    erased_label: None,
                    confidence: 0.98,
                    densities: raw_densities,
                });
                continue;
            }

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
                    for _ in 0..token_width {
                        compact.push(blank_ch);
                    }
                    if field.required_for_valid_sheet || field.route_to_sub_on_blank {
                        field_valid = false;
                    }
                }
                BubbleState::Marked => {
                    if let Some(v) = &dec.value {
                        compact.push_str(&pad_to_chars(v, token_width, blank_ch));
                    }
                }
                BubbleState::MarkedWithErasure => {
                    if let Some(v) = &dec.value {
                        compact.push_str(&pad_to_chars(v, token_width, blank_ch));
                    }
                    warnings.push(format!("{}_q{}_erasure_recovered", field.key, group.index));
                }
                BubbleState::MultiMark => {
                    compact.push_str(&pad_to_chars(multi_str, token_width, blank_ch));
                    needs_operator_review = true;
                    warnings.push(format!("{}_q{}_multi_mark", field.key, group.index));
                    if field.required_for_valid_sheet || field.route_to_sub_on_multi {
                        field_valid = false;
                    }
                }
            }
            decisions.push(dec);
        }

        if (field.required_for_valid_sheet
            || field.route_to_sub_on_multi
            || field.route_to_sub_on_blank)
            && !field_valid
        {
            missing_required_field = true;
        }

        if field.hidden_from_output {
            answer_details.insert(field.key.clone(), decisions);
            continue;
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
        raw_wire_sample: capture.raw_wire_sample.clone(),
        observed_front_rows: capture.observed_front_rows,
    }
}
