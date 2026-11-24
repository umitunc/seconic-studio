import { PortOption } from '../components/cockpit/DeviceTelemetryCard';

export type AgentConnectionState = 'tauri' | 'connected' | 'connecting' | 'disconnected';
export type ScanPhase = 'idle' | 'initializing' | 'scanning';

export interface AgentInfo {
  state: AgentConnectionState;
  version?: string;
  platform?: string;
}

export interface AgentUpdateInfo {
  available: boolean;
  currentVersion?: string;
  latestVersion?: string;
  downloadUrl?: string;
  updating?: boolean;
  checking?: boolean;
  checkedUpToDate?: boolean;
  error?: string;
}

export function isNewerSemver(latest: string, current: string): boolean {
  const parse = (v: string) =>
    v
      .trim()
      .replace(/^[vV]/, '')
      .split(/[-+]/)[0]
      .split('.')
      .map((n) => parseInt(n, 10) || 0);
  const a = parse(latest);
  const b = parse(current);
  for (let i = 0; i < 3; i++) {
    const av = a[i] ?? 0;
    const bv = b[i] ?? 0;
    if (av > bv) return true;
    if (av < bv) return false;
  }
  return false;
}

export interface RawGradedSheet {
  seq_no: number;
  decoded_fields?: Record<string, string>;
  answers?: Record<string, string>;
  missing_required_field?: boolean;
  needs_operator_review?: boolean;
  warnings?: string[];
  raw_matrix_b64?: string;
  raw_wire_sample?: string;
  observed_front_rows?: number;
}

export interface ScannerDiagnosticInfo {
  modelName?: string;
  rm16Level?: boolean;
  initLog?: string[];
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
  onScannerStatus?: (scanning: boolean, phase?: ScanPhase) => void;
  onScannerDiagnostic?: (diag: ScannerDiagnosticInfo) => void;
  onHopperEmpty?: () => void;
  onAgentUpdate?: (update: AgentUpdateInfo) => void;
}

const DEFAULT_WS_URL = 'ws://127.0.0.1:17441';
const INITIAL_RECONNECT_MS = 3500;
const MAX_RECONNECT_MS = 30000;
const REQUEST_TIMEOUT_MS = 8000;

export class LocalAgentClient {
  private ws: WebSocket | null = null;
  private url: string;
  private handlers: LocalAgentHandlers;
  private reconnectTimer: number | null = null;
  private reconnectDelayMs = INITIAL_RECONNECT_MS;
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
        this.reconnectDelayMs = INITIAL_RECONNECT_MS;
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
    this.reconnectDelayMs = INITIAL_RECONNECT_MS;
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

  public async startScan(opts: {
    portId: string;
    baudRate: number;
    simulatorMode: boolean;
    enableBarcode?: boolean;
    schema?: Record<string, unknown>;
  }): Promise<void> {
    await this.sendRequest({
      type: 'start_scan',
      port_id: opts.portId,
      baud_rate: opts.baudRate,
      simulator_mode: opts.simulatorMode,
      enable_barcode: Boolean(opts.enableBarcode),
      schema: opts.schema ?? null,
    });
  }

  public async stopScan(): Promise<void> {
    await this.sendRequest({ type: 'stop_scan' });
  }

  public async getDiagnostics(): Promise<Record<string, unknown> | null> {
    try {
      const res = await this.sendRequest({ type: 'get_diagnostics' });
      return (res.snapshot as Record<string, unknown>) ?? null;
    } catch {
      return null;
    }
  }

  public async checkUpdate(manifestUrl?: string): Promise<AgentUpdateInfo> {
    const res = await this.sendRequest({
      type: 'check_update',
      manifest_url: manifestUrl ?? null,
    });
    return {
      available: Boolean(res.update_available),
      currentVersion: typeof res.current_version === 'string' ? res.current_version : undefined,
      latestVersion: typeof res.latest_version === 'string' ? res.latest_version : undefined,
      downloadUrl: typeof res.download_url === 'string' ? res.download_url : undefined,
    };
  }

  public async updateAgent(downloadUrl?: string): Promise<void> {
    this.reconnectDelayMs = 800;
    await this.sendRequest({
      type: 'update_agent',
      download_url: downloadUrl ?? null,
    });
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
      case 'scanner:status': {
        const phase =
          msg.phase === 'initializing' || msg.phase === 'scanning' || msg.phase === 'idle'
            ? (msg.phase as ScanPhase)
            : undefined;
        if (typeof msg.scanning === 'boolean' || phase) {
          this.handlers.onScannerStatus?.(Boolean(msg.scanning), phase);
        }
        if (msg.model_name !== undefined || Array.isArray(msg.init_log)) {
          this.handlers.onScannerDiagnostic?.({
            modelName: typeof msg.model_name === 'string' ? msg.model_name : undefined,
            rm16Level: typeof msg.rm_16level === 'boolean' ? msg.rm_16level : undefined,
            initLog: Array.isArray(msg.init_log) ? msg.init_log.map(String) : undefined,
          });
        }
        break;
      }
      case 'scanner:hopper_empty':
        this.handlers.onHopperEmpty?.();
        break;
      case 'agent:update_available':
        this.handlers.onAgentUpdate?.({
          available: true,
          currentVersion: typeof msg.current_version === 'string' ? msg.current_version : undefined,
          latestVersion: typeof msg.latest_version === 'string' ? msg.latest_version : undefined,
          downloadUrl: typeof msg.download_url === 'string' ? msg.download_url : undefined,
          updating: false,
        });
        break;
      case 'agent:updating':
        this.reconnectDelayMs = 800;
        this.handlers.onAgentUpdate?.({
          available: true,
          downloadUrl: typeof msg.download_url === 'string' ? msg.download_url : undefined,
          updating: true,
        });
        break;
      case 'agent:update_error':
        this.handlers.onAgentUpdate?.({
          available: true,
          updating: false,
          error: String(msg.message ?? 'Ajan güncellenemedi'),
        });
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
    const delay = this.reconnectDelayMs;
    this.reconnectDelayMs = Math.min(MAX_RECONNECT_MS, Math.round(this.reconnectDelayMs * 2));
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      this.connect();
    }, delay);
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

