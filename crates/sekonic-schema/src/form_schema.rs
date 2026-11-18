use crate::error::SekonicError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OmrFormSchema {
    pub schema_version: u32,
    pub form_id: String,
    pub form_name: String,
    pub duplex: bool,
    pub dropout_color: DropoutColor,
    pub grid: GridGeometry,
    pub threshold_policy: ThresholdPolicy,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DropoutColor {
    None,
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GridGeometry {
    pub front_rows: u16,
    pub front_cols: u16,
    pub back_rows: u16,
    pub back_cols: u16,
    pub width_mm: f32,
    pub height_mm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThresholdPolicy {
    pub min_mark_density: u8,
    pub strong_mark_density: u8,
    pub erasure_ratio: f32,
    pub review_confidence_floor: f32,
}

impl Default for ThresholdPolicy {
    fn default() -> Self {
        Self {
            min_mark_density: 48,
            strong_mark_density: 115,
            erasure_ratio: 0.42,
            review_confidence_floor: 0.65,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldDefinition {
    pub key: String,
    pub field_type: FieldType,
    pub side: SheetSide,
    pub required_for_valid_sheet: bool,
    pub items: Vec<BubbleGroup>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    StudentId,
    NationalId,
    BookletType,
    ClassCode,
    SubjectQuestions,
    CustomCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SheetSide {
    Front,
    Back,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BubbleGroup {
    pub index: u16,
    pub options: Vec<BubbleCoord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BubbleCoord {
    pub label: String,
    pub row: u16,
    pub col: u16,
}

impl OmrFormSchema {
    /// Validates grid dimensions, unique field keys, and coordinate bounds.
    pub fn validate(&self) -> Result<(), SekonicError> {
        if self.form_id.trim().is_empty() {
            return Err(SekonicError::InvalidSchema("form_id cannot be empty".into()));
        }
        if self.grid.front_rows == 0 || self.grid.front_cols == 0 {
            return Err(SekonicError::InvalidSchema(
                "front_rows and front_cols must be > 0".into(),
            ));
        }
        if self.duplex && (self.grid.back_rows == 0 || self.grid.back_cols == 0) {
            return Err(SekonicError::InvalidSchema(
                "duplex form requires back_rows > 0 and back_cols > 0".into(),
            ));
        }

        let mut seen_keys = HashSet::new();
        for field in &self.fields {
            if !seen_keys.insert(field.key.as_str()) {
                return Err(SekonicError::InvalidSchema(format!(
                    "duplicate field key '{}'",
                    field.key
                )));
            }
            let (max_r, max_c) = match field.side {
                SheetSide::Front => (self.grid.front_rows, self.grid.front_cols),
                SheetSide::Back => {
                    if !self.duplex {
                        return Err(SekonicError::InvalidSchema(format!(
                            "field '{}' targets Back side on simplex form",
                            field.key
                        )));
                    }
                    (self.grid.back_rows, self.grid.back_cols)
                }
            };
            for group in &field.items {
                for opt in &group.options {
                    if opt.row >= max_r || opt.col >= max_c {
                        return Err(SekonicError::InvalidSchema(format!(
                            "field '{}' item {} option '{}' ({}, {}) out of grid bounds ({}, {})",
                            field.key, group.index, opt.label, opt.row, opt.col, max_r, max_c
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}
