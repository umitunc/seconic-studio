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
            "01" | "Q1" => Self::HopperEmpty,
            "02" | "Q2" => Self::DoubleFeed,
            "03" | "Q3" => Self::PaperJam,
            "04" | "Q4" => Self::SkewError,
            "05" | "Q5" => Self::CoverOpen,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_standard_and_sr6000_q_codes() {
        assert_eq!(HardwareCondition::from_status_code("00"), HardwareCondition::Ready);
        assert_eq!(HardwareCondition::from_status_code("01"), HardwareCondition::HopperEmpty);
        assert_eq!(HardwareCondition::from_status_code("Q1"), HardwareCondition::HopperEmpty);
        assert_eq!(HardwareCondition::from_status_code("Q2"), HardwareCondition::DoubleFeed);
        assert_eq!(HardwareCondition::from_status_code("Q3"), HardwareCondition::PaperJam);
    }
}
