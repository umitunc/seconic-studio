import * as XLSX from 'xlsx';
import { ScannedRow } from '../types/scanner';
import { FormTemplate } from './formTemplates';

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
    const raw =
      field.fieldType === 'subject_questions'
        ? (row.answers?.[field.key] ?? '')
        : (row.decodedFields?.[field.key] ?? '');
    record[field.label] = field.fieldType === 'subject_questions' ? raw : raw.trim();
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

  const lines = ordered.map((r) => {
    if (hasTemplate && template) {
      return template.fields
        .map((field) => {
          const width = field.direction === 'D' ? field.cols : field.rows;
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

  const blob = new Blob([lines.join('\r\n')], {
    type: 'text/plain;charset=utf-8',
  });
  triggerDownload(blob, `${baseName}.txt`);
}

