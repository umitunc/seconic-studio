#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackerTarget {
    Main,
    Sub,
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

pub fn cmd_feed_sheet() -> &'static [u8] {
    b"SF"
}

pub fn cmd_eject(target: StackerTarget) -> &'static [u8] {
    match target {
        StackerTarget::Main => b"ER0",
        StackerTarget::Sub => b"ER1",
    }
}

pub fn cmd_set_columns(cols: u16) -> Vec<u8> {
    format!("NC1,{cols:02}").into_bytes()
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
