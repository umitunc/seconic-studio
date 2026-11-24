#![forbid(unsafe_code)]

use base64::Engine;
use sekonic_grader::field_decoder::grade_sheet;
use sekonic_schema::form_schema::OmrFormSchema;
use sekonic_schema::legacy_bldm::parse_bldm;
use sekonic_schema::raw_capture::{Matrix2D, RawSheetCapture};
use wasm_bindgen::prelude::*;

/// Converts a legacy `.bldm` template into `OmrFormSchema` JSON.
#[wasm_bindgen]
pub fn convert_bldm_to_schema_json(form_id: &str, content: &str) -> Result<String, JsValue> {
    let schema = parse_bldm(form_id, content).map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_string(&schema).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Grades a raw 0-255 analog matrix (base64 front) against an `OmrFormSchema` JSON in-browser.
#[wasm_bindgen]
pub fn grade_raw_matrix_json(
    schema_json: &str,
    seq_no: u64,
    raw_front_b64: &str,
) -> Result<String, JsValue> {
    let schema: OmrFormSchema =
        serde_json::from_str(schema_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let front = base64::engine::general_purpose::STANDARD
        .decode(raw_front_b64)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut matrix = Matrix2D::new(schema.grid.front_rows, schema.grid.front_cols, false);
    let len = front.len().min(matrix.front.len());
    matrix.front[..len].copy_from_slice(&front[..len]);

    let cap = RawSheetCapture {
        seq_no,
        timestamp_ms: 0,
        barcode: None,
        matrix,
        raw_wire_sample: None,
        observed_front_rows: None,
        right_edge_timing_detected: false,
    };
    let graded = grade_sheet(&schema, &cap);
    serde_json::to_string(&graded).map_err(|e| JsValue::from_str(&e.to_string()))
}
