export interface ScannedSheetEvent {
  event: 'sheet.scanned';
  session_id: string;
  seq_no: number;
  timestamp_ms: number;
  scanner: {
    model: string;
    serial_or_id: string;
    stacker_routed: 'main' | 'sub';
  };
  barcode?: string | null;
  validation: {
    status: 'valid' | 'review' | 'rejected';
    matched_student_id?: string;
    needs_operator_review: boolean;
    warnings: string[];
  };
  decoded_fields: Record<string, string>;
  answers: Record<string, string>;
  raw_matrix_b64: string;
}

export interface ConnectOptions {
  preferBridge?: boolean;
  bridgeUrl?: string;
  bridgeTimeoutMs?: number;
}

export class SekonicClient {
  private ws: WebSocket | null = null;
  private mode: 'bridge' | 'webusb' | 'disconnected' = 'disconnected';
  private listeners: Array<(ev: ScannedSheetEvent) => void> = [];

  get connectionMode(): 'bridge' | 'webusb' | 'disconnected' {
    return this.mode;
  }

  onSheetScanned(cb: (ev: ScannedSheetEvent) => void): () => void {
    this.listeners.push(cb);
    return () => {
      this.listeners = this.listeners.filter((x) => x !== cb);
    };
  }

  async connect(opts: ConnectOptions = {}): Promise<'bridge' | 'webusb'> {
    const preferBridge = opts.preferBridge ?? true;
    const bridgeUrl = opts.bridgeUrl ?? 'ws://127.0.0.1:17441/v1/stream';
    const timeoutMs = opts.bridgeTimeoutMs ?? 300;

    if (preferBridge) {
      const ok = await this.tryConnectBridge(bridgeUrl, timeoutMs);
      if (ok) {
        this.mode = 'bridge';
        return 'bridge';
      }
    }

    const nav = navigator as unknown as { usb?: { requestDevice: (o: unknown) => Promise<unknown> } };
    if (nav.usb) {
      await nav.usb.requestDevice({ filters: [{ vendorId: 0x0a41 }] });
      this.mode = 'webusb';
      return 'webusb';
    }
    throw new Error('Ne yerel sekonic-bridge bulundu ne de tarayıcı WebUSB destekliyor.');
  }

  private tryConnectBridge(url: string, timeoutMs: number): Promise<boolean> {
    return new Promise((resolve) => {
      try {
        const ws = new WebSocket(url);
        const timer = setTimeout(() => {
          ws.close();
          resolve(false);
        }, timeoutMs);

        ws.onopen = () => {
          clearTimeout(timer);
          this.ws = ws;
          resolve(true);
        };
        ws.onerror = () => {
          clearTimeout(timer);
          resolve(false);
        };
        ws.onmessage = (msg) => {
          try {
            const parsed = JSON.parse(String(msg.data)) as ScannedSheetEvent;
            if (parsed.event === 'sheet.scanned') {
              for (const fn of this.listeners) fn(parsed);
            }
          } catch {
            // ignore non-JSON frames
          }
        };
      } catch {
        resolve(false);
      }
    });
  }

  disconnect(): void {
    this.ws?.close();
    this.ws = null;
    this.mode = 'disconnected';
  }
}

/**
 * Renders a 0-255 analog optical density matrix onto an HTML5 <canvas> with
 * green (marked), orange dashed (erasure recovered), and red (multi-mark) highlights.
 */
export function renderSheetHeatmapCanvas(
  canvas: HTMLCanvasElement,
  rawMatrixB64: string,
  rows: number,
  cols: number,
  minMarkDensity = 48,
  strongMarkDensity = 115,
  theme: 'light' | 'dark' = 'light',
): void {
  const ctx = canvas.getContext('2d');
  if (!ctx || rows <= 0 || cols <= 0) return;

  const binStr = atob(rawMatrixB64);
  const w = canvas.width;
  const h = canvas.height;
  ctx.fillStyle = theme === 'dark' ? '#141518' : '#ffffff';
  ctx.fillRect(0, 0, w, h);

  const cellW = w / cols;
  const cellH = h / rows;
  const r = Math.max(2, Math.min(cellW, cellH) * 0.34);

  for (let row = 0; row < rows; row++) {
    for (let col = 0; col < cols; col++) {
      const idx = row * cols + col;
      const val = idx < binStr.length ? binStr.charCodeAt(idx) : 0;
      const cx = col * cellW + cellW / 2;
      const cy = row * cellH + cellH / 2;

      ctx.beginPath();
      ctx.arc(cx, cy, r, 0, Math.PI * 2);
      if (val >= strongMarkDensity) {
        ctx.fillStyle = theme === 'dark' ? '#e4e4e7' : '#18181b';
        ctx.fill();
      } else if (val >= minMarkDensity) {
        ctx.strokeStyle = '#d97706';
        ctx.lineWidth = 1.5;
        ctx.setLineDash([2, 2]);
        ctx.stroke();
        ctx.setLineDash([]);
      } else {
        ctx.strokeStyle = theme === 'dark' ? 'rgba(255, 255, 255, 0.2)' : 'rgba(24, 24, 27, 0.22)';
        ctx.lineWidth = 1;
        ctx.stroke();
      }
    }
  }
}
