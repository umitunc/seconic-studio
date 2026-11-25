import * as XLSX from 'xlsx';
import { ScannedRow } from '../types/scanner';
import { computeFieldOutputWidth, FormTemplate } from './formTemplates';
import { encodeWindows1254 } from './encoding';

export type ExportFormat = 'xlsx' | 'txt' | 'json';

function timestampSuffix(): string {
  const now = new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}_${pad(now.getHours())}-${pad(now.getMinutes())}`;
}

function triggerDownload(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}

function formatFixedWidth(value: string, width: number): string {
  if (width <= 0) return '';
  if (value.length >= width) return value.slice(0, width);
  return value.padEnd(width, ' ');
}

function buildTemplateRecord(row: ScannedRow, template: FormTemplate): Record<string, string> {
  const record: Record<string, string> = {};
  for (const field of template.fields) {
    if (field.hiddenFromOutput) continue;
    const raw =
      field.fieldType === 'subject_questions'
        ? (row.answers?.[field.key] ?? '')
        : (row.decodedFields?.[field.key] ?? '');
    const val = field.fieldType === 'subject_questions' ? raw : raw.trim();
    if (field.label in record) {
      record[field.label] = `${record[field.label]}${val}`;
    } else {
      record[field.label] = val;
    }
  }
  return record;
}

export function exportScannedRows(
  rows: ScannedRow[],
  format: ExportFormat,
  template?: FormTemplate,
): void {
  if (rows.length === 0) return;
  const ordered = [...rows].sort((a, b) => a.seqNo - b.seqNo);
  const baseName = `sekonic-okuma-${timestampSuffix()}`;
  const hasTemplate = Boolean(template && template.fields.length > 0);

  if (format === 'xlsx') {
    const sheetData = ordered.map((r) =>
      hasTemplate && template
        ? buildTemplateRecord(r, template)
        : {
            'Öğrenci No': r.studentId,
            'Ad Soyad': r.studentName,
            Kitapçık: r.booklet,
            Cevaplar: r.answersSummary,
          },
    );
    const ws = XLSX.utils.json_to_sheet(sheetData);
    const wb = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(wb, ws, 'Okuma Sonuçları');
    XLSX.writeFile(wb, `${baseName}.xlsx`);
    return;
  }

  if (format === 'json') {
    const payload = ordered.map((r) =>
      hasTemplate && template
        ? buildTemplateRecord(r, template)
        : {
            studentId: r.studentId,
            studentName: r.studentName,
            booklet: r.booklet,
            answers: r.answersSummary,
          },
    );
    const blob = new Blob([JSON.stringify(payload, null, 2)], {
      type: 'application/json;charset=utf-8',
    });
    triggerDownload(blob, `${baseName}.json`);
    return;
  }

  // Exclude structural error sheets (`Çizgi Hatası` / `Ters Yön`) when valid rows exist,
  // matching `SekOku.exe` which routes bad-line-count sheets to `.hat` instead of `.txt`.
  const validRows = ordered.filter((r) => r.status !== 'Çizgi Hatası' && r.status !== 'Ters Yön');
  const txtRows = validRows.length > 0 ? validRows : ordered;

  const lines = txtRows.map((r) => {
    if (hasTemplate && template) {
      return template.fields
        .filter((field) => !field.hiddenFromOutput)
        .map((field) => {
          const width = computeFieldOutputWidth(field);
          const raw =
            field.fieldType === 'subject_questions'
              ? (r.answers?.[field.key] ?? '')
              : (r.decodedFields?.[field.key] ?? '');
          return formatFixedWidth(raw, width);
        })
        .join('');
    }
    return `${r.studentId}${r.studentName}${r.booklet}${r.answersSummary}`;
  });

  // Match `SekOku.exe` byte-for-byte: Windows-1254 (1 byte per Turkish char) + trailing CRLF.
  const crlfPayload = `${lines.join('\r\n')}\r\n`;
  const ansiBytes = encodeWindows1254(crlfPayload);
  const blob = new Blob([ansiBytes.buffer as ArrayBuffer], {
    type: 'text/plain;charset=windows-1254',
  });
  triggerDownload(blob, `${baseName}.txt`);
}

