import React from 'react';
import { Play, Square, Wifi, Usb } from 'lucide-react';
import { NavTab } from './SidebarNav';

interface TopControlBarProps {
  activeTab: NavTab;
  scanning: boolean;
  simulatorMode: boolean;
  selectedPort: string;
  onStart: () => void;
  onStop: () => void;
}

const TITLES: Record<NavTab, string> = {
  scan: 'Canlı Optik Okuma',
  history: 'Geçmiş Okuma Oturumları',
  forms: 'Form Şablonları (.FMT / JSON)',
  settings: 'Cihaz & Okuma Ayarları',
};

export const TopControlBar: React.FC<TopControlBarProps> = ({
  activeTab,
  scanning,
  simulatorMode,
  selectedPort,
  onStart,
  onStop,
}) => {
  return (
    <header
      className="uxigl-glass-card"
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '10px 18px',
        gap: 12,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <span style={{ fontWeight: 700, fontSize: 15 }}>{TITLES[activeTab]}</span>
        <span className="uxigl-pill">
          <Usb size={13} />
          {simulatorMode
            ? 'Simülatör Modu'
            : selectedPort
              ? selectedPort.replace('serial:', 'COM: ').replace('usb:', 'USB: ')
              : 'Port Seçilmedi'}
        </span>
        <span className="uxigl-pill">
          <Wifi size={13} />
          Ölçme App Hazır
        </span>
      </div>

      {activeTab === 'scan' && (
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          {!scanning ? (
            <button className="uxigl-btn uxigl-btn-primary" onClick={onStart}>
              <Play size={14} /> Okumayı Başlat
            </button>
          ) : (
            <button className="uxigl-btn uxigl-btn-danger" onClick={onStop}>
              <Square size={14} /> Durdur
            </button>
          )}
        </div>
      )}
    </header>
  );
};
