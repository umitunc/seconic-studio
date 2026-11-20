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
}

export interface FormTemplate {
  id: string;
  name: string;
  rows: number;
  cols: number;
  fields: DesignerField[];
}

export const FIELD_COLORS = [
  'rgba(56, 189, 248, 0.22)',
  'rgba(16, 185, 129, 0.22)',
  'rgba(245, 158, 11, 0.22)',
  'rgba(168, 85, 247, 0.22)',
  'rgba(236, 72, 153, 0.22)',
];

export const DEFAULT_TEMPLATES: FormTemplate[] = [
  {
    id: 'tpl_default_bes_faktor',
    name: 'K12 BEŞ FAKTÖR KİŞİLİK ENVANTERİ',
    rows: 51,
    cols: 46,
    fields: [
      { id: 'f1', key: 'ogrenci_no', label: 'ÖĞRENCİ NO', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startCol: 1, startRow: 9, cols: 5, rows: 10, color: FIELD_COLORS[0] },
      { id: 'f2', key: 'tc_kimlik', label: 'T.C. KİMLİK NO', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startCol: 12, startRow: 9, cols: 11, rows: 10, color: FIELD_COLORS[3] },
      { id: 'f3', key: 'adi_soyadi', label: 'ADI SOYADI', fieldType: 'student_id', direction: 'D', alphabet: 'ABCÇDEFGĞHIİJKLMNOÖPRSŞTUÜVYZ', startCol: 1, startRow: 22, cols: 22, rows: 29, color: FIELD_COLORS[2] },
      { id: 'f4', key: 'sorular', label: 'SORULAR', fieldType: 'subject_questions', direction: 'Y', alphabet: '12345', startCol: 41, startRow: 1, cols: 5, rows: 50, color: FIELD_COLORS[1] },
    ],
  },
  {
    id: 'tpl_default_sinav_20',
    name: 'Standart Optik Sınav Formu (20 Soru)',
    rows: 32,
    cols: 24,
    fields: [
      { id: 's1', key: 'student_id', label: 'ÖĞRENCİ NO', fieldType: 'student_id', direction: 'D', alphabet: '0123456789', startCol: 1, startRow: 2, cols: 6, rows: 10, color: FIELD_COLORS[0] },
      { id: 's2', key: 'kitapcik', label: 'KİTAPÇIK', fieldType: 'booklet_type', direction: 'Y', alphabet: 'ABCD', startCol: 9, startRow: 2, cols: 4, rows: 1, color: FIELD_COLORS[2] },
      { id: 's3', key: 'sorular', label: 'CEVAPLAR (1-20)', fieldType: 'subject_questions', direction: 'Y', alphabet: 'ABCDE', startCol: 15, startRow: 2, cols: 5, rows: 20, color: FIELD_COLORS[1] },
    ],
  },
];

const STORAGE_KEY = 'sekonic-templates-v1';
const ACTIVE_ID_KEY = 'sekonic-active-template-id';

export function loadTemplates(): FormTemplate[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as FormTemplate[];
      if (Array.isArray(parsed) && parsed.length > 0) return parsed;
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

export function parseFmtOrBldm(text: string): { rows: number; cols: number; fields: DesignerField[] } | null {
  const lines = text.split(/\r?\n/).map((l) => l.trim()).filter(Boolean);
  if (lines.length < 2) return null;
  const h = lines[0].split('=');
  const rows = Number(h[0]) || 51;
  const cols = Number(h[1]) || 46;
  const fields: DesignerField[] = [];

  for (let i = 1; i < lines.length; i++) {
    const p = lines[i].split('=');
    if (p.length < 7) continue;

    const isFmt = p.length >= 9 && (p[4] === 'K' || p[4] === 'H') && (p[5] === 'D' || p[5] === 'Y');
    if (isFmt) {
      const r1 = Math.max(1, Number(p[0]) || 1) - 1;
      const r2 = Math.max(r1 + 1, Number(p[1]) || r1 + 1) - 1;
      const c1 = Math.max(1, Number(p[2]) || 1) - 1;
      const c2 = Math.max(c1 + 1, Number(p[3]) || c1 + 1) - 1;
      const dir = (p[5] === 'Y' ? 'Y' : 'D') as 'D' | 'Y';
      const alphabet = p[6] ?? '';
      const rawName = (p[8] ?? '').replace(/^\[|\]$/g, '').trim();

      if (!rawName || rawName.toLowerCase() === 'boş' || alphabet.trim() === '') {
        continue;
      }

      const lower = rawName.toLocaleLowerCase('tr-TR');
      const fieldType: DesignerField['fieldType'] =
        lower.includes('kitap')
          ? 'booklet_type'
          : dir === 'D' || lower.includes('no') || lower.includes('tc') || lower.includes('ad') || lower.includes('kimlik')
            ? 'student_id'
            : 'subject_questions';

      fields.push({
        id: `fmt_${i}`,
        key: rawName.toLocaleLowerCase('tr-TR').replace(/[^a-z0-9çğıöşü]+/gi, '_'),
        label: rawName,
        fieldType,
        direction: dir,
        alphabet,
        startCol: c1,
        startRow: r1,
        cols: c2 - c1 + 1,
        rows: r2 - r1 + 1,
        color: FIELD_COLORS[fields.length % FIELD_COLORS.length],
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
  return { rows, cols, fields };
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
): Record<string, unknown> {
  const maxRows = Math.max(1, Math.round(tpl.rows));
  const maxCols = Math.max(1, Math.round(tpl.cols));
  const usedKeys = new Set<string>();

  const fields = tpl.fields.map((f, idx) => {
    let baseKey = (f.key || `field_${idx + 1}`).trim() || `field_${idx + 1}`;
    let uniqueKey = baseKey;
    let suffix = 2;
    while (usedKeys.has(uniqueKey)) {
      uniqueKey = `${baseKey}_${suffix++}`;
    }
    usedKeys.add(uniqueKey);

    const alphaChars = Array.from(f.alphabet || 'ABCDE');
    const items: Array<{ index: number; options: Array<{ label: string; row: number; col: number }> }> = [];

    if (f.direction === 'D') {
      // Vertical digits/letters: each column is one item, rows are options
      for (let c = 0; c < f.cols; c++) {
        const col = f.startCol + c;
        if (col < 0 || col >= maxCols) continue;
        const options: Array<{ label: string; row: number; col: number }> = [];
        for (let r = 0; r < f.rows; r++) {
          const row = f.startRow + r;
          if (row < 0 || row >= maxRows) continue;
          options.push({
            label: alphaChars[r] ?? String(r),
            row,
            col,
          });
        }
        if (options.length > 0) {
          items.push({ index: c + 1, options });
        }
      }
    } else {
      // Horizontal questions: each row is one item, columns are options
      for (let r = 0; r < f.rows; r++) {
        const row = f.startRow + r;
        if (row < 0 || row >= maxRows) continue;
        const options: Array<{ label: string; row: number; col: number }> = [];
        for (let c = 0; c < f.cols; c++) {
          const col = f.startCol + c;
          if (col < 0 || col >= maxCols) continue;
          options.push({
            label: alphaChars[c] ?? String.fromCharCode(65 + (c % 26)),
            row,
            col,
          });
        }
        if (options.length > 0) {
          items.push({ index: r + 1, options });
        }
      }
    }

    return {
      key: uniqueKey,
      field_type: f.fieldType,
      side: 'front',
      required_for_valid_sheet:
        f.fieldType === 'student_id' &&
        (uniqueKey.includes('ogrenci') || uniqueKey.includes('öğrenci') || uniqueKey.includes('student')),
      items,
    };
  });

  const colorLower = dropoutLed.toLowerCase();
  const dropout_color =
    colorLower === 'red' || colorLower === 'green' || colorLower === 'blue' ? colorLower : 'none';

  return {
    schema_version: 1,
    form_id: tpl.id || 'CUSTOM-FORM',
    form_name: tpl.name || 'Aktif Form',
    duplex,
    dropout_color,
    grid: {
      front_rows: maxRows,
      front_cols: maxCols,
      back_rows: duplex ? maxRows : 0,
      back_cols: duplex ? maxCols : 0,
      width_mm: 210.0,
      height_mm: 297.0,
    },
    threshold_policy: {
      min_mark_density: policy.minMarkDensity,
      strong_mark_density: policy.strongMarkDensity,
      erasure_ratio: policy.erasureRatio,
      review_confidence_floor: policy.confidenceFloor,
    },
    fields,
  };
}

