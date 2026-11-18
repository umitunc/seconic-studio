use sekonic_schema::form_schema::{BubbleGroup, SheetSide, ThresholdPolicy};
use sekonic_schema::raw_capture::Matrix2D;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum BubbleState {
    Blank,
    Marked,
    MarkedWithErasure,
    MultiMark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleDecision {
    pub index: u16,
    pub state: BubbleState,
    pub value: Option<String>,
    pub multi_values: Vec<String>,
    pub erased_label: Option<String>,
    pub confidence: f32,
    pub densities: Vec<u8>,
}

/// Computes the 25th percentile paper background baseline $B_{sheet}$ across the front matrix.
pub fn estimate_sheet_baseline(matrix: &Matrix2D) -> u8 {
    if matrix.front.is_empty() {
        return 0;
    }
    let mut hist = [0u32; 256];
    for &v in &matrix.front {
        hist[usize::from(v)] += 1;
    }
    let target = (matrix.front.len() / 4).max(1) as u32;
    let mut acc = 0u32;
    for (val, &cnt) in hist.iter().enumerate() {
        acc += cnt;
        if acc >= target {
            return val as u8;
        }
    }
    0
}

/// Solves a single `BubbleGroup` using Section 3.3 analog 0-255 erasure-aware contrast rules.
pub fn solve_bubble_group(
    matrix: &Matrix2D,
    side: SheetSide,
    group: &BubbleGroup,
    baseline: u8,
    policy: &ThresholdPolicy,
) -> BubbleDecision {
    let mut raw_densities = Vec::with_capacity(group.options.len());
    let mut scored: Vec<(&str, u8)> = Vec::with_capacity(group.options.len());

    for opt in &group.options {
        let raw = matrix.get(side, opt.row, opt.col);
        raw_densities.push(raw);
        let net = raw.saturating_sub(baseline);
        scored.push((opt.label.as_str(), net));
    }

    if scored.is_empty() {
        return BubbleDecision {
            index: group.index,
            state: BubbleState::Blank,
            value: None,
            multi_values: Vec::new(),
            erased_label: None,
            confidence: 1.0,
            densities: raw_densities,
        };
    }

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    let (l1, v1) = scored[0];
    let (l2, v2) = if scored.len() > 1 { scored[1] } else { ("", 0) };

    let t_min = policy.min_mark_density;
    let t_strong = policy.strong_mark_density;

    if v1 < t_min {
        let conf = (1.0 - f32::from(v1) / f32::from(t_min.max(1))).clamp(0.5, 1.0);
        return BubbleDecision {
            index: group.index,
            state: BubbleState::Blank,
            value: None,
            multi_values: Vec::new(),
            erased_label: None,
            confidence: conf,
            densities: raw_densities,
        };
    }

    if v2 < t_min {
        let denom = f32::from(255u8.saturating_sub(t_min).max(1));
        let conf = (0.60 + 0.40 * (f32::from(v1 - t_min) / denom)).clamp(0.60, 1.0);
        return BubbleDecision {
            index: group.index,
            state: BubbleState::Marked,
            value: Some(l1.to_string()),
            multi_values: Vec::new(),
            erased_label: None,
            confidence: conf,
            densities: raw_densities,
        };
    }

    let ratio = f32::from(v2) / f32::from(v1.max(1));
    if v1 >= t_strong && ratio <= policy.erasure_ratio {
        let conf = (1.0 - ratio).clamp(0.65, 0.95);
        return BubbleDecision {
            index: group.index,
            state: BubbleState::MarkedWithErasure,
            value: Some(l1.to_string()),
            multi_values: Vec::new(),
            erased_label: Some(l2.to_string()),
            confidence: conf,
            densities: raw_densities,
        };
    }

    let multi: Vec<String> = scored
        .iter()
        .filter(|(_, v)| *v >= t_min)
        .map(|(lbl, _)| (*lbl).to_string())
        .collect();
    let conf = ratio.clamp(0.50, 1.0);
    BubbleDecision {
        index: group.index,
        state: BubbleState::MultiMark,
        value: None,
        multi_values: multi,
        erased_label: None,
        confidence: conf,
        densities: raw_densities,
    }
}
