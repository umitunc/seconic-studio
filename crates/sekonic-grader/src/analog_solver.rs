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

/// Computes the 10th percentile paper background baseline $B_{sheet}$ across the front matrix.
/// Using the 10th percentile ensures even densely-marked sheets reflect true un-bubbled paper.
pub fn estimate_sheet_baseline(matrix: &Matrix2D) -> u8 {
    if matrix.front.is_empty() {
        return 0;
    }
    let mut hist = [0u32; 256];
    for &v in &matrix.front {
        hist[usize::from(v)] += 1;
    }
    let target = (matrix.front.len() / 10).max(1) as u32;
    let mut acc = 0u32;
    for (val, &cnt) in hist.iter().enumerate() {
        acc += cnt;
        if acc >= target {
            return val as u8;
        }
    }
    0
}

/// Solves a single `BubbleGroup` using `SekOku.exe`-compatible saturation + erasure contrast rules:
/// - Skips dummy backtick (`` ` ``) placeholder cells (`SekOku.exe @ 0x444de0`).
/// - Applies optional `saturation_density` (`Koyuluk Üst Sınırı`, default `135` = `MD` level `9`):
///   if two bubbles both reach saturation (`lvl >= 9`), both clamp to `sat` and produce `MultiMark` (`*`),
///   whereas a real mark (`>= sat`) cleanly beats a lighter pencil erasure (`< sat`) as `MarkedWithErasure`.
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
        if opt.label == "`" {
            continue;
        }
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

    let sat = policy.saturation_density.unwrap_or(255).max(t_min);
    let c1 = v1.min(sat);
    let c2 = v2.min(sat);
    let clamped_ratio = f32::from(c2) / f32::from(c1.max(1));

    if c1 > c2 && v1 >= t_strong && (v1 >= sat || clamped_ratio <= policy.erasure_ratio) {
        let conf = (1.0 - clamped_ratio * 0.5).clamp(0.68, 0.95);
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
    let conf = clamped_ratio.clamp(0.50, 1.0);
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
