use crate::field_decoder::GradedSheet;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StackerDecision {
    RouteToMain,
    RouteToSubStacker(SubStackerReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubStackerReason {
    MissingRequiredField,
    StudentNotInRoster,
    DuplicateStudentSheet,
}

/// O(1) HashSet-backed exam roster validator for real-time Main vs Sub-Stacker routing.
pub struct RosterValidator {
    expected_students: HashSet<String>,
    seen_students: HashSet<String>,
    enforce_roster: bool,
}

impl RosterValidator {
    pub fn new(expected_students: impl IntoIterator<Item = String>, enforce_roster: bool) -> Self {
        Self {
            expected_students: expected_students.into_iter().collect(),
            seen_students: HashSet::new(),
            enforce_roster,
        }
    }

    pub fn validate(&mut self, graded: &GradedSheet) -> StackerDecision {
        if graded.missing_required_field {
            return StackerDecision::RouteToSubStacker(SubStackerReason::MissingRequiredField);
        }
        let student_id = graded
            .decoded_fields
            .get("student_id")
            .or_else(|| graded.decoded_fields.get("ogrenci_no"))
            .map(|s| s.trim())
            .unwrap_or("");

        if self.enforce_roster {
            if student_id.is_empty() || !self.expected_students.contains(student_id) {
                return StackerDecision::RouteToSubStacker(SubStackerReason::StudentNotInRoster);
            }
            if !self.seen_students.insert(student_id.to_string()) {
                return StackerDecision::RouteToSubStacker(
                    SubStackerReason::DuplicateStudentSheet,
                );
            }
        }
        StackerDecision::RouteToMain
    }
}
