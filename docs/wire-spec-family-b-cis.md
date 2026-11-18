# Sekonic Family B (CIS) Wire Protocol Specification (`SkDvSr55.dll` + `SkRgOmr.dll`)

**Scope:** `SR-55D` (`VID_0A41`, `PID_2000`) and CIS/Hybrid variants (`PID_2002`, `PID_2003`, `PID_3005`, `PID_5000`)  
**Verified Source Binaries:**
- `SkDvSr55.dll` (`188,416` bytes, SHA-256 `68594b30831adab8`)
- `SkRgOmr.dll` (`177,664` bytes, SHA-256 `348a9ad1bbd6476e`)
- `SkCommIo.dll` (`126,976` bytes, SHA-256 `f7ed656556426228`)

---

## 1. Dual-Pipe USB Architecture (`SkCommIo.dll`)

- **WinUSB Interface GUID:** `{dc78e193-e752-4b23-bc0b-4da9a52ec825}` (`SkCommIo.dll` RVA `0x17BD8`)
- **Pipe 0 / Control Bulk OUT & IN:** ASCII/Binary `[STX]...[ETX]` command & ACK/Response channel (`SkCd_SendData @ RVA 0x8C40`, `SkCd_RecvData @ RVA 0x8D30`, `SkCd_CmdAckRes @ RVA 0x8B70`).
- **Pipe 1 / Image Bulk IN (`SkCommIo_RecvPipe`):** Dedicated high-speed bulk IN stream delivering raw 8-bit grayscale CIS scanlines after `SF1` (`SkCd_ReqScan_Start`) is issued.

---

## 2. Verified Control Command Set (`SkDvSr55.dll` `.rdata`)

| Command | Purpose | Parameters |
| :--- | :--- | :--- |
| `IS%02d` | Initialize scanner state | Mode (`00`..`99`) |
| `RS07,%c%c%c` | Software reset | 3-char reset flags |
| `BRw,%01d` / `BRr` | Set/Get Back-side (Duplex) reading | `0` = Simplex, `1` = Duplex |
| `FLw,%04d,%04d,%03d,%03d` | Set form length bounds (mm/steps) | `max_len, min_len, ...` |
| `LCw,%01d,%01d` / `LCr,%01d` | Set CIS Dropout LED color | Face (`0`=Front, `1`=Back), Color (`Red`, `Green`, `Blue`, `GrayS`) |
| `REw,%01d,%01d` / `REr` | Set scan resolution | `300` DPI / `600` DPI |
| `HRw,%01d,%01d,%01d,0,%04d` | Set CIS LED On-Time | Face, Color, Reso, Time |
| `SSw,%01d,%01d,%01d,%02d,` | Upload CIS Shading / Base calibration | Face, Color, Reso, Bank |
| `CS0` / `CS1,%04d` / `CS2` | Calibration Start / Feed / Exit | White reference calibration |
| `SF1` | Start scan & stream image | Triggers Bulk IN image pipe |
| `ER%1.1d` | Eject sheet | `0` = Main / `1` = Sub |

---

## 3. CIS Image-to-OMR Pipeline (`SkRgOmr.dll`)

1. **Shading Normalization:** Raw 8-bit grayscale pixels are normalized per column against stored dark/white calibration arrays (`SkDv_GetScanHeadAnalogDataShading`).
2. **Timing Mark & Skew Tracking:** Left/right timing tracks are detected on the 8-bit image to compensate for feed speed jitter and ±3° paper skew.
3. **ROI Mark Density (`0..255`):** Each bubble ROI computes `255 - mean_luminance` so dark pencil marks produce high `u8` values (`180..255`), matching Family A's `OMR_GetMarkAD` scale.
