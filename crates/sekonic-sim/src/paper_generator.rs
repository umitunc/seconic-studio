use sekonic_schema::form_schema::{OmrFormSchema, SheetSide};
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

    for field in &schema.fields {
        for (idx, group) in field.items.iter().enumerate() {
            if group.options.is_empty() {
                continue;
            }
            if scenario == SheetScenario::InvalidStudentId
                && matches!(
                    field.field_type,
                    sekonic_schema::form_schema::FieldType::StudentId
                )
            {
                // Leave student ID blank
                continue;
            }

            let chosen = (rng.next_u64() as usize) % group.options.len();
            let opt = &group.options[chosen];
            matrix.set(field.side, opt.row, opt.col, rng.range(195, 232));

            if scenario == SheetScenario::WithErasures && idx % 3 == 0 && group.options.len() > 1 {
                let erased = (chosen + 1) % group.options.len();
                let eopt = &group.options[erased];
                matrix.set(field.side, eopt.row, eopt.col, rng.range(60, 74));
            } else if scenario == SheetScenario::WithMultiMarks
                && idx == 0
                && group.options.len() > 1
            {
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
