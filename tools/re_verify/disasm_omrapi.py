from pathlib import Path
import pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

ROOT = Path(__file__).resolve().parents[2]
matches = [m for m in (ROOT / ".cache" / "extracted_bins").rglob("OMRAPI.dll") if m.stat().st_size == 122880]
pe = pefile.PE(str(matches[0]))
image_base = pe.OPTIONAL_HEADER.ImageBase
img = pe.get_memory_mapped_image()
exports = {exp.name.decode("ascii"): exp.address for exp in pe.DIRECTORY_ENTRY_EXPORT.symbols if exp.name}

# Also inspect 90KB OMRAPI.DLL for OMR_OpenDeviceSerial DCB settings
matches_90k = [m for m in (ROOT / ".cache" / "extracted_bins").rglob("OMRAPI.DLL") if m.stat().st_size == 90112]
pe90 = pefile.PE(str(matches_90k[0]))
img90 = pe90.get_memory_mapped_image()
exp90 = {exp.name.decode("ascii"): exp.address for exp in pe90.DIRECTORY_ENTRY_EXPORT.symbols if exp.name}

md = Cs(CS_ARCH_X86, CS_MODE_32)

def dump_fn(title, image, base, rva, max_bytes=512):
    print(f"\n=== {title} @ RVA {hex(rva)} ===")
    code = image[rva : rva + max_bytes]
    int3_run = 0
    for insn in md.disasm(code, base + rva):
        if insn.mnemonic == "int3":
            int3_run += 1
            if int3_run >= 2:
                break
        else:
            int3_run = 0
        print(f"  {hex(insn.address)}: {insn.mnemonic:8s} {insn.op_str}")

dump_fn("122K::_OMR_SendCommand", img, image_base, exports["_OMR_SendCommand"], 450)
dump_fn("122K::_OMR_SendCommandAndGetResponse", img, image_base, exports["_OMR_SendCommandAndGetResponse"], 300)
dump_fn("122K::_OMR_ReceiveACKorNAK", img, image_base, exports["_OMR_ReceiveACKorNAK"], 300)
dump_fn("122K::OMR_GetMarkAD", img, image_base, exports["OMR_GetMarkAD"], 500)
dump_fn("122K::OMR_GetMarks", img, image_base, exports["OMR_GetMarks"], 450)
dump_fn("122K::OMR_GetStatus", img, image_base, exports["OMR_GetStatus"], 350)
dump_fn("90K::OMR_OpenDeviceSerial", img90, pe90.OPTIONAL_HEADER.ImageBase, exp90["OMR_OpenDeviceSerial"], 450)
