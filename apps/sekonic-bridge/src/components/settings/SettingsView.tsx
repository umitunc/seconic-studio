import React from 'react';
import { RefreshCw } from 'lucide-react';
import { PortOption } from '../cockpit/DeviceTelemetryCard';
import { ThresholdPolicy } from '../../types/scanner';

export interface HardwareSettings {
  scannerModel: 'sekonic' | 'sr410';
  separationEnabled: boolean;
  separationColumn: number;
  duplex: boolean;
  subStackerEnabled: boolean;
  stopOnError: boolean;
  manualFeedTimeoutSec: number;
  densityEnabled: boolean;
  densityUpperLimit: number;
  barcodeEnabled: boolean;
  language: 'tr' | 'en';
  printerEnabled: boolean;
  printerPrintSeq: boolean;
  printerPrintOkundu: boolean;
  printerPrintDateTime: boolean;
  printerPrintOkunmadi: boolean;
  printerPrintFileName: boolean;
  printerPosMm: number;
  printerText: string;
  assessmentApiUrl: string;
}

interface SettingsViewProps {
  simulatorMode: boolean;
  onToggleSimulator: (v: boolean) => void;
  ports: PortOption[];
  selectedPort: string;
  onSelectPort: (id: string) => void;
  baudRate: number;
  onBaudRateChange: (b: number) => void;
  onRefreshPorts: () => void;
  dropoutLed: string;
  onDropoutChange: (c: string) => void;
  policy: ThresholdPolicy;
  onPolicyChange: (p: ThresholdPolicy) => void;
  hw: HardwareSettings;
  onHwChange: (h: HardwareSettings) => void;
}

export const SettingsView: React.FC<SettingsViewProps> = ({
  simulatorMode,
  onToggleSimulator,
  ports,
  selectedPort,
  onSelectPort,
  baudRate,
  onBaudRateChange,
  onRefreshPorts,
  dropoutLed,
  onDropoutChange,
  policy,
  onPolicyChange,
  hw,
  onHwChange,
}) => {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 14 }}>
      {/* 1. Bağlantı & Cihaz */}
      <div className="uxigl-glass-card" style={{ padding: 18, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <div style={{ fontWeight: 700, fontSize: 14 }}>1. Bağlantı & Donanım Portu</div>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Sanal Simülatör Modu (Test)</span>
          <input
            type="checkbox"
            checked={simulatorMode}
            onChange={(e) => onToggleSimulator(e.target.checked)}
          />
        </label>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 12, color: 'var(--uxigl-text-muted)' }}>
            <span>COM / USB Aygıtı</span>
            <button className="uxigl-btn" style={{ padding: '2px 8px', fontSize: 11 }} onClick={onRefreshPorts}>
              <RefreshCw size={11} /> Portları Yenile
            </button>
          </div>
          <select
            className="uxigl-select"
            value={selectedPort}
            disabled={simulatorMode}
            onChange={(e) => onSelectPort(e.target.value)}
          >
            {ports.length === 0 ? (
              <option value="">Bağlı COM/USB aygıtı bulunamadı</option>
            ) : (
              ports.map((p) => (
                <option key={p.id} value={p.id}>{p.label}</option>
              ))
            )}
          </select>
        </div>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Optik Okuyucu Tipi</span>
          <select
            className="uxigl-select"
            value={hw.scannerModel}
            onChange={(e) => onHwChange({ ...hw, scannerModel: e.target.value as 'sekonic' | 'sr410' })}
          >
            <option value="sekonic">Sekonic (SR-55D / 1800 / 3500 / 6000 / 6500)</option>
            <option value="sr410">SR-410</option>
          </select>
        </label>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Okuma Sürati (Baud Rate)</span>
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

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Lisan (Language)</span>
          <select
            className="uxigl-select"
            value={hw.language}
            onChange={(e) => onHwChange({ ...hw, language: e.target.value as 'tr' | 'en' })}
          >
            <option value="tr">Türkçe</option>
            <option value="en">English</option>
          </select>
        </label>
      </div>

      {/* 2. Besleme & Tepsi Kontrolü */}
      <div className="uxigl-glass-card" style={{ padding: 18, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <div style={{ fontWeight: 700, fontSize: 14 }}>2. Besleme, Çift Yüz & Çıkış Tepsisi</div>

        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13, gap: 8 }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <input
              type="checkbox"
              checked={hw.separationEnabled}
              onChange={(e) => onHwChange({ ...hw, separationEnabled: e.target.checked })}
            />
            <span>Ayırma Kullanılsın</span>
          </label>
          <input
            type="number"
            className="uxigl-select"
            style={{ width: 70 }}
            value={hw.separationColumn}
            disabled={!hw.separationEnabled}
            onChange={(e) => onHwChange({ ...hw, separationColumn: Number(e.target.value) })}
          />
        </div>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Çift Çıkış Tepsisi (Sub-Stacker Ayırma)</span>
          <input
            type="checkbox"
            checked={hw.subStackerEnabled}
            onChange={(e) => onHwChange({ ...hw, subStackerEnabled: e.target.checked })}
          />
        </label>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Çift Yüzlü Okuma (Duplex)</span>
          <input
            type="checkbox"
            checked={hw.duplex}
            onChange={(e) => onHwChange({ ...hw, duplex: e.target.checked })}
          />
        </label>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Hata Durumunda Beslemeyi Durdur</span>
          <input
            type="checkbox"
            checked={hw.stopOnError}
            onChange={(e) => onHwChange({ ...hw, stopOnError: e.target.checked })}
          />
        </label>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Elle Besleme Bekleme Süresi (sn)</span>
          <input
            type="number"
            className="uxigl-select"
            style={{ width: 80 }}
            min={1}
            max={30}
            value={hw.manualFeedTimeoutSec}
            onChange={(e) => onHwChange({ ...hw, manualFeedTimeoutSec: Number(e.target.value) })}
          />
        </label>
      </div>

      {/* 3. Optik Koyuluk & Barkod */}
      <div className="uxigl-glass-card" style={{ padding: 18, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <div style={{ fontWeight: 700, fontSize: 14 }}>3. Optik Koyuluk (0–255) & Barkod</div>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Dropout LED Işık Rengi (CIS)</span>
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

        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13, gap: 8 }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
            <input
              type="checkbox"
              checked={hw.densityEnabled}
              onChange={(e) => onHwChange({ ...hw, densityEnabled: e.target.checked })}
            />
            <span>Koyuluk Ayarı (Üst Sınır)</span>
          </label>
          <input
            type="number"
            className="uxigl-select"
            style={{ width: 70 }}
            value={hw.densityUpperLimit}
            disabled={!hw.densityEnabled}
            onChange={(e) => onHwChange({ ...hw, densityUpperLimit: Number(e.target.value) })}
          />
        </div>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Barkod Okuyucu Ünitesi</span>
          <input
            type="checkbox"
            checked={hw.barcodeEnabled}
            onChange={(e) => onHwChange({ ...hw, barcodeEnabled: e.target.checked })}
          />
        </label>

        <label style={{ display: 'flex', flexDirection: 'column', gap: 4, fontSize: 12 }}>
          <span>Minimum İşaret Koyuluğu: {policy.minMarkDensity}</span>
          <input
            type="range"
            min={20}
            max={120}
            value={policy.minMarkDensity}
            onChange={(e) => onPolicyChange({ ...policy, minMarkDensity: Number(e.target.value) })}
          />
        </label>

        <label style={{ display: 'flex', flexDirection: 'column', gap: 4, fontSize: 12 }}>
          <span>Silgi Kurtarma Oranı: {policy.erasureRatio.toFixed(2)}</span>
          <input
            type="range"
            min={0.2}
            max={0.7}
            step={0.02}
            value={policy.erasureRatio}
            onChange={(e) => onPolicyChange({ ...policy, erasureRatio: Number(e.target.value) })}
          />
        </label>
      </div>

      {/* 4. Kağıt Üstü Yazıcı & Ölçme App Entegrasyonu */}
      <div className="uxigl-glass-card" style={{ padding: 18, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <div style={{ fontWeight: 700, fontSize: 14 }}>4. Kağıt Üstü Yazıcı & Ölçme App Bağlantısı</div>

        <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 13 }}>
          <span>Endorser Yazıcı Kullanılsın</span>
          <input
            type="checkbox"
            checked={hw.printerEnabled}
            onChange={(e) => onHwChange({ ...hw, printerEnabled: e.target.checked })}
          />
        </label>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 6, fontSize: 12 }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
            <input type="checkbox" checked={hw.printerPrintSeq} onChange={(e) => onHwChange({ ...hw, printerPrintSeq: e.target.checked })} />
            <span>Sıra No</span>
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
            <input type="checkbox" checked={hw.printerPrintOkundu} onChange={(e) => onHwChange({ ...hw, printerPrintOkundu: e.target.checked })} />
            <span>Okundu</span>
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
            <input type="checkbox" checked={hw.printerPrintDateTime} onChange={(e) => onHwChange({ ...hw, printerPrintDateTime: e.target.checked })} />
            <span>Tarih & Saat</span>
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
            <input type="checkbox" checked={hw.printerPrintOkunmadi} onChange={(e) => onHwChange({ ...hw, printerPrintOkunmadi: e.target.checked })} />
            <span>Okunmadı</span>
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: 5 }}>
            <input type="checkbox" checked={hw.printerPrintFileName} onChange={(e) => onHwChange({ ...hw, printerPrintFileName: e.target.checked })} />
            <span>Dosya Adı</span>
          </label>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
          <label style={{ display: 'flex', flexDirection: 'column', gap: 4, fontSize: 12 }}>
            <span>Pozisyon (mm)</span>
            <input
              type="number"
              className="uxigl-select"
              value={hw.printerPosMm}
              onChange={(e) => onHwChange({ ...hw, printerPosMm: Number(e.target.value) })}
            />
          </label>
          <label style={{ display: 'flex', flexDirection: 'column', gap: 4, fontSize: 12 }}>
            <span>Baskı Metni</span>
            <input
              type="text"
              className="uxigl-select"
              value={hw.printerText}
              onChange={(e) => onHwChange({ ...hw, printerText: e.target.value })}
            />
          </label>
        </div>

        <label style={{ display: 'flex', flexDirection: 'column', gap: 4, fontSize: 12 }}>
          <span>Ölçme-Değerlendirme WebSocket API Adresi</span>
          <input
            type="text"
            className="uxigl-select"
            value={hw.assessmentApiUrl}
            onChange={(e) => onHwChange({ ...hw, assessmentApiUrl: e.target.value })}
          />
        </label>
      </div>
    </div>
  );
};
