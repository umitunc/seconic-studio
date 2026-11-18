import hashlib
import os
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
ARCHIVE_DIR = ROOT / "docs" / "reverse-engineering"
EXTRACT_DIR = ROOT / ".cache" / "extracted_bins"

EXPECTED_ZIPS = [
    "sr55dkurulum.zip",
    "sr1800kurulum.zip",
    "sr3500kurulum.zip",
    "sr6000kurulum.zip",
    "sr6500kurulum.zip",
]

EXPECTED_HASHES = {
    "e91cb0851c77ccd2": "OMRAPI.dll (122,880 bytes)",
    "f7ed656556426228": "SkCommIo.dll (126,976 bytes)",
    "68594b30831adab8": "SkDvSr55.dll (188,416 bytes)",
    "348a9ad1bbd6476e": "SkRgOmr.dll (177,664 bytes)",
}


def sha256_16(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()[:16]


def main() -> int:
    if not ARCHIVE_DIR.is_dir():
        print(f"ERROR: Missing archive directory: {ARCHIVE_DIR}", file=sys.stderr)
        return 1

    EXTRACT_DIR.mkdir(parents=True, exist_ok=True)

    for zname in EXPECTED_ZIPS:
        zpath = ARCHIVE_DIR / zname
        if not zpath.is_file():
            print(f"ERROR: Missing expected archive: {zpath}", file=sys.stderr)
            return 1
        dest = EXTRACT_DIR / zpath.stem
        if not dest.exists():
            print(f"Extracting {zname} -> {dest.relative_to(ROOT)}")
            with zipfile.ZipFile(zpath, "r") as zf:
                zf.extractall(dest)

    found_hashes: dict[str, list[str]] = {}
    for root, _, files in os.walk(EXTRACT_DIR):
        for fname in files:
            fpath = Path(root) / fname
            if fpath.suffix.lower() in (".dll", ".exe", ".sys", ".ocx"):
                digest = sha256_16(fpath)
                found_hashes.setdefault(digest, []).append(str(fpath.relative_to(EXTRACT_DIR)))

    all_ok = True
    for expected_hash, desc in EXPECTED_HASHES.items():
        if expected_hash in found_hashes:
            print(f"[OK] {expected_hash} ({desc}) -> {len(found_hashes[expected_hash])} copy/copies")
        else:
            print(f"[FAIL] Missing binary hash {expected_hash} ({desc})", file=sys.stderr)
            all_ok = False

    return 0 if all_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
