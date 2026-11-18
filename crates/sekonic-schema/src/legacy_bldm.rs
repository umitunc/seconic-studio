use crate::error::SekonicError;
use crate::form_schema::{
    BubbleCoord, BubbleGroup, DropoutColor, FieldDefinition, FieldType, GridGeometry,
    OmrFormSchema, SheetSide, ThresholdPolicy,
};

/// Parses a legacy Sekonic `.bldm` definition into a modern `OmrFormSchema`.
///
/// Header format: `<rows>=<cols>=<timing_or_pad>=<dir>=<multi_char>=<blank_char>=`
/// Item format:   `<idx>=<start_row>=<start_col>=<end_col>=<kind>=<dir>=<label>=<opts>=`
pub fn parse_bldm(form_id: &str, content: &str) -> Result<OmrFormSchema, SekonicError> {
    let mut lines = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty());

    let header = lines
        .next()
        .ok_or_else(|| SekonicError::LegacyFormat("empty .bldm file".into()))?;
    let hparts: Vec<&str> = header.split('=').collect();
    if hparts.len() < 4 {
        return Err(SekonicError::LegacyFormat(format!(
            "invalid .bldm header: '{header}'"
        )));
    }

    let front_rows: u16 = hparts[0].trim().parse().map_err(|_| {
        SekonicError::LegacyFormat(format!("invalid row count '{}'", hparts[0]))
    })?;
    let front_cols: u16 = hparts[1].trim().parse().map_err(|_| {
        SekonicError::LegacyFormat(format!("invalid col count '{}'", hparts[1]))
    })?;

    let mut groups = Vec::new();
    for line in lines {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() < 7 {
            return Err(SekonicError::LegacyFormat(format!(
                "invalid .bldm line: '{line}'"
            )));
        }
        let item_idx: u16 = parts[0].trim().parse().map_err(|_| {
            SekonicError::LegacyFormat(format!("invalid item index '{}'", parts[0]))
        })?;
        let row_1based: u16 = parts[1].trim().parse().map_err(|_| {
            SekonicError::LegacyFormat(format!("invalid row '{}'", parts[1]))
        })?;
        let col_start_1based: u16 = parts[2].trim().parse().map_err(|_| {
            SekonicError::LegacyFormat(format!("invalid start col '{}'", parts[2]))
        })?;
        let col_end_1based: u16 = parts[3].trim().parse().map_err(|_| {
            SekonicError::LegacyFormat(format!("invalid end col '{}'", parts[3]))
        })?;

        if row_1based == 0 || col_start_1based == 0 || col_end_1based < col_start_1based {
            return Err(SekonicError::LegacyFormat(format!(
                "invalid 1-based coordinates in '{line}'"
            )));
        }

        let label_prefix = parts[6].trim();
        let row = row_1based - 1;
        let mut options = Vec::new();
        for (offset, col1) in (col_start_1based..=col_end_1based).enumerate() {
            let opt_label = if col_start_1based == col_end_1based {
                label_prefix.to_string()
            } else {
                format!("{label_prefix}{}", offset + 1)
            };
            options.push(BubbleCoord {
                label: opt_label,
                row,
                col: col1 - 1,
            });
        }
        groups.push(BubbleGroup {
            index: item_idx,
            options,
        });
    }

    let schema = OmrFormSchema {
        schema_version: 1,
        form_id: form_id.to_string(),
        form_name: format!("Imported {form_id}"),
        duplex: false,
        dropout_color: DropoutColor::Red,
        grid: GridGeometry {
            front_rows,
            front_cols,
            back_rows: 0,
            back_cols: 0,
            width_mm: 210.0,
            height_mm: 297.0,
        },
        threshold_policy: ThresholdPolicy::default(),
        fields: vec![FieldDefinition {
            key: "imported_fields".to_string(),
            field_type: FieldType::CustomCode,
            side: SheetSide::Front,
            required_for_valid_sheet: false,
            items: groups,
        }],
    };
    schema.validate()?;
    Ok(schema)
}
