import { QuestionDetail, ScannedRow } from '../types/scanner';

function encodeMockMatrixB64(rows: number, cols: number, seed: number): string {
  const bytes = new Uint8Array(rows * cols);
  bytes.fill(10);
  for (let r = 0; r < rows; r++) {
    const c = (r * 7 + seed * 3) % Math.min(5, cols);
    bytes[r * cols + c] = 210;
    if (seed % 4 === 1 && r === 13) {
      bytes[r * cols + ((c + 1) % 5)] = 54;
    }
    if (seed % 7 === 0 && r === 21) {
      bytes[r * cols + ((c + 2) % 5)] = 188;
    }
  }
  let bin = '';
  for (let i = 0; i < bytes.length; i++) bin += String.fromCharCode(bytes[i]);
  return btoa(bin);
}

const OPTS = ['A', 'B', 'C', 'D', 'E'] as const;

export function createSimulatedRow(seqNo: number): ScannedRow {
  const isUnknown = seqNo % 15 === 0;
  const isDouble = !isUnknown && seqNo % 7 === 0;
  const isErasure = !isUnknown && !isDouble && seqNo % 4 === 1;

  const names = ['Ali Yılmaz', 'Ayşe Kaya', 'Mehmet Demir', 'Zeynep Çelik', 'Emre Arslan'];
  const status: ScannedRow['status'] = isUnknown
    ? 'Kayıtsız'
    : isDouble
      ? 'Çift İşaret'
      : isErasure
        ? 'Silgi Kurtarıldı'
        : 'Tamam';

  // Deterministic pseudo-random answers per sheet
  let s = seqNo * 1103515245 + 12345;
  const questionDetails: QuestionDetail[] = [];
  let summary = '';
  for (let q = 1; q <= 20; q++) {
    s = (s * 1664525 + 1013904223) >>> 0;
    const roll = s % 100;
    if (isDouble && q === 12) {
      questionDetails.push({ q, answer: '*', state: 'multi', density: 188 });
      summary += '*';
    } else if (isErasure && q === 14) {
      const opt = OPTS[s % 5];
      const erased = OPTS[(s + 1) % 5];
      questionDetails.push({ q, answer: opt, state: 'erased', density: 208, erasedOption: erased });
      summary += opt;
    } else if (roll < 6) {
      questionDetails.push({ q, answer: '-', state: 'blank', density: 12 });
      summary += ' ';
    } else {
      const opt = OPTS[s % 5];
      questionDetails.push({ q, answer: opt, state: 'ok', density: 185 + (s % 50) });
      summary += opt;
    }
    if (q % 5 === 0 && q < 20) summary += ' ';
  }

  return {
    id: seqNo,
    seqNo,
    studentId: isUnknown ? '????????' : String(20261000 + seqNo),
    studentName: isUnknown ? 'Tanımsız Öğrenci' : names[seqNo % names.length],
    booklet: seqNo % 2 === 0 ? 'A' : 'B',
    stacker: isUnknown || isDouble ? 'sub' : 'main',
    confidence: isUnknown ? 42 : isDouble ? 54 : isErasure ? 81 : 96 + (seqNo % 4),
    status,
    answersSummary: summary,
    questionDetails,
    rawMatrixB64: encodeMockMatrixB64(24, 10, seqNo),
    flaggedQuestion: isErasure
      ? { q: 14, densities: [54, 8, 210, 11, 7], erasedLabel: 'A', selectedLabel: 'C' }
      : isDouble
        ? { q: 22, densities: [9, 188, 12, 182, 8] }
        : undefined,
  };
}
