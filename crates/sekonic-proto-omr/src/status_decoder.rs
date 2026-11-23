#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareCondition {
    Ready,
    HopperEmpty,
    DoubleFeed,
    PaperJam,
    SkewError,
    CoverOpen,
    HopperStopped,
    FeedTimeout,
    TimingMarkError,
    SensorError,
    Unknown(u8),
}

impl HardwareCondition {
    /// Decodes the 2-char status code returned in `SF` / `ST` responses.
    /// Verified directly against `OMRAPI.dll::OMR_GetLastError @ 0x10001e40..0x1000239e`
    /// and `OMRAPI.dll::OMR_FormatMessage @ 0x10002a80`.
    pub fn from_status_code(code: &str) -> Self {
        match code {
            // Normal / non-fatal option unit unconnected (`P1..P4`)
            "00" | "@@" | "0@" | "@0" | "P1" | "P2" | "P3" | "P4" => Self::Ready,
            // Sheet Empty (`Q1` on real SR-1800/3500/6000/6500; `01` on simulator)
            "01" | "Q1" => Self::HopperEmpty,
            // Double Feed Error (`Q2` on real hardware; `02` on simulator)
            "02" | "Q2" => Self::DoubleFeed,
            // Skew Error (`Q3` = Left End Skew, `Q4` = Mark Skew; `04` on simulator)
            "04" | "Q3" | "Q4" => Self::SkewError,
            // Cover Open (`G1` = Main Body, `G2` = Select Stacker; `05` on simulator)
            "05" | "G1" | "G2" => Self::CoverOpen,
            // Paper Jam / No Feed / Form Left In Path (`H1..H4`, `I1..I3`, `T1..T6`; `03` on simulator)
            "03" | "H1" | "H2" | "H3" | "H4" | "I1" | "I2" | "I3" | "T1" | "T2" | "T3"
            | "T4" | "T5" | "T6" => Self::PaperJam,
            // Hopper & feed transport faults (`R1..R3`)
            "R1" => Self::HopperStopped,
            "R2" | "R3" => Self::FeedTimeout,
            // Timing Mark Error (`R4`)
            "R4" => Self::TimingMarkError,
            // Sensor white/black/stain/setting errors (`R5`, `S1..S4`)
            "R5" | "S1" | "S2" | "S3" | "S4" => Self::SensorError,
            other => Self::Unknown(other.bytes().next().unwrap_or(0)),
        }
    }

    pub fn message_tr(&self) -> &'static str {
        match self {
            Self::Ready => "Hazır",
            Self::HopperEmpty => "Besleme tepsisi boş (Kağıt bitti)",
            Self::DoubleFeed => "Çift kağıt çekme hatası",
            Self::PaperJam => "Kağıt sıkışması veya besleme hatası",
            Self::SkewError => "Kağıt eğiklik hatası",
            Self::CoverOpen => "Cihaz kapağı açık",
            Self::HopperStopped => "Besleme tablası durdu",
            Self::FeedTimeout => "Kağıt çekme zaman aşımı",
            Self::TimingMarkError => "Referans çizgisi okuma hatası",
            Self::SensorError => "Optik sensör kalibrasyon/kirlilik hatası",
            Self::Unknown(_) => "Bilinmeyen donanım durumu",
        }
    }
}
