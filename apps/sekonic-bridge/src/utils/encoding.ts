/**
 * Turkish ANSI (Windows-1254 / CP1254) <-> UTF-8 helpers for 100% byte-compatible
 * interoperability with `SekOku.exe`, `Hazirlama106.exe`, and institutional OMR evaluators.
 */

const CP1254_EXTRA: Record<number, number> = {
  0x20ac: 0x80, // €
  0x201a: 0x82,
  0x0192: 0x83,
  0x201e: 0x84,
  0x2026: 0x85,
  0x2020: 0x86,
  0x2021: 0x87,
  0x02c6: 0x88,
  0x2030: 0x89,
  0x0160: 0x8a, // Š
  0x2039: 0x8b,
  0x0152: 0x8c, // Œ
  0x2018: 0x91,
  0x2019: 0x92,
  0x201c: 0x93,
  0x201d: 0x94,
  0x2022: 0x95,
  0x2013: 0x96,
  0x2014: 0x97,
  0x02dc: 0x98,
  0x2122: 0x99,
  0x0161: 0x9a, // š
  0x203a: 0x9b,
  0x0153: 0x9c, // œ
  0x0178: 0x9f, // Ÿ
  0x011e: 0xd0, // Ğ
  0x0130: 0xdd, // İ
  0x015e: 0xde, // Ş
  0x011f: 0xf0, // ğ
  0x0131: 0xfd, // ı
  0x015f: 0xfe, // ş
};

/**
 * Decodes an ArrayBuffer containing either UTF-8 (with or without BOM) or Windows-1254 Turkish ANSI.
 * Valid Windows-1254 Turkish byte sequences (`Ğ, İ, Ş, Ç, Ö, Ü` followed by ASCII) are strictly
 * invalid UTF-8, so trying fatal UTF-8 first distinguishes both encodings reliably.
 */
export function decodeAutoTurkishText(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  if (bytes.length >= 3 && bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
    return new TextDecoder('utf-8').decode(bytes.subarray(3));
  }
  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  } catch {
    return new TextDecoder('windows-1254').decode(bytes);
  }
}

/**
 * Encodes a JavaScript string into exact 1-byte-per-character Windows-1254 (CP1254) bytes
 * matching `SekOku.exe` `.txt` and `Hazirlama106.exe` `.fmt` output byte-for-byte.
 */
export function encodeWindows1254(text: string): Uint8Array {
  const out = new Uint8Array(text.length);
  for (let i = 0; i < text.length; i++) {
    const code = text.charCodeAt(i);
    if (code <= 0x7f) {
      out[i] = code;
    } else if (code in CP1254_EXTRA) {
      out[i] = CP1254_EXTRA[code];
    } else if (code >= 0xa0 && code <= 0xff && code !== 0xd0 && code !== 0xdd && code !== 0xde && code !== 0xf0 && code !== 0xfd && code !== 0xfe) {
      out[i] = code;
    } else {
      out[i] = 0x3f; // '?'
    }
  }
  return out;
}

