use thiserror::Error;

/// Unified error type for all Sekonic scanner, protocol, grading, and sync operations.
#[derive(Debug, Error)]
pub enum SekonicError {
    #[error("device not connected")]
    NotConnected,

    #[error("transport I/O error: {0}")]
    Transport(String),

    #[error("operation timed out after {timeout_ms} ms")]
    Timeout { timeout_ms: u64 },

    #[error("protocol framing or CRC mismatch: {0}")]
    Protocol(String),

    #[error("scanner rejected command with NAK status 0x{status:02X}")]
    Nak { status: u8 },

    #[error("hardware fault ({code}): {message}")]
    HardwareFault { code: String, message: String },

    #[error("invalid form schema: {0}")]
    InvalidSchema(String),

    #[error("legacy .bldm/.fmt parse error: {0}")]
    LegacyFormat(String),

    #[error("sync/storage error: {0}")]
    Sync(String),
}
