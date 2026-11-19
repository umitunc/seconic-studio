import { PortOption } from '../components/cockpit/DeviceTelemetryCard';

export type AgentConnectionState = 'tauri' | 'connected' | 'connecting' | 'disconnected';

export interface AgentInfo {
  state: AgentConnectionState;
  version?: string;
  platform?: string;
}

export interface RawGradedSheet {
  seq_no: number;
  decoded_fields?: Record<string, string>;
  answers?: Record<string, string>;
  missing_required_field?: boolean;
  needs_operator_review?: boolean;
  warnings?: string[];
  raw_matrix_b64?: string;
}

interface PendingReq {
  resolve: (msg: Record<string, unknown>) => void;
  reject: (err: Error) => void;
  timer: number;
}

export interface LocalAgentHandlers {
  onStatusChange?: (info: AgentInfo) => void;
  onSheetScanned?: (sheet: RawGradedSheet) => void;
  onScannerError?: (message: string) => void;
  onScannerStatus?: (scanning: boolean) => void;
  onHopperEmpty?: () => void;
}

const DEFAULT_WS_URL = 'ws://127.0.0.1:17441';
const RECONNECT_MS = 3500;
const REQUEST_TIMEOUT_MS = 5000;

export class LocalAgentClient {
  private ws: WebSocket | null = null;
  private url: string;
  private handlers: LocalAgentHandlers;
  private reconnectTimer: number | null = null;
  private disposed = false;
  private reqCounter = 0;
  private pending = new Map<string, PendingReq>();
  private info: AgentInfo = { state: 'disconnected' };

  constructor(handlers: LocalAgentHandlers, url = DEFAULT_WS_URL) {
    this.handlers = handlers;
    this.url = url;
  }

  public getInfo(): AgentInfo {
    return this.info;
  }

  public isConnected(): boolean {
    return this.ws !== null && this.ws.readyState === WebSocket.OPEN;
  }

  public connect(): void {
    if (this.disposed) return;
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }
    this.clearReconnectTimer();
    this.updateInfo({ state: 'connecting' });

    try {
      const socket = new WebSocket(this.url);
      this.ws = socket;

      socket.onopen = () => {
        if (this.ws !== socket) return;
        this.updateInfo({ state: 'connected', version: this.info.version, platform: this.info.platform });
      };

      socket.onmessage = (ev) => {
        if (this.ws !== socket) return;
        this.handleMessage(String(ev.data));
      };

      socket.onerror = () => {
        // onclose will follow and schedule reconnect
      };

      socket.onclose = () => {
        if (this.ws !== socket) return;
        this.ws = null;
        this.rejectAllPending(new Error('Yerel ajan bağlantısı kapandı.'));
        this.updateInfo({ state: 'disconnected' });
        this.scheduleReconnect();
      };
    } catch {
      this.updateInfo({ state: 'disconnected' });
      this.scheduleReconnect();
    }
  }

  public reconnectNow(): void {
    this.clearReconnectTimer();
    if (this.ws) {
      try {
        this.ws.close();
      } catch {
        // ignore
      }
      this.ws = null;
    }
    this.connect();
  }

  public dispose(): void {
    this.disposed = true;
    this.clearReconnectTimer();
    this.rejectAllPending(new Error('İstemci kapatıldı.'));
    if (this.ws) {
      try {
        this.ws.close();
      } catch {
        // ignore
      }
      this.ws = null;
    }
  }

  public async listPorts(): Promise<PortOption[]> {
    const res = await this.sendRequest({ type: 'list_ports' });
    const ports = Array.isArray(res.ports) ? (res.ports as PortOption[]) : [];
    return ports;
  }

  public async startScan(opts: { portId: string; baudRate: number; simulatorMode: boolean }): Promise<void> {
    await this.sendRequest({
      type: 'start_scan',
      port_id: opts.portId,
      baud_rate: opts.baudRate,
      simulator_mode: opts.simulatorMode,
    });
  }

  public async stopScan(): Promise<void> {
    await this.sendRequest({ type: 'stop_scan' });
  }

  private sendRequest(payload: Record<string, unknown>): Promise<Record<string, unknown>> {
    return new Promise((resolve, reject) => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error('Yerel Sekonic Ajanı bağlı değil.'));
        return;
      }
      const reqId = `req_${++this.reqCounter}`;
      const timer = window.setTimeout(() => {
        this.pending.delete(reqId);
        reject(new Error('Yerel ajan yanıt vermedi (zaman aşımı).'));
      }, REQUEST_TIMEOUT_MS);

      this.pending.set(reqId, { resolve, reject, timer });
      try {
        this.ws.send(JSON.stringify({ ...payload, req_id: reqId }));
      } catch (err) {
        window.clearTimeout(timer);
        this.pending.delete(reqId);
        reject(err instanceof Error ? err : new Error(String(err)));
      }
    });
  }

  private handleMessage(raw: string): void {
    let msg: Record<string, unknown>;
    try {
      msg = JSON.parse(raw) as Record<string, unknown>;
    } catch {
      return;
    }

    const reqId = typeof msg.req_id === 'string' ? msg.req_id : null;
    if (reqId && this.pending.has(reqId)) {
      const entry = this.pending.get(reqId)!;
      this.pending.delete(reqId);
      window.clearTimeout(entry.timer);
      if (msg.type === 'error') {
        entry.reject(new Error(String(msg.message ?? 'Bilinmeyen ajan hatası')));
      } else {
        entry.resolve(msg);
      }
      return;
    }

    switch (msg.type) {
      case 'hello':
        this.updateInfo({
          state: 'connected',
          version: typeof msg.version === 'string' ? msg.version : undefined,
          platform: typeof msg.platform === 'string' ? msg.platform : undefined,
        });
        if (typeof msg.scanning === 'boolean') {
          this.handlers.onScannerStatus?.(msg.scanning);
        }
        break;
      case 'sheet:scanned':
        if (msg.payload && typeof msg.payload === 'object') {
          this.handlers.onSheetScanned?.(msg.payload as RawGradedSheet);
        }
        break;
      case 'scanner:error':
        this.handlers.onScannerError?.(String(msg.message ?? 'Tarayıcı hatası'));
        break;
      case 'scanner:status':
        if (typeof msg.scanning === 'boolean') {
          this.handlers.onScannerStatus?.(msg.scanning);
        }
        break;
      case 'scanner:hopper_empty':
        this.handlers.onHopperEmpty?.();
        break;
    }
  }

  private updateInfo(next: AgentInfo): void {
    if (
      this.info.state === next.state &&
      this.info.version === next.version &&
      this.info.platform === next.platform
    ) {
      return;
    }
    this.info = next;
    this.handlers.onStatusChange?.(next);
  }

  private scheduleReconnect(): void {
    if (this.disposed || this.reconnectTimer !== null) return;
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, RECONNECT_MS);
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer !== null) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  private rejectAllPending(err: Error): void {
    for (const [, entry] of this.pending) {
      window.clearTimeout(entry.timer);
      entry.reject(err);
    }
    this.pending.clear();
  }
}

