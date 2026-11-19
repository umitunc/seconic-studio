import React, { useEffect, useRef, useState } from 'react';
import { Play, Square, Wifi, Usb, Download, ChevronDown, FileSpreadsheet, FileText, FileJson } from 'lucide-react';
import { NavTab } from './SidebarNav';
import { ExportFormat } from '../../utils/exportScans';

interface TopControlBarProps {
  activeTab: NavTab;
  scanning: boolean;
  simulatorMode: boolean;
  selectedPort: string;
  rowCount: number;
  onStart: () => void;
  onStop: () => void;
  onExport: (format: ExportFormat) => void;
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
  rowCount,
  onStart,
  onStop,
  onExport,
}) => {
  const [exportOpen, setExportOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setExportOpen(false);
      }
    };
    document.addEventListener('mousedown', handleOutside);
    return () => document.removeEventListener('mousedown', handleOutside);
  }, []);

  const handleSelectExport = (format: ExportFormat) => {
    setExportOpen(false);
    onExport(format);
  };

  return (
    <header
      className="uxigl-glass-card"
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '10px 18px',
        gap: 12,
        position: 'relative',
        zIndex: 20,
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
          <div ref={dropdownRef} style={{ position: 'relative' }}>
            <button
              type="button"
              className="uxigl-btn"
              disabled={rowCount === 0}
              title={rowCount === 0 ? 'Dışa aktarmak için önce okuma başlatın' : `${rowCount} kaydı dışa aktar`}
              onClick={() => setExportOpen((v) => !v)}
              style={{ opacity: rowCount === 0 ? 0.5 : 1 }}
            >
              <Download size={14} />
              <span>Dışa Aktar</span>
              <ChevronDown size={13} />
            </button>

            {exportOpen && rowCount > 0 && (
              <div
                className="uxigl-glass-card"
                style={{
                  position: 'absolute',
                  top: 'calc(100% + 6px)',
                  right: 0,
                  minWidth: 180,
                  padding: 6,
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 4,
                  zIndex: 50,
                  boxShadow: '0 10px 30px rgba(0, 0, 0, 0.25)',
                }}
              >
                <button
                  type="button"
                  className="uxigl-btn"
                  style={{ justifyContent: 'flex-start', width: '100%' }}
                  onClick={() => handleSelectExport('xlsx')}
                >
                  <FileSpreadsheet size={14} />
                  <span>Excel (.xlsx)</span>
                </button>
                <button
                  type="button"
                  className="uxigl-btn"
                  style={{ justifyContent: 'flex-start', width: '100%' }}
                  onClick={() => handleSelectExport('txt')}
                >
                  <FileText size={14} />
                  <span>Metin (.txt)</span>
                </button>
                <button
                  type="button"
                  className="uxigl-btn"
                  style={{ justifyContent: 'flex-start', width: '100%' }}
                  onClick={() => handleSelectExport('json')}
                >
                  <FileJson size={14} />
                  <span>JSON (.json)</span>
                </button>
              </div>
            )}
          </div>

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
