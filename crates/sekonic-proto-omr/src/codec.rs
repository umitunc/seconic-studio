use bytes::{BufMut, BytesMut};
use sekonic_schema::error::SekonicError;

pub const STX: u8 = 0x02;
pub const ACK: u8 = 0x06;
pub const NAK: u8 = 0x15;

/// Reflected CRC-16/CCITT (poly 0x8408, init 0xFFFF, xorout 0xFFFF) verified from `OMRAPI.dll`.
#[inline]
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        crc ^= u16::from(b);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0x8408;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF
}

/// Encodes `[STX (1B)] [LEN_BE (2B)] [PAYLOAD] [CRC16_BE (2B)]` into a pre-allocated buffer.
pub fn encode_frame(payload: &[u8], out: &mut BytesMut) -> Result<(), SekonicError> {
    let len = u16::try_from(payload.len())
        .map_err(|_| SekonicError::Protocol("payload exceeds u16 frame length".into()))?;
    out.clear();
    out.reserve(payload.len() + 5);
    out.put_u8(STX);
    out.put_u16(len);
    out.put_slice(payload);
    let crc = crc16_ccitt(out);
    out.put_u16(crc);
    Ok(())
}

/// Parses an ACK (`0x06`) or NAK (`0x15, nak_status`).
pub fn decode_ack_nak(buf: &[u8]) -> Result<(), SekonicError> {
    match buf {
        [ACK, ..] => Ok(()),
        [NAK, status, ..] => Err(SekonicError::Nak { status: *status }),
        _ => Err(SekonicError::Protocol(format!(
            "unexpected ACK/NAK response: {:02X?}",
            buf
        ))),
    }
}
