import * as XLSX from 'xlsx';
import { ScannedRow } from '../types/scanner';

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

export function exportScannedRows(rows: ScannedRow[], format: ExportFormat): void {
  if (rows.length === 0) return;
  const ordered = [...rows].sort((a, b) => a.seqNo - b.seqNo);
  const baseName = `sekonic-okuma-${timestampSuffix()}`;

  if (format === 'xlsx') {
    const sheetData = ordered.map((r) => ({
      Sıra: r.seqNo,
      'Öğrenci No': r.studentId,
      'Ad Soyad': r.studentName,
      Kitapçık: r.booklet,
      Tepsi: r.stacker === 'main' ? 'Ana' : 'Alt',
      'Güven (%)': r.confidence,
      Durum: r.status,
      'Cevap Özeti': r.answersSummary,
    }));
    const ws = XLSX.utils.json_to_sheet(sheetData);
    const wb = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(wb, ws, 'Okuma Sonuçları');
    XLSX.writeFile(wb, `${baseName}.xlsx`);
    return;
  }

  if (format === 'json') {
    const payload = ordered.map((r) => ({
      seqNo: r.seqNo,
      studentId: r.studentId,
      studentName: r.studentName,
      booklet: r.booklet,
      stacker: r.stacker,
      confidence: r.confidence,
      status: r.status,
      answersSummary: r.answersSummary,
      ...(r.questionDetails ? { questionDetails: r.questionDetails } : {}),
    }));
    const blob = new Blob([JSON.stringify(payload, null, 2)], {
      type: 'application/json;charset=utf-8',
    });
    triggerDownload(blob, `${baseName}.json`);
    return;
  }

  const header = ['SIRA', 'OGRENCI_NO', 'AD_SOYAD', 'KITAPCIK', 'TEPSI', 'GUVEN', 'DURUM', 'CEVAPLAR'].join('\t');
  const lines = ordered.map((r) =>
    [
      r.seqNo,
      r.studentId,
      r.studentName,
      r.booklet,
      r.stacker === 'main' ? 'Ana' : 'Alt',
      `${r.confidence}%`,
      r.status,
      r.answersSummary,
    ].join('\t'),
  );
  const blob = new Blob([[header, ...lines].join('\r\n')], {
    type: 'text/plain;charset=utf-8',
  });
  triggerDownload(blob, `${baseName}.txt`);
}

