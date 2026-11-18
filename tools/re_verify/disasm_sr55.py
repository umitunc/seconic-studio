from pathlib import Path
import pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

ROOT = Path(__file__).resolve().parents[2]

TARGETS = {
    "SkDvSr55.dll": [
        "SkCd_SendData",
        "SkCd_RecvData",
        "SkCd_CmdAckRes",
        "SkCd_ReqScan_Start",
        "SkCd_ReqScan_Recv",
        "SkCd_ReqScan_RecvImage",
        "SkCd_ReqCalibrationStart",
        "SkCd_SetLightColor",
        "SkCd_SetResolution",
    ],
    "SkRgOmr.dll": [
        "SkRg_Open",
        "SkRg_ReqInit",
        "SkRg_ReqRecog",
        "SkRg_GetMarkLevel",
        "SkRg_GetFormResult",
    ],
}


def main() -> int:
    md = Cs(CS_ARCH_X86, CS_MODE_32)
    for dll_name, funcs in TARGETS.items():
        matches = list((ROOT / ".cache" / "extracted_bins").rglob(dll_name))
        if not matches:
            print(f"ERROR: {dll_name} not found")
            return 1
        dll_path = matches[0]
        pe = pefile.PE(str(dll_path))
        image_base = pe.OPTIONAL_HEADER.ImageBase
        img = pe.get_memory_mapped_image()
        exports = {
            exp.name.decode("ascii"): exp.address
            for exp in pe.DIRECTORY_ENTRY_EXPORT.symbols
            if exp.name
        }
        print(f"\n=== Disassembly of {dll_name} (ImageBase={hex(image_base)}) ===")
        for fn in funcs:
            rva = exports.get(fn)
            if rva is None:
                print(f"--- {fn}: NOT FOUND ---")
                continue
            va = image_base + rva
            code = img[rva : rva + 240]
            print(f"\n--- {fn} @ RVA {hex(rva)} ---")
            for insn in md.disasm(code, va):
                print(f"  {hex(insn.address)}: {insn.mnemonic:8s} {insn.op_str}")
                if insn.mnemonic in ("ret", "retn") and insn.address > va + 8:
                    break
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
