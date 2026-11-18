use sekonic_schema::form_schema::DropoutColor;

pub fn cmd_init(mode: u8) -> Vec<u8> {
    format!("\x02IS{mode:02}\x03").into_bytes()
}

pub fn cmd_set_back_side(duplex: bool) -> Vec<u8> {
    format!("\x02BRw,{}\x03", u8::from(duplex)).into_bytes()
}

pub fn cmd_set_light_color(face: u8, color: DropoutColor) -> Vec<u8> {
    let code = match color {
        DropoutColor::None => 0,
        DropoutColor::Red => 1,
        DropoutColor::Green => 2,
        DropoutColor::Blue => 3,
    };
    format!("\x02LCw,{face},{code}\x03").into_bytes()
}

pub fn cmd_set_resolution(face: u8, dpi_300: bool) -> Vec<u8> {
    let code = if dpi_300 { 0 } else { 1 };
    format!("\x02REw,{face},{code}\x03").into_bytes()
}

pub fn cmd_scan_start() -> &'static [u8] {
    b"\x02SF1\x03"
}

pub fn cmd_eject(sub_stacker: bool) -> Vec<u8> {
    format!("\x02ER{}\x03", u8::from(sub_stacker)).into_bytes()
}
