# Sekonic Studio (dijiScan)

A clean-room, next-generation **Optical Mark Reading (OMR) & CIS Scanner Platform** built in Rust, TypeScript, WebAssembly, and Tauri v2.

Sekonic Studio replaces legacy 32-bit Windows-only drivers and VB5/VB6 tools (`OMRAPI.dll`, `SkCommIo.dll`, `SekOku.exe`) with a memory-safe, cross-platform architecture that runs natively on **Windows (7/8.1/10/11), macOS, Linux**, and directly in modern browsers via **WebUSB + WebAssembly**.

---

## Key Features

- **Clean-Room Protocol Implementation**: Native async drivers for both **Family A (Classic OMR)** (`STX/ETX` + CRC16) and **Family B (CIS Image OMR)** (`SR-55D` shading calibration & software mark extraction).
- **High-Throughput Pipelined Scanning**: Pre-allocated buffers and pipelined I/O designed for high-speed feeders (up to 6,500 sheets/hour on `SR-6500`).
- **0–255 Analog Erasure Recovery & Grading**: Confidence scoring, double-mark detection, erasure recovery, and sheet validation (`sekonic-grader`).
- **Legacy Form Compatibility**: JSON form schema with `.bldm` / `.fmt` conversion support (`sekonic-schema`).
- **Offline-First Sync**: SQLite WAL-backed offline queue with WebSocket/REST synchronization (`sekonic-sync`).
- **Hybrid Local Agent & Web UI**:
  - `sekonic-agent`: Lightweight headless local WebSocket bridge (`ws://127.0.0.1:17441`) supporting universal WinUSB and 32-bit/64-bit Windows systems.
  - `sekonic-bridge`: Tauri v2 desktop app & React + `sheetgl` browser interface (`http://localhost:5179/scan`).
- **Built-in Hardware Simulator**: Virtual `SR-6500` and `SR-55D` emulators with synthetic sheet generation (`sekonic-sim`) for testing without physical hardware.

---

## Supported Hardware (`VID_0A41`)

| Model | USB PID | Protocol Family |
| :--- | :--- | :--- |
| **SR-2300 / SR-5500** | `0x0002` | Family A (Classic OMR) |
| **SR-3500 / SR-6000 / SR-6500** | `0x1004` | Family A (Classic OMR) |
| **SR-1800 / SR-1800EX** | `0x1008` | Family A (Classic OMR) |
| **SR-55D** | `0x2000` | Family B (CIS Image OMR) |
| **SR-45C / SMF / SC-1500 / SP-100** | `0x2002` / `0x2003` / `0x3005` / `0x5000` | WinUSB INF supported |

A universal WinUSB driver definition is included at `apps/sekonic-bridge/drivers/sekonic-winusb.inf`.

---

## Repository Structure

```text
.
├── crates/
│   ├── sekonic-schema        # JSON form schema, error types, and .bldm/.fmt converter
│   ├── sekonic-transport     # USB Bulk (nusb / WebUSB), Serial, and InMemoryTransport
│   ├── sekonic-proto-omr     # Family A STX/ETX async & pipelined OMR driver
│   ├── sekonic-proto-cis     # Family B SR-55D CIS protocol, shading & image OMR engine
│   ├── sekonic-grader        # 0-255 analog erasure recovery, confidence & grading engine
│   ├── sekonic-sim           # Virtual SR-6500 & SR-55D emulators + synthetic sheets
│   ├── sekonic-sync          # SQLite WAL offline queue & WebSocket/REST sync client
│   └── sekonic-agent         # Headless local WebSocket bridge agent (port 17441)
├── packages/
│   └── sekonic-web-sdk       # @kurum/sekonic-web-sdk (WASM + TypeScript SDK)
├── apps/
│   └── sekonic-bridge        # Tauri v2 desktop bridge + React/Vite/SheetGL Web UI
├── docs/                     # Wire protocol specs & implementation plan
├── tools/                    # Reverse-engineering verification scripts & release helpers
├── PROGRESS.md               # Work-package (WP-01 to WP-12) progress log
├── start-local.bat           # Local dev launcher (agent + web UI)
└── deploy.bat                # Version bump, build, and deployment script
```

---

## Getting Started

### Prerequisites

- **Rust** (stable toolchain, 2021 edition)
- **Node.js** & **npm**
- *(Optional, for WASM)* `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)

### Quick Start (Windows)

Run the local development launcher to start both the local WebSocket agent (`ws://127.0.0.1:17441`) and the web UI (`http://localhost:5179/scan`):

```bat
start-local.bat
```

### Manual Development Setup

1. **Start the local bridge agent:**
   ```bash
   cargo run -p sekonic-agent
   ```

2. **Build the TypeScript Web SDK:**
   ```bash
   npm --prefix packages/sekonic-web-sdk run build
   ```

3. **Start the Web UI:**
   ```bash
   npm --prefix apps/sekonic-bridge install
   npm --prefix apps/sekonic-bridge run dev
   ```

---

## Testing & Verification

Run the full Rust workspace test suite (including end-to-end simulator stress tests):

```bash
cargo test --workspace --release
```

Typecheck and build the Web SDK and Web UI:

```bash
npm --prefix packages/sekonic-web-sdk run typecheck
npm --prefix apps/sekonic-bridge run build
```

---

## Documentation

- [Implementation Plan](docs/sekonic_implementation_plan.md)
- [Family A Wire Specification (Classic OMR)](docs/wire-spec-family-a.md)
- [Family B Wire Specification (CIS / SR-55D)](docs/wire-spec-family-b-cis.md)
- [Work Package Progress](PROGRESS.md)

---

## License

Dual-licensed under **MIT** or **Apache-2.0** (`MIT OR Apache-2.0`). See `Cargo.toml` for workspace metadata.

