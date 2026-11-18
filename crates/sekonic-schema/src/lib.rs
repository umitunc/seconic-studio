#![forbid(unsafe_code)]

//! Core schemas, error types, and legacy `.bldm` / `.fmt` converters.

pub mod error;
pub mod form_schema;
pub mod legacy_bldm;
pub mod raw_capture;

#[cfg(test)]
mod tests {
    use super::legacy_bldm::parse_bldm;
    use super::raw_capture::Matrix2D;
    use super::form_schema::SheetSide;
    use std::path::PathBuf;

    #[test]
    fn test_parse_real_bldm_files() {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../.cache/extracted_bins/sr1800kurulum/SR1800");
        for name in ["j-17.bldm", "j-34.bldm", "j-52.bldm", "j-64.bldm"] {
            let path = base.join(name);
            let content = std::fs::read_to_string(&path).expect("read .bldm fixture");
            let schema = parse_bldm(name, &content).expect("parse .bldm");
            assert_eq!(schema.grid.front_rows, 64);
            assert_eq!(schema.grid.front_cols, 48);
            assert!(!schema.fields[0].items.is_empty());
        }
    }

    #[test]
    fn test_matrix2d_roundtrip() {
        let mut m = Matrix2D::new(64, 48, true);
        m.set(SheetSide::Front, 10, 5, 212);
        m.set(SheetSide::Back, 63, 47, 189);
        assert_eq!(m.get(SheetSide::Front, 10, 5), 212);
        assert_eq!(m.get(SheetSide::Back, 63, 47), 189);
        assert_eq!(m.get(SheetSide::Front, 99, 99), 0);
    }
}
