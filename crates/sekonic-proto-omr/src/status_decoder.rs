#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareCondition {
    Ready,
    HopperEmpty,
    DoubleFeed,
    PaperJam,
    SkewError,
    CoverOpen,
    Unknown(u8),
}

impl HardwareCondition {
    /// Decodes the 2-char status code returned in `SF` / `ST` responses.
    pub fn from_status_code(code: &str) -> Self {
        match code {
            "00" => Self::Ready,
            "01" => Self::HopperEmpty,
            "02" => Self::DoubleFeed,
            "03" => Self::PaperJam,
            "04" => Self::SkewError,
            "05" => Self::CoverOpen,
            other => Self::Unknown(other.bytes().next().unwrap_or(0)),
        }
    }

    pub fn message_tr(&self) -> &'static str {
        match self {
            Self::Ready => "Hazır",
            Self::HopperEmpty => "Besleme tepsisi boş (Kağıt bitti)",
            Self::DoubleFeed => "Çift kağıt çekme hatası (Double Feed)",
            Self::PaperJam => "Kağıt sıkışması (Jam)",
            Self::SkewError => "Kağıt eğiklik hatası (Skew)",
            Self::CoverOpen => "Cihaz kapağı açık",
            Self::Unknown(_) => "Bilinmeyen donanım durumu",
        }
    }
}
