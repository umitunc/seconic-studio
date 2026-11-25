import { ThresholdPolicy } from '../types/scanner';

export interface DesignerField {
  id: string;
  key: string;
  label: string;
  fieldType: 'student_id' | 'booklet_type' | 'subject_questions';
  direction: 'D' | 'Y';
  alphabet: string;
  startCol: number;
  startRow: number;
  cols: number;
  rows: number;
  color: string;
  /** Step stride across option rows/columns (`1` for `D`/`Y`, `2` for `D2`/`Y2`, etc.). */
  optionStep?: number;
  /**
   * Per-field ordering (`Hazirlama106.exe` `Combo4`):
   * - `'normal'` (`2 - Normal`, default): follows template `orientation` (`D` = upright, `T` = 180° reversed).
   * - `'reverse'` (`1 - Ters!`): inverts ordering relative to template `orientation`.
   */
  sortOrder?: 'normal' | 'reverse';
  /** Legacy flag kept for backward compatibility with saved localStorage templates. */
  reverseItems?: boolean;
  /** `C` / `M` in `.fmt`: multiple bubbles may be marked in the same item (`C2`, `M2`). */
  allowMultipleMarks?: boolean;
  /** `Q` in `.fmt`: validate star/multi-mark only, omit from `.txt`/`.xlsx`/`.json` output. */
  hiddenFromOutput?: boolean;
  /** `Z` / `U` in `.fmt`: route sheet to sub-stacker when multi-marked. */
  routeToSubOnError?: boolean;
  /** `S` / `V` / `U` in `.fmt`: route sheet to sub-stacker when left blank. */
  routeToSubOnBlank?: boolean;
  /** Original `.fmt` `p[4]` (`K` = Karakter, `H` = Harf, `S` = Sayısal). */
  dataKind?: 'K' | 'H' | 'S';
}

export interface FormTemplate {
  id: string;
  name: string;
  rows: number;
  cols: number;
  /**
   * `D` (Düz): Timing stripes are on the left edge when the sheet is held upright.
   * `T` (Ters): Timing stripes are on the right edge when held upright, so sheets are fed 180° rotated.
   */
  orientation?: 'D' | 'T';
  markThreshold?: number;
  multiMarkChar?: string;
  blankChar?: string;
  fields: DesignerField[];
}

export const FIELD_COLORS = [
  'rgba(56, 189, 248, 0.22)',
  'rgba(16, 185, 129, 0.22)',
  'rgba(245, 158, 11, 0.22)',
  'rgba(168, 85, 247, 0.22)',
  'rgba(236, 72, 153, 0.22)',
];

const TR_ALPHABET = 'ABCÇDEFGĞHIİJKLMNOÖPRSŞTUÜVYZ';

export const DEFAULT_TEMPLATES: FormTemplate[] = [
  {
    id: 'tpl_builtin_aydin_536_d',
    name: 'AYDIN TYT AYT 536 (61 Satır • Sol Şerit D)',
    rows: 61,
    cols: 46,
    orientation: 'D',
    markThreshold: 4,
    multiMarkChar: '*',
    blankChar: ' ',
    fields: [
      { id: 'ayd_1', key: 'kod_1', label: 'KOD-1', fieldType: 'student_id', direction: 'D', alphabet: '5', startRow: 0, rows: 1, startCol: 9, cols: 1, color: FIELD_COLORS[0] },
      { id: 'ayd_2', key: 'kod_2', label: 'KOD-2', fieldType: 'student_id', direction: 'D', alphabet: '3', startRow: 9, rows: 1, startCol: 9, cols: 1, color: FIELD_COLORS[0] },
      { id: 'ayd_3', key: 'kod_3', label: 'KOD-3', fieldType: 'student_id', direction: 'D', alphabet: '6', startRow: 29, rows: 1, startCol: 1, cols: 1, color: FIELD_COLORS[0] },
      { id: 'ayd_4', key: 'okul_kodu', label: 'OKUL KODU', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startRow: 0, rows: 10, startCol: 1, cols: 8, color: FIELD_COLORS[3] },
      { id: 'ayd_5', key: 'öğrenci_no', label: 'ÖĞRENCİ NO', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startRow: 0, rows: 10, startCol: 10, cols: 5, color: FIELD_COLORS[0] },
      { id: 'ayd_6', key: 'adı_soyadı', label: 'ADI SOYADI', fieldType: 'student_id', direction: 'D', alphabet: TR_ALPHABET, startRow: 32, rows: 29, startCol: 1, cols: 20, color: FIELD_COLORS[2] },
      { id: 'ayd_7', key: 'tc_tel', label: 'TC/TEL', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startRow: 19, rows: 10, startCol: 1, cols: 11, color: FIELD_COLORS[3] },
      { id: 'ayd_8', key: 'alan', label: 'ALAN', fieldType: 'student_id', direction: 'D', alphabet: '1 2 3 4', startRow: 1, rows: 7, startCol: 21, cols: 1, color: FIELD_COLORS[4] },
      { id: 'ayd_9', key: 'sinif_1', label: 'SINIF-1', fieldType: 'student_id', direction: 'D', alphabet: '  1 1 1  ', startRow: 0, rows: 9, startCol: 17, cols: 1, color: FIELD_COLORS[4] },
      { id: 'ayd_10', key: 'sinif_2', label: 'SINIF-2', fieldType: 'student_id', direction: 'D', alphabet: '9 0 1 2 M', startRow: 0, rows: 9, startCol: 17, cols: 1, color: FIELD_COLORS[4] },
      { id: 'ayd_11', key: 'şube', label: 'ŞUBE', fieldType: 'student_id', direction: 'D', alphabet: TR_ALPHABET, startRow: 0, rows: 29, startCol: 18, cols: 1, color: FIELD_COLORS[4] },
      { id: 'ayd_12', key: 'grup', label: 'GRUP', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startRow: 19, rows: 10, startCol: 13, cols: 3, color: FIELD_COLORS[3] },
      { id: 'ayd_13', key: 'oturum_no', label: 'OTURUM NO', fieldType: 'student_id', direction: 'D', alphabet: '12', startRow: 9, rows: 2, startCol: 33, cols: 1, color: FIELD_COLORS[4] },
      { id: 'ayd_14', key: 'kitapçık', label: 'KİTAPÇIK', fieldType: 'booklet_type', direction: 'Y', alphabet: 'A B', startRow: 12, rows: 1, startCol: 7, cols: 3, color: FIELD_COLORS[2] },
      { id: 'ayd_15', key: 'türkçe', label: 'TÜRKÇE', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startRow: 15, rows: 40, startCol: 23, cols: 5, color: FIELD_COLORS[1] },
      { id: 'ayd_16', key: 'sosyal_bilimler', label: 'SOSYAL BİLİMLER', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startRow: 15, rows: 46, startCol: 29, cols: 5, color: FIELD_COLORS[1] },
      { id: 'ayd_17', key: 'matematik', label: 'MATEMATİK', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startRow: 15, rows: 40, startCol: 35, cols: 5, color: FIELD_COLORS[1] },
      { id: 'ayd_18', key: 'fen_bilimleri', label: 'FEN BİLİMLERİ', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startRow: 15, rows: 40, startCol: 41, cols: 5, color: FIELD_COLORS[1] },
    ],
  },
  {
    id: 'tpl_builtin_yks_58_t',
    name: 'YKS TYT-AYT Cevap Kağıdı (58 Satır • Sağ Şerit T)',
    rows: 58,
    cols: 45,
    orientation: 'T',
    markThreshold: 4,
    multiMarkChar: '*',
    blankChar: ' ',
    fields: [
      { id: 'yks_1', key: 'öğrenci_no', label: 'ÖĞRENCİ NO', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startRow: 48, rows: 10, startCol: 29, cols: 5, color: FIELD_COLORS[0] },
      { id: 'yks_2', key: 'adı_soyadı', label: 'ADI SOYADI', fieldType: 'student_id', direction: 'D', alphabet: TR_ALPHABET, startRow: 0, rows: 29, startCol: 24, cols: 20, color: FIELD_COLORS[2] },
      { id: 'yks_3', key: 'sınıf_1', label: 'SINIF-1', fieldType: 'student_id', direction: 'D', alphabet: '  1 1 1  ', startRow: 49, rows: 9, startCol: 28, cols: 1, color: FIELD_COLORS[4] },
      { id: 'yks_4', key: 'sınıf_2', label: 'SINIF-2', fieldType: 'student_id', direction: 'D', alphabet: '9 0 1 2 M', startRow: 49, rows: 9, startCol: 28, cols: 1, color: FIELD_COLORS[4] },
      { id: 'yks_5', key: 'şube', label: 'ŞUBE', fieldType: 'student_id', direction: 'D', alphabet: TR_ALPHABET, startRow: 29, rows: 29, startCol: 27, cols: 1, color: FIELD_COLORS[4] },
      { id: 'yks_6', key: 'kitapçık', label: 'KİTAPÇIK', fieldType: 'booklet_type', direction: 'Y', alphabet: 'A B', startRow: 45, rows: 1, startCol: 33, cols: 3, color: FIELD_COLORS[2] },
      { id: 'yks_7', key: 'türkçe', label: 'TÜRKÇE', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startRow: 0, rows: 40, startCol: 19, cols: 5, color: FIELD_COLORS[1] },
      { id: 'yks_8', key: 'sosyal_bilimler', label: 'SOSYAL BİLİMLER', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startRow: 0, rows: 40, startCol: 13, cols: 5, color: FIELD_COLORS[1] },
      { id: 'yks_9', key: 'temel_matematik', label: 'TEMEL MATEMATİK', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startRow: 0, rows: 40, startCol: 7, cols: 5, color: FIELD_COLORS[1] },
      { id: 'yks_10', key: 'fen_bilimleri', label: 'FEN BİLİMLERİ', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startRow: 0, rows: 40, startCol: 1, cols: 5, color: FIELD_COLORS[1] },
    ],
  },
  {
    id: 'tpl_default_bes_faktor',
    name: 'K12 BEŞ FAKTÖR KİŞİLİK ENVANTERİ',
    rows: 51,
    cols: 46,
    orientation: 'D',
    markThreshold: 4,
    fields: [
      { id: 'f1', key: 'ogrenci_no', label: 'ÖĞRENCİ NO', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startCol: 1, startRow: 9, cols: 5, rows: 10, color: FIELD_COLORS[0] },
      { id: 'f2', key: 'tc_kimlik', label: 'T.C. KİMLİK NO', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startCol: 12, startRow: 9, cols: 11, rows: 10, color: FIELD_COLORS[3] },
      { id: 'f3', key: 'adi_soyadi', label: 'ADI SOYADI', fieldType: 'student_id', direction: 'D', alphabet: TR_ALPHABET, startCol: 1, startRow: 22, cols: 22, rows: 29, color: FIELD_COLORS[2] },
      { id: 'f4', key: 'sorular', label: 'SORULAR', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startCol: 41, startRow: 1, cols: 5, rows: 50, color: FIELD_COLORS[1] },
    ],
  },
  {
    id: 'tpl_default_sinav_20',
    name: 'Standart Optik Sınav Formu (20 Soru)',
    rows: 32,
    cols: 24,
    orientation: 'D',
    markThreshold: 4,
    fields: [
      { id: 's1', key: 'student_id', label: 'ÖĞRENCİ NO', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startCol: 1, startRow: 2, cols: 6, rows: 10, color: FIELD_COLORS[0] },
      { id: 's2', key: 'kitapcik', label: 'KİTAPÇIK', fieldType: 'booklet_type', direction: 'Y', alphabet: 'ABCD', startCol: 9, startRow: 2, cols: 4, rows: 1, color: FIELD_COLORS[2] },
      { id: 's3', key: 'sorular', label: 'CEVAPLAR (1-20)', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startCol: 15, startRow: 2, cols: 5, rows: 20, color: FIELD_COLORS[1] },
    ],
  },
];

const STORAGE_KEY = 'sekonic-templates-v1';
const ACTIVE_ID_KEY = 'sekonic-active-template-id';

function reverseAlphabetSpec(alphabet: string): string {
  if (alphabet.includes('~')) {
    const tokens = splitAlphabetTokens(alphabet).reverse();
    return `${tokens.join('~')}~`;
  }
  return Array.from(alphabet).reverse().join('');
}

/** Migrates any older saved template where `reverseItems` pre-reversed `alphabet` on `T` forms. */
function normalizeSavedTemplate(tpl: FormTemplate): FormTemplate {
  const upgradedFields = tpl.fields.map((f) => {
    let next = f;
    if (tpl.id === 'tpl_default_bes_faktor' && f.id === 'f4' && f.alphabet === '12345') {
      next = { ...next, alphabet: 'ABCDE' };
    }
    if (next.reverseItems !== undefined && next.sortOrder === undefined) {
      // Previously `parseFmtOrBldm` pre-reversed `alphabet` when `reverseItems === true` on `T` templates.
      const restoredAlpha =
        tpl.orientation === 'T' && next.reverseItems
          ? reverseAlphabetSpec(next.alphabet)
          : next.alphabet;
      const { reverseItems: _legacy, ...rest } = next;
      next = {
        ...rest,
        alphabet: restoredAlpha,
        sortOrder: 'normal',
      };
    }
    return next;
  });

  return {
    ...tpl,
    orientation: tpl.orientation ?? 'D',
    fields: upgradedFields,
  };
}

export function loadTemplates(): FormTemplate[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as FormTemplate[];
      if (Array.isArray(parsed) && parsed.length > 0) {
        const normalized = parsed.map(normalizeSavedTemplate);
        const existingIds = new Set(normalized.map((t) => t.id));
        const missingBuiltins = DEFAULT_TEMPLATES.filter((b) => !existingIds.has(b.id));
        return [...normalized, ...missingBuiltins];
      }
    }
  } catch {
    // ignore
  }
  return DEFAULT_TEMPLATES;
}

export function saveTemplates(templates: FormTemplate[]): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(templates));
  } catch {
    // ignore
  }
}

export function loadActiveTemplateId(templates: FormTemplate[]): string {
  try {
    const saved = localStorage.getItem(ACTIVE_ID_KEY);
    if (saved && templates.some((t) => t.id === saved)) return saved;
  } catch {
    // ignore
  }
  return templates[0]?.id ?? DEFAULT_TEMPLATES[0].id;
}

export function saveActiveTemplateId(id: string): void {
  try {
    localStorage.setItem(ACTIVE_ID_KEY, id);
  } catch {
    // ignore
  }
}

export function splitAlphabetTokens(alphabet: string): string[] {
  if (alphabet.includes('~')) {
    const parts = alphabet.split('~');
    if (parts[parts.length - 1] === '') parts.pop();
    return parts;
  }
  return Array.from(alphabet);
}

/**
 * Parses Sekonic / Bikom `.fmt` (`Hazirlama106.exe` & `SekOku.exe`) and `.bldm` files.
 *
 * Verified from `Hazirlama106.exe` (`0x487d73..0x488a78`) & `SekOku.exe` (`0x476900..0x4af9b4`):
 * - Header: `rows=cols=thresholdHex=orientation(D|T)=multiChar=blankChar=`
 *   - `D` (Düz): Timing stripes on the left edge when held upright.
 *   - `T` (Ters): Timing stripes on the right edge when held upright (fed 180° rotated so stripes sit on left).
 * - Field: `r1=r2=c1=c2=dataKind(K|H|S|T)=dir(D|Y|D2|Y2)=alphabet=mode(X2|X1|X|C2|M2|Q2|Z2|U2)=label=`
 */
export function parseFmtOrBldm(text: string): {
  rows: number;
  cols: number;
  orientation: 'D' | 'T';
  markThreshold?: number;
  multiMarkChar?: string;
  blankChar?: string;
  fields: DesignerField[];
} | null {
  const lines = text.split(/\r?\n/).map((l) => l.trimEnd()).filter((l) => l.trim().length > 0);
  if (lines.length < 2) return null;
  const h = lines[0].trim().split('=');
  const rows = Number(h[0]) || 51;
  const cols = Number(h[1]) || 46;
  // `SekOku.exe @ 0x48853f` parses `h[2]` via `Val("&H" & h[2])`
  const rawThresh = parseInt((h[2] ?? '').trim(), 16);
  const markThreshold = Number.isFinite(rawThresh) && rawThresh > 0 ? rawThresh : undefined;
  const isReverseMode = (h[3] ?? '').trim().toUpperCase() === 'T';
  const orientation: 'D' | 'T' = isReverseMode ? 'T' : 'D';
  const multiMarkChar = h[4] && h[4].length > 0 ? h[4] : '*';
  const blankChar = h[5] && h[5].length > 0 ? h[5] : ' ';
  const fields: DesignerField[] = [];
  const usedKeys = new Set<string>();
  const allocKey = (raw: string, fallback: string): string => {
    const base =
      raw
        .toLocaleLowerCase('tr-TR')
        .replace(/[^a-z0-9çğıöşü]+/gi, '_')
        .replace(/^_+|_+$/g, '') || fallback;
    let unique = base;
    let suffix = 2;
    while (usedKeys.has(unique)) {
      unique = `${base}_${suffix++}`;
    }
    usedKeys.add(unique);
    return unique;
  };

  for (let i = 1; i < lines.length; i++) {
    const p = lines[i].trim().split('=');
    if (p.length < 7) continue;

    // Recover occasional `.fmt` typos where `~X2=` missed `=` before `X2` (e.g. `K12 SOSYOMETRİ`)
    if (p.length === 8 && /~[A-Z][123]?$/i.test(p[6] ?? '')) {
      const m = (p[6] ?? '').match(/^(.*~)([A-Z][123]?)$/i);
      if (m) {
        p.splice(6, 1, m[1], m[2]);
      }
    }

    const kindUpper = (p[4] ?? '').trim().toUpperCase();
    const dirToken = (p[5] ?? '').trim().toUpperCase();
    const isFmt =
      p.length >= 8 &&
      (kindUpper === 'K' || kindUpper === 'H' || kindUpper === 'S' || kindUpper === 'T') &&
      /^[DY]\d*$/.test(dirToken);

    if (isFmt) {
      const r1 = Math.max(1, Number(p[0]) || 1) - 1;
      const r2 = Math.max(r1 + 1, Number(p[1]) || r1 + 1) - 1;
      const c1 = Math.max(1, Number(p[2]) || 1) - 1;
      const c2 = Math.max(c1 + 1, Number(p[3]) || c1 + 1) - 1;
      const dir: 'D' | 'Y' = dirToken.startsWith('Y') ? 'Y' : 'D';
      const stepParsed = parseInt(dirToken.slice(1), 10);
      const optionStep = Number.isFinite(stepParsed) && stepParsed > 1 ? stepParsed : 1;

      const rawAlphabet = p[6] ?? '';
      const modeFlag = (p[7] ?? '').trim().toUpperCase();
      const modeLead = modeFlag[0] ?? 'X';
      const sortDigit = modeFlag.slice(1); // '2' = Normal, '1' = Ters!, '' = legacy Bikom ('X')

      const rawName = p
        .slice(8)
        .join('=')
        .replace(/=+$/g, '')
        .replace(/^\[|\]$/g, '')
        .trim();

      if (rawName.toLocaleLowerCase('tr-TR') === 'boş' || rawAlphabet.length === 0) {
        continue;
      }
      // Skip spacer-only lines (`33=33=01=40=K=D= =X2==` where alphabet is pure whitespace and name is empty)
      if (rawAlphabet.trim() === '' && rawName === '') {
        continue;
      }

      // Normalize alphabet & sortOrder so `DesignerField.alphabet` always holds logical order
      // while `templateToOmrSchema` applies exact `SekOku.exe` `T`-mode reversal!
      let alphabet = rawAlphabet;
      let sortOrder: 'normal' | 'reverse' = 'normal';
      if (isReverseMode) {
        if (sortDigit === '2') {
          // Standard Sekonic `.fmt` (`X2`, `C2`, `Z2`, `U2`): `rawAlphabet` is normal (`ABCDE`),
          // and `templateToOmrSchema` reverses both option labels and item sequence when `orientation === 'T'`.
          sortOrder = 'normal';
        } else if (sortDigit === '1') {
          // `1 - Ters!`: do NOT reverse on `T` form (`SekOku.exe @ 0x4aec1c`)
          sortOrder = 'reverse';
        } else {
          // Legacy `BIKOM.fmt` (`p[7] = 'X'` without digit on `T` header):
          // If author pre-reversed alphabet (`ZYV...CBA`, `9876543210`, `CBA`), un-reverse it to logical order
          // so `templateToOmrSchema` reverses both option labels and item sequence uniformly.
          const looksPreReversed =
            rawAlphabet.startsWith('ZY') ||
            rawAlphabet.startsWith('987') ||
            rawAlphabet === 'EDCBA' ||
            rawAlphabet === 'DCBA' ||
            rawAlphabet === 'CBA';
          if (looksPreReversed) {
            alphabet = reverseAlphabetSpec(rawAlphabet);
            sortOrder = 'normal';
          } else {
            // Only item sequence is reversed; keep alphabet as written by flipping it once before `T` reversal
            alphabet = reverseAlphabetSpec(rawAlphabet);
            sortOrder = 'normal';
          }
        }
      } else if (sortDigit === '1') {
        // Explicit `1 - Ters!` on a `D` form
        sortOrder = 'reverse';
      }

      const allowMultipleMarks = modeLead === 'C' || modeLead === 'M';
      const hiddenFromOutput = modeLead === 'Q';
      const routeToSubOnError = modeLead === 'Z' || modeLead === 'U';
      const routeToSubOnBlank = modeLead === 'S' || modeLead === 'V' || modeLead === 'U';

      const effectiveLabel = rawName || `Alan ${fields.length + 1}`;
      const lower = effectiveLabel.toLocaleLowerCase('tr-TR');
      const fieldType: DesignerField['fieldType'] =
        lower.includes('kitap')
          ? 'booklet_type'
          : dir === 'D' ||
              kindUpper === 'S' ||
              lower.includes('no') ||
              lower.includes('tc') ||
              lower.includes('ad') ||
              lower.includes('kimlik') ||
              lower.includes('kurum') ||
              lower.includes('okul') ||
              lower.includes('sınıf') ||
              lower.includes('sinif') ||
              lower.includes('şube') ||
              lower.includes('sube')
            ? 'student_id'
            : 'subject_questions';

      fields.push({
        id: `fmt_${i}`,
        key: allocKey(effectiveLabel, `alan_${i}`),
        label: effectiveLabel,
        fieldType,
        direction: dir,
        alphabet,
        startCol: c1,
        startRow: r1,
        cols: c2 - c1 + 1,
        rows: r2 - r1 + 1,
        color: FIELD_COLORS[fields.length % FIELD_COLORS.length],
        optionStep: optionStep > 1 ? optionStep : undefined,
        sortOrder,
        allowMultipleMarks: allowMultipleMarks || undefined,
        hiddenFromOutput: hiddenFromOutput || undefined,
        routeToSubOnError: routeToSubOnError || undefined,
        routeToSubOnBlank: routeToSubOnBlank || undefined,
        dataKind: (kindUpper === 'S' ? 'S' : kindUpper === 'H' ? 'H' : 'K'),
      });
    } else {
      const r1 = Math.max(1, Number(p[1]) || 1) - 1;
      const c1 = Math.max(1, Number(p[2]) || 1) - 1;
      const c2 = Math.max(c1 + 1, Number(p[3]) || c1 + 1) - 1;
      const lbl = p[6]?.trim() || `Alan ${p[0]}`;
      fields.push({
        id: `bldm_${i}`,
        key: `alan_${p[0]}`,
        label: lbl,
        fieldType: 'subject_questions',
        direction: 'Y',
        alphabet: 'ABCDE',
        startCol: c1,
        startRow: r1,
        cols: c2 - c1 + 1,
        rows: 1,
        color: FIELD_COLORS[fields.length % FIELD_COLORS.length],
      });
    }
  }
  return { rows, cols, orientation, markThreshold, multiMarkChar, blankChar, fields };
}

/** Exports a `FormTemplate` to a genuine `SekOku.exe` / `Hazirlama106.exe`-compatible `.fmt` string. */
export function serializeTemplateToFmt(tpl: FormTemplate): string {
  const pad2 = (n: number) => String(Math.max(1, Math.round(n))).padStart(2, '0');
  const threshHex = String(Math.max(1, Math.min(15, tpl.markThreshold ?? 4)))
    .toString()
    .padStart(2, '0');
  const orient = tpl.orientation === 'T' ? 'T' : 'D';
  const multi = tpl.multiMarkChar || '*';
  const blank = tpl.blankChar || ' ';

  const lines: string[] = [`${pad2(tpl.rows)}=${pad2(tpl.cols)}=${threshHex}=${orient}=${multi}=${blank}=`];
  for (const f of tpl.fields) {
    const r1 = pad2(f.startRow + 1);
    const r2 = pad2(f.startRow + f.rows);
    const c1 = pad2(f.startCol + 1);
    const c2 = pad2(f.startCol + f.cols);
    const kind = f.dataKind ?? 'K';
    const step = f.optionStep && f.optionStep > 1 ? String(f.optionStep) : '';
    const dir = `${f.direction}${step}`;
    const modeLead = f.allowMultipleMarks
      ? f.direction === 'Y'
        ? 'C'
        : 'M'
      : f.hiddenFromOutput
        ? 'Q'
        : f.routeToSubOnError && f.routeToSubOnBlank
          ? 'U'
          : f.routeToSubOnBlank
            ? 'S'
            : f.routeToSubOnError
              ? 'Z'
              : 'X';
    const sortDigit = f.sortOrder === 'reverse' ? '1' : '2';
    lines.push(`${r1}=${r2}=${c1}=${c2}=${kind}=${dir}=${f.alphabet}=${modeLead}${sortDigit}=${f.label}=`);
  }
  return `${lines.join('\r\n')}\r\n`;
}

/** Computes the fixed character width produced by a `DesignerField` in `SekOku.exe` `.txt` output. */
export function computeFieldOutputWidth(f: DesignerField): number {
  if (f.hiddenFromOutput) return 0;
  const itemCount = f.direction === 'D' ? f.cols : f.rows;
  const tokens = splitAlphabetTokens(f.alphabet || 'ABCDE');
  const maxTokenLen = Math.max(1, ...tokens.map((t) => Array.from(t).length));
  if (f.allowMultipleMarks) {
    const step = Math.max(1, f.optionStep ?? 1);
    const span = f.direction === 'D' ? f.rows : f.cols;
    const optCount = Math.floor((span - 1) / step) + 1;
    return itemCount * Math.max(optCount, maxTokenLen);
  }
  return itemCount * maxTokenLen;
}

export interface DensityCapConfig {
  enabled: boolean;
  /** `SekOku.exe` `Koyuluk Ayarı -> Üst Sınır` (`5..16`, `sekonic.ini` default `9`). */
  upperLimit: number;
}

function mdLevelToDensity(lvl: number): number {
  if (lvl <= 4) return lvl * 8;
  if (lvl <= 8) return 64 + (lvl - 5) * 16;
  return Math.min(255, 140 + (lvl - 9) * 14);
}

/**
 * Converts a UI `FormTemplate` into the exact Rust `OmrFormSchema` JSON payload
 * accepted by `sekonic-agent` and `sekonic-grader`.
 */
export function templateToOmrSchema(
  tpl: FormTemplate,
  policy: ThresholdPolicy,
  duplex: boolean,
  dropoutLed: string,
  densityConfig?: DensityCapConfig,
): Record<string, unknown> {
  const maxRows = Math.max(1, Math.round(tpl.rows));
  const maxCols = Math.max(1, Math.round(tpl.cols));
  const isTplReverse = tpl.orientation === 'T';
  const usedKeys = new Set<string>();

  const fields = tpl.fields.map((f, idx) => {
    const baseKey = (f.key || `field_${idx + 1}`).trim() || `field_${idx + 1}`;
    let uniqueKey = baseKey;
    let suffix = 2;
    while (usedKeys.has(uniqueKey)) {
      uniqueKey = `${baseKey}_${suffix++}`;
    }
    usedKeys.add(uniqueKey);

    // Exact `SekOku.exe @ 0x4aea09..0x4af9b4` orientation truth table:
    // - `D` + `normal` (`X2`): upright alphabet & upright item order
    // - `T` + `normal` (`X2`): 180°-rotated sheet -> reverse both option labels AND item order!
    // - `D` + `reverse` (`X1`): reversed item order
    // - `T` + `reverse` (`X1`): un-reversed
    const effectiveReverse =
      f.sortOrder !== undefined
        ? isTplReverse !== (f.sortOrder === 'reverse')
        : Boolean(f.reverseItems) || isTplReverse;

    const baseTokens = splitAlphabetTokens(f.alphabet || 'ABCDE');
    const step = Math.max(1, Math.round(f.optionStep ?? 1));
    const optSpan = f.direction === 'D' ? f.rows : f.cols;
    const slotCount = Math.max(1, Math.floor((optSpan - 1) / step) + 1);

    // Align option labels to exact grid slot count before reversing on `T` forms
    const slotLabels: string[] = [];
    for (let s = 0; s < slotCount; s++) {
      slotLabels.push(
        baseTokens[s] ??
          (f.direction === 'D' ? String(s) : String.fromCharCode(65 + (s % 26))),
      );
    }
    if (effectiveReverse) {
      slotLabels.reverse();
    }

    const items: Array<{ index: number; options: Array<{ label: string; row: number; col: number }> }> = [];

    if (f.direction === 'D') {
      // Vertical digits/letters: each column is one item, rows (stepped by `step`) are options
      for (let c = 0; c < f.cols; c++) {
        const col = f.startCol + c;
        if (col < 0 || col >= maxCols) continue;
        const options: Array<{ label: string; row: number; col: number }> = [];
        for (let s = 0; s < slotCount; s++) {
          const row = f.startRow + s * step;
          if (row < 0 || row >= maxRows) continue;
          options.push({
            label: slotLabels[s] ?? String(s),
            row,
            col,
          });
        }
        if (options.length > 0) {
          items.push({ index: c + 1, options });
        }
      }
    } else {
      // Horizontal questions: each row is one item, columns (stepped by `step`) are options
      for (let r = 0; r < f.rows; r++) {
        const row = f.startRow + r;
        if (row < 0 || row >= maxRows) continue;
        const options: Array<{ label: string; row: number; col: number }> = [];
        for (let s = 0; s < slotCount; s++) {
          const col = f.startCol + s * step;
          if (col < 0 || col >= maxCols) continue;
          options.push({
            label: slotLabels[s] ?? String.fromCharCode(65 + (s % 26)),
            row,
            col,
          });
        }
        if (options.length > 0) {
          items.push({ index: r + 1, options });
        }
      }
    }

    if (effectiveReverse && items.length > 1) {
      items.reverse();
      items.forEach((it, i) => {
        it.index = i + 1;
      });
    }

    return {
      key: uniqueKey,
      field_type: f.fieldType,
      side: 'front',
      // Matches `SekOku.exe`: `X2` mode does NOT reject partially/fully blank fields;
      // only explicit `S`/`V`/`U` `.fmt` modes route blanks to Sub-Stacker.
      required_for_valid_sheet: Boolean(f.routeToSubOnBlank),
      allow_multiple_marks: Boolean(f.allowMultipleMarks),
      hidden_from_output: Boolean(f.hiddenFromOutput),
      route_to_sub_on_multi: Boolean(f.routeToSubOnError),
      route_to_sub_on_blank: Boolean(f.routeToSubOnBlank),
      items,
    };
  });

  const colorLower = dropoutLed.toLowerCase();
  const dropout_color =
    colorLower === 'red' || colorLower === 'green' || colorLower === 'blue' ? colorLower : 'none';

  // Map `.fmt` header `markThreshold` (`h[2]`: `03` -> `lvl > 3`, `04` -> `lvl > 4`) onto our calibrated curve
  // (`0..=4 -> lvl * 8`, `5..=8 -> 64..112`, `9..=16 -> 140..238`) when default `minMarkDensity === 48`.
  let effectiveMinDensity = policy.minMarkDensity;
  if (policy.minMarkDensity === 48 && tpl.markThreshold !== undefined) {
    if (tpl.markThreshold <= 3) {
      effectiveMinDensity = Math.max(12, tpl.markThreshold * 8 + 4);
    } else if (tpl.markThreshold === 4) {
      effectiveMinDensity = 48;
    } else if (tpl.markThreshold <= 8) {
      effectiveMinDensity = 64 + (tpl.markThreshold - 5) * 16 + 4;
    }
  }

  const saturationDensity =
    densityConfig === undefined
      ? 135
      : !densityConfig.enabled
        ? 255
        : Math.max(effectiveMinDensity, mdLevelToDensity(Math.max(5, Math.min(16, Math.round(densityConfig.upperLimit)))) - 5);

  return {
    schema_version: 1,
    form_id: tpl.id || 'CUSTOM-FORM',
    form_name: tpl.name || 'Aktif Form',
    duplex,
    dropout_color,
    orientation: isTplReverse ? 'T' : 'D',
    multi_mark_char: tpl.multiMarkChar || '*',
    blank_char: tpl.blankChar || ' ',
    grid: {
      front_rows: maxRows,
      front_cols: maxCols,
      back_rows: duplex ? maxRows : 0,
      back_cols: duplex ? maxCols : 0,
      width_mm: 210.0,
      height_mm: 297.0,
    },
    threshold_policy: {
      min_mark_density: effectiveMinDensity,
      strong_mark_density: policy.strongMarkDensity,
      erasure_ratio: policy.erasureRatio,
      review_confidence_floor: policy.confidenceFloor,
      saturation_density: saturationDensity,
    },
    fields,
  };
}

export interface RegradedWireResult {
  decoded_fields: Record<string, string>;
  answers: Record<string, string>;
  answer_details: Record<
    string,
    Array<{
      index: number;
      state: 'blank' | 'marked' | 'marked_with_erasure' | 'multi_mark';
      value: string | null;
      multi_values: string[];
      erased_label: string | null;
      confidence: number;
      densities: number[];
    }>
  >;
  warnings: string[];
  needs_operator_review: boolean;
  missing_required_field: boolean;
  raw_matrix_b64: string;
  observed_front_rows: number;
}

function decodeMdChar(code: number): number {
  if (code >= 48 && code <= 64) return code - 48; // '0'..='@'
  if (code >= 65 && code <= 70) return code - 65 + 10; // 'A'..='F'
  if (code >= 97 && code <= 102) return code - 97 + 10; // 'a'..='f'
  return 0;
}

function padChars(s: string, width: number, padCh: string): string {
  const chars = Array.from(s);
  if (chars.length >= width) return chars.slice(0, width).join('');
  return s + padCh.repeat(width - chars.length);
}

/**
 * Re-grades a captured `MD0` wire string against ANY `FormTemplate` directly in the browser,
 * matching Rust `decode_md_into_slice` + `solve_bubble_group` + `grade_sheet` 1:1.
 * Allows instant template/orientation switching without physically re-feeding sheets.
 */
export function regradeWireWithTemplate(
  wire: string,
  tpl: FormTemplate,
  policy: ThresholdPolicy,
  densityConfig?: DensityCapConfig,
): RegradedWireResult | null {
  if (!wire) return null;
  let body = wire.startsWith('MD') && wire.length >= 6 ? wire.slice(6) : wire.startsWith('MD') ? wire.slice(2) : wire;
  body = body.replace(/[\0\r\n ]+$/g, '');
  if (!body) return null;

  const schema = templateToOmrSchema(tpl, policy, false, 'Red', densityConfig) as {
    multi_mark_char: string;
    blank_char: string;
    grid: { front_rows: number; front_cols: number };
    threshold_policy: {
      min_mark_density: number;
      strong_mark_density: number;
      erasure_ratio: number;
      review_confidence_floor: number;
      saturation_density: number;
    };
    fields: Array<{
      key: string;
      field_type: string;
      required_for_valid_sheet: boolean;
      allow_multiple_marks: boolean;
      hidden_from_output: boolean;
      route_to_sub_on_multi: boolean;
      route_to_sub_on_blank: boolean;
      items: Array<{ index: number; options: Array<{ label: string; row: number; col: number }> }>;
    }>;
  };

  const rows = schema.grid.front_rows;
  const cols = schema.grid.front_cols;
  const matrix = new Uint8Array(rows * cols);
  const tokens = body.includes(',') ? body.split(',').filter((t) => t.length > 0) : [];
  const observedRows = tokens.length;

  const platenMask = new Array<boolean>(48).fill(false);
  if (tokens.length >= 15) {
    for (let ch = 47; ch >= 43; ch--) {
      let white = 0;
      let dark = 0;
      for (const row of tokens) {
        if (ch < row.length) {
          const lvl = decodeMdChar(row.charCodeAt(ch));
          if (lvl <= 4) white++;
          else if (lvl >= 11) dark++;
        }
      }
      if (white === 0 && dark * 100 >= tokens.length * 92) {
        platenMask[ch] = true;
      } else {
        break;
      }
    }
  }

  for (let r = 0; r < Math.min(rows, tokens.length); r++) {
    const rowStr = tokens[r];
    for (let c = 0; c < Math.min(cols, rowStr.length); c++) {
      matrix[r * cols + c] = c < 48 && platenMask[c] ? 0 : mdLevelToDensity(decodeMdChar(rowStr.charCodeAt(c)));
    }
  }

  // Estimate 10th-percentile baseline
  const hist = new Uint32Array(256);
  for (let i = 0; i < matrix.length; i++) hist[matrix[i]]++;
  const target = Math.max(1, Math.floor(matrix.length / 10));
  let acc = 0;
  let baseline = 0;
  for (let v = 0; v < 256; v++) {
    acc += hist[v];
    if (acc >= target) {
      baseline = v;
      break;
    }
  }

  const decoded_fields: Record<string, string> = {};
  const answers: Record<string, string> = {};
  const answer_details: RegradedWireResult['answer_details'] = {};
  const warnings: string[] = [];
  let needs_operator_review = false;
  let missing_required_field = false;

  if (observedRows > 0 && observedRows !== rows) {
    needs_operator_review = true;
    missing_required_field = true;
    warnings.push(`okuma_çizgisi_uyumsuz:${observedRows}/${rows}`);
  }

  const tp = schema.threshold_policy;
  const blankCh = Array.from(schema.blank_char || ' ')[0] ?? ' ';
  const multiStr = schema.multi_mark_char || '*';

  for (const field of schema.fields) {
    const decisions: RegradedWireResult['answer_details'][string] = [];
    let compact = '';
    let fieldValid = true;

    for (const group of field.items) {
      const tokenWidth = Math.max(1, ...group.options.map((o) => Array.from(o.label).length));
      const rawDensities: number[] = [];

      if (field.allow_multiple_marks) {
        const marked: string[] = [];
        for (const opt of group.options) {
          const raw = opt.row < rows && opt.col < cols ? matrix[opt.row * cols + opt.col] : 0;
          rawDensities.push(raw);
          if (opt.label !== '`' && Math.max(0, raw - baseline) >= tp.min_mark_density) {
            marked.push(opt.label);
          }
        }
        const joined = marked.join('');
        compact += padChars(joined, Math.max(group.options.length, tokenWidth), blankCh);
        decisions.push({
          index: group.index,
          state: marked.length === 0 ? 'blank' : 'marked',
          value: marked.length === 0 ? null : joined,
          multi_values: marked,
          erased_label: null,
          confidence: 0.98,
          densities: rawDensities,
        });
        continue;
      }

      const scored: Array<[string, number]> = [];
      for (const opt of group.options) {
        const raw = opt.row < rows && opt.col < cols ? matrix[opt.row * cols + opt.col] : 0;
        rawDensities.push(raw);
        if (opt.label === '`') continue;
        scored.push([opt.label, Math.max(0, raw - baseline)]);
      }
      scored.sort((a, b) => b[1] - a[1]);
      const [l1, v1] = scored[0] ?? ['', 0];
      const [l2, v2] = scored[1] ?? ['', 0];

      if (v1 < tp.min_mark_density) {
        compact += blankCh.repeat(tokenWidth);
        if (field.required_for_valid_sheet || field.route_to_sub_on_blank) fieldValid = false;
        decisions.push({
          index: group.index,
          state: 'blank',
          value: null,
          multi_values: [],
          erased_label: null,
          confidence: Math.max(0.5, Math.min(1, 1 - v1 / Math.max(1, tp.min_mark_density))),
          densities: rawDensities,
        });
      } else if (v2 < tp.min_mark_density) {
        compact += padChars(l1, tokenWidth, blankCh);
        decisions.push({
          index: group.index,
          state: 'marked',
          value: l1,
          multi_values: [],
          erased_label: null,
          confidence: Math.max(0.6, Math.min(1, 0.6 + 0.4 * ((v1 - tp.min_mark_density) / Math.max(1, 255 - tp.min_mark_density)))),
          densities: rawDensities,
        });
      } else {
        const sat = Math.max(tp.min_mark_density, tp.saturation_density ?? 255);
        const c1 = Math.min(v1, sat);
        const c2 = Math.min(v2, sat);
        const ratio = c2 / Math.max(1, c1);
        if (c1 > c2 && (v1 >= sat || ratio <= tp.erasure_ratio)) {
          compact += padChars(l1, tokenWidth, blankCh);
          warnings.push(`${field.key}_q${group.index}_erasure_recovered`);
          decisions.push({
            index: group.index,
            state: 'marked_with_erasure',
            value: l1,
            multi_values: [],
            erased_label: l2,
            confidence: Math.max(0.68, Math.min(0.95, 1 - ratio * 0.5)),
            densities: rawDensities,
          });
        } else {
          compact += padChars(multiStr, tokenWidth, blankCh);
          needs_operator_review = true;
          warnings.push(`${field.key}_q${group.index}_multi_mark`);
          if (field.required_for_valid_sheet || field.route_to_sub_on_multi) fieldValid = false;
          decisions.push({
            index: group.index,
            state: 'multi_mark',
            value: null,
            multi_values: scored.filter(([, v]) => v >= tp.min_mark_density).map(([l]) => l),
            erased_label: null,
            confidence: Math.max(0.5, Math.min(1, ratio)),
            densities: rawDensities,
          });
        }
      }
    }

    if ((field.required_for_valid_sheet || field.route_to_sub_on_multi || field.route_to_sub_on_blank) && !fieldValid) {
      missing_required_field = true;
    }

    answer_details[field.key] = decisions;
    if (field.hidden_from_output) continue;
    if (field.field_type === 'subject_questions') {
      answers[field.key] = compact;
    } else {
      decoded_fields[field.key] = compact;
    }
  }

  let bin = '';
  for (let i = 0; i < matrix.length; i++) bin += String.fromCharCode(matrix[i]);
  return {
    decoded_fields,
    answers,
    answer_details,
    warnings,
    needs_operator_review,
    missing_required_field,
    raw_matrix_b64: btoa(bin),
    observed_front_rows: observedRows,
  };
}
