import React from 'react';
import { Cpu, Gauge, RefreshCw } from 'lucide-react';

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
}) => {
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
