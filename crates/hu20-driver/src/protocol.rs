//! Protocol packet definitions and safe parsing for SecuGen HU20 USB commands.

use thiserror::Error;

/// SecuGen USB Vendor ID.
pub const SECUGEN_VID: u16 = secugen_hu20_rs::SECUGEN_VID;
/// SecuGen Hamster Pro 20 (HU20) Product ID.
pub const HU20_PID: u16 = secugen_hu20_rs::HU20_PID;

/// Vendor control request: start high-speed image stream (`0x03`).
pub const START_IMAGE_REQUEST: u8 = 0x03;
/// Virtual driver trigger opcode: capture with closed-loop auto-exposure (`0x04`).
pub const START_SMART_IMAGE_REQUEST: u8 = 0x04;
/// Vendor control request: stop image stream (`0x02`).
pub const STOP_IMAGE_REQUEST: u8 = 0x02;
/// Vendor control request: set optical LED (`0x11`).
pub const SET_LED_REQUEST: u8 = 0x11;
/// Vendor control request: reset transport (`0x14`).
pub const RESET_DEVICE_REQUEST: u8 = 0x14;
/// Vendor control request: read 30-byte UTF-16LE serial number (`0x25`).
pub const READ_SERIAL_REQUEST: u8 = 0x25;
/// Bulk-IN endpoint for HU20 sensor frame stream.
pub const BULK_IN_ENDPOINT: u8 = 0x82;

/// HU20 USB command opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandOpcode {
    StopImage = 0x02,
    StartImage = 0x03,
    StartSmartImage = 0x04,
    SetLed = 0x11,
    ResetDevice = 0x14,
    ReadSerial = 0x25,
}

impl CommandOpcode {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x02 => Some(Self::StopImage),
            0x03 => Some(Self::StartImage),
            0x04 => Some(Self::StartSmartImage),
            0x11 => Some(Self::SetLed),
            0x14 => Some(Self::ResetDevice),
            0x25 => Some(Self::ReadSerial),
            _ => None,
        }
    }
}

/// Structured USB command sent from host to scanner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbCommand {
    pub opcode: CommandOpcode,
    pub parameter: Option<u8>,
}

impl UsbCommand {
    pub fn new(opcode: CommandOpcode, parameter: Option<u8>) -> Self {
        Self { opcode, parameter }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self.parameter {
            Some(p) => vec![self.opcode as u8, p],
            None => vec![self.opcode as u8],
        }
    }
}

/// Decoded USB response packet from device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponsePacket {
    pub status_code: u8,
    pub payload: Vec<u8>,
}

/// Errors occurring during USB protocol parsing.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("packet truncated: expected at least {expected} bytes, got {actual}")]
    Truncated { expected: usize, actual: usize },

    #[error("invalid header byte: 0x{0:02X}")]
    InvalidHeader(u8),

    #[error("checksum mismatch: expected 0x{expected:02X}, calculated 0x{calculated:02X}")]
    ChecksumMismatch { expected: u8, calculated: u8 },

    #[error("unknown opcode byte: 0x{0:02X}")]
    UnknownOpcode(u8),
}

/// Parses a raw USB response packet fallibly without panics or unchecked indexing.
pub fn parse_response_packet(buf: &[u8]) -> Result<ResponsePacket, ProtocolError> {
    let header = buf.first().copied().ok_or(ProtocolError::Truncated {
        expected: 1,
        actual: 0,
    })?;

    if header == 0xFF {
        return Err(ProtocolError::InvalidHeader(header));
    }

    if buf.len() < 2 {
        return Err(ProtocolError::Truncated {
            expected: 2,
            actual: buf.len(),
        });
    }

    let status_code = *buf.first().unwrap_or(&0);
    let payload_len = *buf.get(1).unwrap_or(&0) as usize;

    let total_expected = 2 + payload_len + 1;
    if buf.len() < total_expected {
        return Err(ProtocolError::Truncated {
            expected: total_expected,
            actual: buf.len(),
        });
    }

    let payload = buf
        .get(2..2 + payload_len)
        .ok_or(ProtocolError::Truncated {
            expected: 2 + payload_len,
            actual: buf.len(),
        })?
        .to_vec();

    let expected_checksum = *buf.get(2 + payload_len).ok_or(ProtocolError::Truncated {
        expected: total_expected,
        actual: buf.len(),
    })?;

    let calculated_checksum = payload
        .iter()
        .fold(status_code ^ (payload_len as u8), |acc, &b| acc ^ b);
    if expected_checksum != calculated_checksum {
        return Err(ProtocolError::ChecksumMismatch {
            expected: expected_checksum,
            calculated: calculated_checksum,
        });
    }

    Ok(ResponsePacket {
        status_code,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_packet() {
        let packet = vec![0x00, 0x02, 0xAA, 0xBB, 0x02 ^ 0xAA ^ 0xBB];
        let parsed = parse_response_packet(&packet).expect("valid packet");
        assert_eq!(parsed.status_code, 0x00);
        assert_eq!(parsed.payload, vec![0xAA, 0xBB]);
    }

    #[test]
    fn test_parse_truncated_never_panics() {
        assert!(matches!(
            parse_response_packet(&[]),
            Err(ProtocolError::Truncated { .. })
        ));
        assert!(matches!(
            parse_response_packet(&[0x00, 0x05, 0x01]),
            Err(ProtocolError::Truncated { .. })
        ));
    }

    #[test]
    fn test_parse_invalid_header() {
        assert_eq!(
            parse_response_packet(&[0xFF, 0x00, 0x00]),
            Err(ProtocolError::InvalidHeader(0xFF))
        );
    }
}
