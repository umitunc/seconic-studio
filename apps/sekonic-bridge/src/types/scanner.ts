export type StackerRoute = 'main' | 'sub';
export type SheetStatus =
  | 'Tamam'
  | 'Silgi Kurtarıldı'
  | 'Çift İşaret'
  | 'Kayıtsız'
  | 'Ters Yön'
  | 'Çizgi Hatası';

export interface QuestionDetail {
  q: number;
  answer: string;
  state: 'ok' | 'erased' | 'multi' | 'blank';
  density: number;
  erasedOption?: string;
}

export interface ScannedRow {
  id: number;
  seqNo: number;
  studentId: string;
  studentName: string;
  booklet: string;
  stacker: StackerRoute;
  confidence: number;
  status: SheetStatus;
  answersSummary: string;
  decodedFields?: Record<string, string>;
  answers?: Record<string, string>;
  warnings?: string[];
  questionDetails?: QuestionDetail[];
  rawMatrixB64: string;
  rawWireSample?: string;
  flaggedQuestion?: {
    q: number;
    densities: number[];
    erasedLabel?: string;
    selectedLabel?: string;
  };
}

export interface ThresholdPolicy {
  minMarkDensity: number;
  strongMarkDensity: number;
  erasureRatio: number;
  confidenceFloor: number;
}
