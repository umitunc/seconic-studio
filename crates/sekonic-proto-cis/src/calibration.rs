/// Per-column CIS shading (dark/white gain) calibration profile.
#[derive(Debug, Clone)]
pub struct CisShadingProfile {
    pub dark: Vec<u8>,
    pub white: Vec<u8>,
}

impl CisShadingProfile {
    pub fn identity(width_px: usize) -> Self {
        Self {
            dark: vec![0; width_px],
            white: vec![255; width_px],
        }
    }

    /// Normalizes a raw 8-bit grayscale scanline in-place using dark/white reference arrays.
    pub fn normalize_scanline(&self, line: &mut [u8]) {
        for ((px, &d), &w) in line.iter_mut().zip(self.dark.iter()).zip(self.white.iter()) {
            let denom = i32::from(w) - i32::from(d);
            if denom <= 0 {
                continue;
            }
            let norm = ((i32::from(*px) - i32::from(d)) * 255) / denom;
            *px = norm.clamp(0, 255) as u8;
        }
    }
}
