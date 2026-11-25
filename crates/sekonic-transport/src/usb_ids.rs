use crate::traits::DeviceFamily;

pub const VID_SEKONIC: u16 = 0x0A41;

pub const PID_SR2300_5500: u16 = 0x0002;
pub const PID_SR3500_6000_6500: u16 = 0x1004;
pub const PID_SR1800: u16 = 0x1008;
pub const PID_SR55D: u16 = 0x2000;
pub const PID_SR45C_38C: u16 = 0x2002;
pub const PID_SMF_HYBRID: u16 = 0x2003;
pub const PID_SC1500_HYBRID: u16 = 0x3005;
pub const PID_SR55: u16 = 0x3006;
pub const PID_SP100: u16 = 0x5000;

/// Verified Windows DeviceInterfaceGUIDs from SkCommIo.dll / OMRAPI.DLL
pub const GUID_WINUSB_SR_SERIES: &str = "{c4fa33ee-2253-4394-8a32-389e13b03fd3}";
pub const GUID_WINUSB_SR55D: &str = "{dc78e193-e752-4b23-bc0b-4da9a52ec825}";
pub const GUID_LEGACY_USBSKOMR: &str = "{b637bf20-a911-4127-b57f-303fbf314a7e}";
pub const GUID_CLASS_OMR: &str = "{a43d14b6-add8-11d3-99f0-005004d38a59}";

pub fn classify_pid(pid: u16) -> Option<(DeviceFamily, &'static str)> {
    match pid {
        PID_SR2300_5500 => Some((DeviceFamily::FamilyAOmrGrid, "SEKONIC SR-2300/5500")),
        PID_SR3500_6000_6500 => Some((DeviceFamily::FamilyAOmrGrid, "SEKONIC SR-3500/6000/6500")),
        PID_SR1800 => Some((DeviceFamily::FamilyAOmrGrid, "SEKONIC SR-1800/1800EX")),
        PID_SR55D => Some((DeviceFamily::FamilyBCisImage, "SEKONIC SR-55D")),
        PID_SR45C_38C => Some((DeviceFamily::FamilyBCisImage, "SEKONIC SR-45C/38C")),
        PID_SMF_HYBRID => Some((DeviceFamily::FamilyBCisImage, "SEKONIC SMF/Hybrid")),
        PID_SC1500_HYBRID => Some((DeviceFamily::FamilyBCisImage, "SEKONIC SC-1500")),
        PID_SR55 => Some((DeviceFamily::FamilyAOmrGrid, "SEKONIC SR-55")),
        PID_SP100 => Some((DeviceFamily::FamilyBCisImage, "SEKONIC SP-100")),
        _ => None,
    }
}
