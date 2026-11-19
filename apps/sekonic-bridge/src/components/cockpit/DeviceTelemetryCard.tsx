import React from 'react';
import { Cpu, Download, Gauge, RefreshCw, ShieldCheck, AlertTriangle } from 'lucide-react';
import { AgentInfo } from '../../services/localAgentClient';

export interface PortOption {
  id: string;
  label: string;
  kind: 'serial' | 'usb';
}

interface DeviceTelemetryCardProps {
  scanning: boolean;
  simulatorMode: boolean;
  onToggleSimulator: (v: boolean) => void;
  dropoutLed: string;
  onDropoutChange: (c: string) => void;
  ports: PortOption[];
  selectedPort: string;
  onSelectPort: (id: string) => void;
  baudRate: number;
  onBaudRateChange: (b: number) => void;
  onRefreshPorts: () => void;
  errorMsg: string | null;
  agentInfo: AgentInfo;
  onReconnectAgent: () => void;
}

export const DeviceTelemetryCard: React.FC<DeviceTelemetryCardProps> = ({
  scanning,
  simulatorMode,
  onToggleSimulator,
  dropoutLed,
  onDropoutChange,
  ports,
  selectedPort,
  onSelectPort,
  baudRate,
  onBaudRateChange,
  onRefreshPorts,
  errorMsg,
  agentInfo,
  onReconnectAgent,
}) => {
  const isBrowserWithoutAgent = agentInfo.state === 'disconnected' || agentInfo.state === 'connecting';

  return (
    <div className="uxigl-glass-card" style={{ padding: 14, display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontWeight: 600, fontSize: 13, display: 'flex', alignItems: 'center', gap: 6 }}>
          <Cpu size={15} color="#38bdf8" /> Cihaz & Port Bağlantısı
        </span>
        <span style={{ fontSize: 12, color: scanning ? '#34d399' : '#94a3b8' }}>
          {scanning ? '● Aktif' : '○ Beklemede'}
        </span>
      </div>

      <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 12, cursor: 'pointer' }}>
        <span style={{ color: '#cbd5e1' }}>Sanal Simülatör Modu</span>
        <input
          type="checkbox"
          checked={simulatorMode}
          onChange={(e) => onToggleSimulator(e.target.checked)}
        />
      </label>

      {!simulatorMode && (
        <div className="uxigl-subcard" style={{ display: 'flex', flexDirection: 'column', gap: 8, padding: 10 }}>
          {agentInfo.state === 'connected' && (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: 6,
                fontSize: 11,
                color: '#10b981',
                background: 'rgba(16, 185, 129, 0.12)',
                padding: '5px 8px',
                borderRadius: 6,
                fontWeight: 600,
              }}
            >
              <ShieldCheck size={13} />
              <span>Yerel Ajan Bağlı {agentInfo.version ? `(v${agentInfo.version})` : ''}</span>
            </div>
          )}

          {isBrowserWithoutAgent && (
            <div
              style={{
                display: 'flex',
                flexDirection: 'column',
                gap: 6,
                fontSize: 11,
                color: '#b45309',
                background: 'rgba(245, 158, 11, 0.14)',
                border: '1px solid rgba(245, 158, 11, 0.35)',
                padding: '8px 9px',
                borderRadius: 6,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontWeight: 600 }}>
                <AlertTriangle size={13} />
                <span>
                  {agentInfo.state === 'connecting'
                    ? 'Yerel Ajan aranıyor...'
                    : 'Sekonic Yerel Ajanı çalışmıyor'}
                </span>
              </div>
              <div style={{ lineHeight: 1.4, opacity: 0.9 }}>
                Windows 7/8.1/10/11 üzerinden COM/USB Sekonic cihazına bağlanmak için yerel ajanı çalıştırın.
              </div>
              <div style={{ display: 'flex', gap: 6, marginTop: 2 }}>
                <a
                  href="/downloads/sekonic-agent.exe"
                  download="sekonic-agent.exe"
                  className="uxigl-btn"
                  style={{
                    padding: '4px 8px',
                    fontSize: 11,
                    textDecoration: 'none',
                    display: 'inline-flex',
                    alignItems: 'center',
                    gap: 4,
                    flex: 1,
                    justifyContent: 'center',
                  }}
                >
                  <Download size={12} />
                  <span>Ajanı İndir (.exe)</span>
                </a>
                <button
                  type="button"
                  className="uxigl-btn"
                  style={{ padding: '4px 8px', fontSize: 11 }}
                  onClick={onReconnectAgent}
                >
                  Tekrar Dene
                </button>
              </div>
            </div>
          )}

          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 11, color: 'var(--uxigl-text-muted)' }}>
            <span>Gerçek COM / USB Aygıtı</span>
            <button
              className="uxigl-btn"
              style={{ padding: '2px 7px', fontSize: 11 }}
              onClick={onRefreshPorts}
              title="Portları Yenile"
            >
              <RefreshCw size={11} /> Yenile
            </button>
          </div>
          <select
            className="uxigl-select"
            value={selectedPort}
            onChange={(e) => onSelectPort(e.target.value)}
          >
            {ports.length === 0 ? (
              <option value="">Bağlı COM/USB aygıtı bulunamadı</option>
            ) : (
              ports.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.label}
                </option>
              ))
            )}
          </select>

          {selectedPort.startsWith('serial:') && (
            <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 12 }}>
              <span style={{ color: 'var(--uxigl-text-muted)' }}>Baud Rate</span>
              <select
                className="uxigl-select"
                value={baudRate}
                onChange={(e) => onBaudRateChange(Number(e.target.value))}
              >
                {[9600, 19200, 38400, 57600, 115200].map((b) => (
                  <option key={b} value={b}>{b} bps</option>
                ))}
              </select>
            </label>
          )}
        </div>
      )}

      {errorMsg && (
        <div style={{ fontSize: 11, color: '#ef4444', background: 'rgba(239,68,68,0.12)', padding: '6px 8px', borderRadius: 6 }}>
          {errorMsg}
        </div>
      )}

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8, fontSize: 12 }}>
        <div className="uxigl-subcard" style={{ padding: 8 }}>
          <div style={{ color: 'var(--uxigl-text-muted)' }}>Okuma Hızı</div>
          <div style={{ fontWeight: 700, fontSize: 15, display: 'flex', alignItems: 'center', gap: 4 }}>
            <Gauge size={14} color="#10b981" /> {scanning ? '108 kağıt/dk' : '0 kağıt/dk'}
          </div>
        </div>
        <div className="uxigl-subcard" style={{ padding: 8 }}>
          <div style={{ color: 'var(--uxigl-text-muted)' }}>Hopper</div>
          <div style={{ fontWeight: 700, fontSize: 15 }}>Dolu</div>
        </div>
      </div>

      <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 12 }}>
        <span style={{ color: 'var(--uxigl-text-muted)' }}>Dropout LED Rengi</span>
        <select
          className="uxigl-select"
          value={dropoutLed}
          onChange={(e) => onDropoutChange(e.target.value)}
        >
          <option value="Red">Kırmızı (Red)</option>
          <option value="Green">Yeşil (Green)</option>
          <option value="Blue">Mavi (Blue)</option>
        </select>
      </label>
    </div>
  );
};
