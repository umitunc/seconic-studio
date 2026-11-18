# Sekonic Family A Wire Protocol Specification (`OMRAPI.dll` Clean-Room Spec)

**Scope:** `SR-1800`, `SR-1800EX`, `SR-2300`, `SR-3500`, `SR-5500`, `SR-6000`, `SR-6500`  
**Verified Source Binary:** `OMRAPI.dll` (`122,880` bytes, SHA-256 `e91cb0851c77ccd2`, `ImageBase = 0x10000000`) & `OMRAPI.DLL` (`90,112` bytes, SHA-256 `02c9244c8d024c5f`).

---

## 1. Transport Layer

### 1.1. USB Bulk (`SkCommIo.dll` / `WINUSB.DLL`)
- **Vendor ID:** `0x0A41`
- **Product IDs:**
  - `0x0002`: SR-2300 / SR-5500
  - `0x1004`: SR-3500 / SR-6000 / SR-6500
  - `0x1008`: SR-1800 / SR-1800EX
- **Device Interface GUIDs:**
  - WinUSB: `{c4fa33ee-2253-4394-8a32-389e13b03fd3}` (`SkCommIo.dll` RVA `0x17BC8`)
  - Legacy WDM (`\\.\UsbSkOmr%d`): `{b637bf20-a911-4127-b57f-303fbf314a7e}` (`OMRAPI.DLL` RVA `0xC120`)
- **Endpoints:** 1× Bulk OUT (Host → Scanner), 1× Bulk IN (Scanner → Host).

### 1.2. RS-232 Serial (`90K::OMR_OpenDeviceSerial @ RVA 0x51F0`)
- Disassembly of DCB initialization (`0x10005261..0x100052AC`):
  - `ByteSize = 8` (`mov byte ptr [esp + 0x32], 8`)
  - `Parity = 0` (`NOPARITY`, `bl = 0`)
  - `StopBits = 0` (`ONESTOPBIT`, `bl = 0`)
  - `BaudRate`: caller-supplied (standard `9600` / `19200` / `38400` / `115200`).
  - Default timeouts: `1000 ms` (`0x3e8` at `0x1000522b`).

---

## 2. USB Packet Framing & CRC-16 (`122K::_OMR_SendCommand @ RVA 0x3410`)

Verified from `0x10003453..0x100034BA` (USB mode `[0x1001ac24] == 2`):

```text
+------+--------+--------+----------------------+----------+----------+
| STX  | LEN_HI | LEN_LO | PAYLOAD (ASCII/Bin)  | CRC16_HI | CRC16_LO |
| 0x02 | 1 byte | 1 byte | N bytes              | 1 byte   | 1 byte   |
+------+--------+--------+----------------------+----------+----------+
|<------------------ CRC-16 Scope (N + 3 bytes) -------------------->|
```

- **Total Frame Length:** `N + 5` bytes.
- **Length Field:** Big-endian `u16` (`LEN_HI = N >> 8`, `LEN_LO = N & 0xFF`).
- **CRC-16 Algorithm (`RVA 0x1010`):**
  - Lookup table starts with `0x0000, 0x1189, 0x2312, 0x329b, 0x4624, 0x57ad, 0x6536, 0x74bf, ...`
  - Reflected **CRC-16/CCITT (Poly `0x8408` / `0x1021` reversed)**.
  - Computed over `[STX, LEN_HI, LEN_LO, PAYLOAD...]` (`N + 3` bytes) with initial value `0xFFFF` (or `0x0000` with final XOR `0xFFFF`, verified by `push -1` at `0x1000345f`).
  - Written big-endian: `buf[N + 3] = CRC_HI`, `buf[N + 4] = CRC_LO`.

---

## 3. ACK / NAK Handshake (`122K::_OMR_ReceiveACKorNAK @ RVA 0x3350`)

After every command frame, the host reads from Bulk IN:
- **1-byte `0x06` (`ACK`):** Command accepted (`return 0`).
- **2-byte `[0x15, nak_code]` (`NAK`):** Command rejected; `nak_code` is saved and exposed via `OMR_GetNakST` (`return 6`).

---

## 4. Command & Response Layouts

### 4.1. Response Validation (`122K::_OMR_GetResponse @ RVA 0x3530` & `0x34D0`)
- Response echoes the 2-byte command mnemonic (e.g. `"AD"`, `"MD"`, `"ST"`), followed by 2-byte status (`0x1001ad35`) and 2-byte warning (`0x1001ad39`).

### 4.2. Analog Mark Density Matrix (`OMR_GetMarkAD @ RVA 0x5240`)
- Sends `AD%01d` (`0` = Front side, `1` = Back side).
- Response length must be `>= 3` bytes; first 2 bytes are header/command prefix, remaining `len - 2` bytes (`[0x1001dd60] + 2`) are raw **`u8` optical density values (`0..255`)** per cell in column/row scan order.

### 4.3. Binary Mark Matrix (`OMR_GetMarks @ RVA 0x3FF0`)
- Sends `MD%01d` (`0` = Front side, `1` = Back side).
- Response contains ASCII `'0'`/`'1'` bit-digits separated by `','` (`0x2c`) per row/column; decoded by subtracting `0x30` (`sub dl, 0x30` at `0x10004130`).

### 4.4. Core Command Reference
- `IS`: Initial setting
- `MN`: Get machine name (`SR-1800`, `SR-3500`, `SR-6000`, `SR-6500`)
- `ST`: Get 2-byte status + warning flags
- `SF`: Feed sheet
- `ER0` / `ER1`: Eject to **Main Stacker (`0`)** or **Sub-Stacker (`1`)**
- `NC1,%02d`: Set number of columns to read
- `WB1,%d,%d`: Enable/disable front & back sensors (Duplex)
- `BC1,1,1` / `BD1`: Enable barcode reader / read barcode string
- `PR1,8,1` / `PR1,1,%01d,%s`: Enable endorser printer / set print text
