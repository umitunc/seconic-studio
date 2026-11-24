use sekonic_schema::form_schema::{FieldType, OmrFormSchema, SheetSide};
use sekonic_schema::raw_capture::Matrix2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SheetScenario {
    Clean,
    WithErasures,
    WithMultiMarks,
    InvalidStudentId,
}

#[derive(Debug, Clone)]
pub struct GeneratedSheet {
    pub seq_no: u64,
    pub scenario: SheetScenario,
    pub barcode: Option<String>,
    pub matrix: Matrix2D,
}

const TURKISH_FIRST_NAMES: &[&str] = &[
    "AHMET", "AYŞE", "MEHMET", "ZEYNEP", "MUSTAFA", "ELİF", "EMRE", "FATMA",
    "BURAK", "MERVE", "CAN", "ECEM", "KEREM", "DEFNE", "YUSUF", "ASLI",
    "ALİ", "İREM", "ONUR", "SELİN", "ARDA", "DENİZ", "KAAN", "BÜŞRA",
];

const TURKISH_LAST_NAMES: &[&str] = &[
    "YILMAZ", "KAYA", "DEMİR", "ÇELİK", "ŞAHİN", "YILDIZ", "ARSLAN", "ÖZTÜRK",
    "AYDIN", "ÖZDEMİR", "DOĞAN", "KILIÇ", "ASLAN", "ÇETİN", "KARA", "KOÇ",
    "KURT", "ÖZKAN", "ŞİMŞEK", "POLAT", "KORKMAZ", "AKSOY", "YILDIRIM", "TAŞ",
];

/// Deterministic PRNG (xorshift64) so test runs are 100% reproducible without external deps.
struct Rng64(u64);

impl Rng64 {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn range(&mut self, lo: u8, hi: u8) -> u8 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u64() % u64::from(hi - lo + 1)) as u8
    }
}

fn mark_char_in_group(
    matrix: &mut Matrix2D,
    side: SheetSide,
    options: &[sekonic_schema::form_schema::BubbleCoord],
    target_ch: char,
    density: u8,
    fallback_seed: usize,
) {
    if options.is_empty() {
        return;
    }
    let target = target_ch.to_string();
    let opt = options
        .iter()
        .find(|o| o.label == target)
        .unwrap_or(&options[fallback_seed % options.len()]);
    matrix.set(side, opt.row, opt.col, density);
}

/// Generates a synthetic 0-255 analog density matrix for a given `OmrFormSchema`.
pub fn generate_synthetic_sheet(
    schema: &OmrFormSchema,
    seq_no: u64,
    scenario: SheetScenario,
    seed: u64,
) -> GeneratedSheet {
    let mut rng = Rng64(seed ^ seq_no.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 1);
    let mut matrix = Matrix2D::new(schema.grid.front_rows, schema.grid.front_cols, schema.duplex);

    // Fill realistic paper background noise (6..14)
    for cell in &mut matrix.front {
        *cell = rng.range(6, 14);
    }
    if let Some(back) = matrix.back.as_mut() {
        for cell in back {
            *cell = rng.range(6, 14);
        }
    }

    let first = TURKISH_FIRST_NAMES[(seq_no as usize) % TURKISH_FIRST_NAMES.len()];
    let last = TURKISH_LAST_NAMES[((seq_no as usize) * 7 + 3) % TURKISH_LAST_NAMES.len()];
    let full_name = format!("{first} {last}");
    let student_no_val = 1000 + (seq_no * 37) % 8999;

    for field in &schema.fields {
        let key_lower = field.key.to_lowercase();
        let is_student_no = key_lower.contains("ogrenci")
            || key_lower.contains("öğrenci")
            || key_lower.contains("student_id")
            || key_lower.contains("numara");
        let is_name_field = key_lower.contains("adi")
            || key_lower.contains("adı")
            || key_lower.contains("soyad")
            || key_lower.contains("name");
        let is_tc_field = key_lower.contains("tc") || key_lower.contains("kimlik") || key_lower.contains("tel");

        if scenario == SheetScenario::InvalidStudentId && (is_student_no || field.required_for_valid_sheet) {
            // Simulate a student who forgot to bubble their student ID
            continue;
        }

        if is_name_field {
            let name_src = if key_lower.contains("soyad") && !key_lower.contains("ad") {
                last.to_string()
            } else if key_lower == "adi" || key_lower == "adı" {
                first.to_string()
            } else {
                full_name.clone()
            };
            let chars: Vec<char> = name_src.chars().collect();
            for (idx, group) in field.items.iter().enumerate() {
                if let Some(&ch) = chars.get(idx) {
                    if ch != ' ' {
                        mark_char_in_group(
                            &mut matrix,
                            field.side,
                            &group.options,
                            ch,
                            rng.range(195, 232),
                            (seq_no as usize) + idx,
                        );
                    }
                }
            }
            continue;
        }

        if is_student_no {
            let width = field.items.len();
            let formatted = format!("{:0width$}", student_no_val, width = width);
            let digits: Vec<char> = formatted.chars().collect();
            let offset = digits.len().saturating_sub(width);
            for (idx, group) in field.items.iter().enumerate() {
                if let Some(&ch) = digits.get(offset + idx) {
                    mark_char_in_group(
                        &mut matrix,
                        field.side,
                        &group.options,
                        ch,
                        rng.range(195, 232),
                        (seq_no as usize) + idx,
                    );
                    if scenario == SheetScenario::WithMultiMarks && idx == 0 && group.options.len() > 1 {
                        let next_opt = &group.options[((seq_no as usize) + 1) % group.options.len()];
                        if matrix.get(field.side, next_opt.row, next_opt.col) < 100 {
                            matrix.set(field.side, next_opt.row, next_opt.col, rng.range(195, 228));
                        } else {
                            let alt_opt = &group.options[((seq_no as usize) + 2) % group.options.len()];
                            matrix.set(field.side, alt_opt.row, alt_opt.col, rng.range(195, 228));
                        }
                    }
                }
            }
            continue;
        }

        if is_tc_field && field.items.len() >= 10 {
            let tc_str = format!("05{:09}", 300_000_000u64 + (seq_no * 7919) % 699_999_999);
            for (idx, group) in field.items.iter().enumerate() {
                if let Some(ch) = tc_str.chars().nth(idx) {
                    mark_char_in_group(
                        &mut matrix,
                        field.side,
                        &group.options,
                        ch,
                        rng.range(195, 232),
                        (seq_no as usize) + idx,
                    );
                }
            }
            continue;
        }

        for (idx, group) in field.items.iter().enumerate() {
            if group.options.is_empty() {
                continue;
            }
            if scenario == SheetScenario::InvalidStudentId && matches!(field.field_type, FieldType::StudentId) {
                continue;
            }

            // Leave ~6% of subject questions blank for realistic OMR output (except forced test indices)
            let is_erasure_idx = scenario == SheetScenario::WithErasures && idx % 3 == 0;
            let is_multimark_idx = scenario == SheetScenario::WithMultiMarks && idx == 0;
            if !is_erasure_idx
                && !is_multimark_idx
                && matches!(field.field_type, FieldType::SubjectQuestions)
                && (rng.next_u64() % 16) == 0
            {
                continue;
            }

            let chosen = (rng.next_u64() as usize) % group.options.len();
            let opt = &group.options[chosen];
            matrix.set(field.side, opt.row, opt.col, rng.range(195, 232));

            if is_erasure_idx && group.options.len() > 1 {
                let erased = (chosen + 1) % group.options.len();
                let eopt = &group.options[erased];
                matrix.set(field.side, eopt.row, eopt.col, rng.range(60, 74));
            } else if is_multimark_idx && group.options.len() > 1 {
                let second = (chosen + 1) % group.options.len();
                let sopt = &group.options[second];
                matrix.set(field.side, sopt.row, sopt.col, rng.range(180, 210));
            }
        }
    }

    GeneratedSheet {
        seq_no,
        scenario,
        barcode: Some(format!("BC{:08}", seq_no)),
        matrix,
    }
}

/// Renders a synthetic 8-bit grayscale image (255 = white paper, 0 = dark mark) for SR-55D CIS testing.
pub fn render_cis_grayscale_image(
    matrix: &Matrix2D,
    side: SheetSide,
    width_px: u16,
    height_px: u16,
) -> Vec<u8> {
    let mut img = vec![245u8; usize::from(width_px) * usize::from(height_px)];
    let row_step = height_px / matrix.rows.max(1);
    let col_step = width_px / (matrix.cols + 2).max(1);

    // Draw left timing marks at x = 2..10 centered at cy
    for r in 0..matrix.rows {
        let cy = r * row_step + row_step / 2;
        for dy in -2i32..=2 {
            let y = (i32::from(cy) + dy).clamp(0, i32::from(height_px - 1)) as usize;
            for x in 2..10u16 {
                img[y * usize::from(width_px) + usize::from(x)] = 20;
            }
        }
    }

    // Draw bubble marks centered at (cx, cy)
    for r in 0..matrix.rows {
        let cy = r * row_step + row_step / 2;
        for c in 0..matrix.cols {
            let density = matrix.get(side, r, c);
            if density > 20 {
                let cx = (c + 1) * col_step + col_step / 2;
                let lum = 255u8.saturating_sub(density);
                for dy in -2i32..=2 {
                    let y = (i32::from(cy) + dy).clamp(0, i32::from(height_px - 1)) as usize;
                    for dx in -2i32..=2 {
                        let x = (i32::from(cx) + dx).clamp(0, i32::from(width_px - 1)) as usize;
                        img[y * usize::from(width_px) + x] = lum;
                    }
                }
            }
        }
    }
    img
}
