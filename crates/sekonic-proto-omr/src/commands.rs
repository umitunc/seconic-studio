#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackerTarget {
    Main,
    Sub,
}

pub fn cmd_cancel_error() -> &'static [u8] {
    b"CE"
}

pub fn cmd_reset() -> &'static [u8] {
    b"SR"
}

pub fn cmd_init() -> &'static [u8] {
    b"IS"
}

pub fn cmd_machine_name() -> &'static [u8] {
    b"MN"
}

pub fn cmd_device_info() -> &'static [u8] {
    b"DI"
}

pub fn cmd_status() -> &'static [u8] {
    b"ST"
}

pub fn cmd_move_hopper(up: bool) -> &'static [u8] {
    if up {
        b"HU1"
    } else {
        b"HU0"
    }
}

pub fn cmd_feed_sheet() -> &'static [u8] {
    b"SF"
}

/// Verified from `OMRAPI.dll::OMR_EjectSheet @ 0x10005440` (`1 <= target <= 4`):
/// `ER1` = Main Stacker, `ER2` = Sub Stacker.
pub fn cmd_eject(target: StackerTarget) -> &'static [u8] {
    match target {
        StackerTarget::Main => b"ER1",
        StackerTarget::Sub => b"ER2",
    }
}

/// Verified from `OMRAPI.dll::OMR_SetReadingMethod @ 0x10005ef2` and `SekOku.exe @ 0x4b6da4`:
/// `RM1,3` selects 16-level optical density mode (`0..15`) for `MD0`/`MD1`.
pub fn cmd_set_reading_method_16level() -> &'static [u8] {
    b"RM1,3"
}

/// Verified from `OMRAPI.dll::OMR_SetNumberOfColumnsToRead @ 0x10005dd0` (`1..=48`).
pub fn cmd_set_columns(cols: u16) -> Vec<u8> {
    format!("NC1,{cols:02}").into_bytes()
}

pub fn cmd_set_back_sensor(enabled: bool) -> &'static [u8] {
    if enabled {
        b"BR1,1"
    } else {
        b"BR1,0"
    }
}

pub fn cmd_set_duplex(front: bool, back: bool) -> Vec<u8> {
    format!("WB1,{},{}", u8::from(front), u8::from(back)).into_bytes()
}

pub fn cmd_set_barcode_unit(enabled: bool) -> &'static [u8] {
    if enabled {
        b"BC1,1,1"
    } else {
        b"BC1,1,0"
    }
}

pub fn cmd_get_barcode() -> &'static [u8] {
    b"BD1"
}

pub fn cmd_get_mark_ad(back_side: bool) -> &'static [u8] {
    if back_side {
        b"AD1"
    } else {
        b"AD0"
    }
}

pub fn cmd_get_marks_md(back_side: bool) -> &'static [u8] {
    if back_side {
        b"MD1"
    } else {
        b"MD0"
    }
}
