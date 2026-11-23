import React, { useEffect, useRef, useState } from 'react';
import {
  Play,
  Square,
  Wifi,
  WifiOff,
  Usb,
  Download,
  RefreshCw,
  ChevronDown,
  FileSpreadsheet,
  FileText,
  FileJson,
  Palette,
  Loader2,
  Bug,
  Check,
} from 'lucide-react';
import { NavTab } from './SidebarNav';
import { ExportFormat } from '../../utils/exportScans';
import { AgentInfo, AgentUpdateInfo, ScanPhase } from '../../services/localAgentClient';

interface TopControlBarProps {
  activeTab: NavTab;
  scanning: boolean;
  scanPhase: ScanPhase;
  simulatorMode: boolean;
  selectedPort: string;
  activeTemplateName: string;
  rowCount: number;
  agentInfo: AgentInfo;
  agentUpdate: AgentUpdateInfo;
  onUpdateAgent: () => void;
  themeStudioOpen: boolean;
  onToggleThemeStudio: () => void;
  onCopyDiagnostic: () => void;
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
  scanPhase,
  simulatorMode,
  selectedPort,
  activeTemplateName,
  rowCount,
  agentInfo,
  agentUpdate,
  onUpdateAgent,
  themeStudioOpen,
  onToggleThemeStudio,
  onCopyDiagnostic,
  onStart,
  onStop,
  onExport,
}) => {
  const [exportOpen, setExportOpen] = useState(false);
  const [diagCopied, setDiagCopied] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const handleCopyDiag = () => {
    onCopyDiagnostic();
    setDiagCopied(true);
    window.setTimeout(() => setDiagCopied(false), 1800);
  };

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
        <span className="uxigl-pill" title={`Aktif Optik Form: ${activeTemplateName}`}>
          <FileText size={13} />
          <span style={{ maxWidth: 180, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {activeTemplateName || 'Form Seçilmedi'}
          </span>
        </span>
        <span className="uxigl-pill">
          <Usb size={13} />
          {simulatorMode
            ? 'Simülatör Modu'
            : selectedPort
              ? selectedPort.replace('serial:', 'COM: ').replace('usb:', 'USB: ')
              : 'Port Seçilmedi'}
        </span>
        <span
          className="uxigl-pill"
          style={{
            color:
              agentInfo.state === 'connected' || agentInfo.state === 'tauri'
                ? '#10b981'
                : '#d97706',
          }}
        >
          {agentInfo.state === 'connected' || agentInfo.state === 'tauri' ? (
            <Wifi size={13} />
          ) : (
            <WifiOff size={13} />
          )}
          {agentInfo.state === 'tauri'
            ? 'Masaüstü Köprü Hazır'
            : agentInfo.state === 'connected'
              ? `Yerel Ajan Hazır${agentInfo.version ? ` v${agentInfo.version}` : ''}`
              : agentInfo.state === 'connecting'
                ? 'Ajan Aranıyor...'
                : 'Yerel Ajan Gerekli'}
        </span>
        {agentInfo.state === 'connected' && (agentUpdate.available || agentUpdate.updating) && (
          <button
            type="button"
            className="uxigl-btn"
            disabled={agentUpdate.updating}
            onClick={onUpdateAgent}
            title={
              agentUpdate.latestVersion
                ? `Ajanı v${agentUpdate.latestVersion} sürümüne güncelle`
                : 'Sekonic Yerel Ajanını güncelle'
            }
            style={{
              padding: '4px 10px',
              fontSize: 12,
              background: 'rgba(14, 165, 233, 0.16)',
              borderColor: 'rgba(14, 165, 233, 0.45)',
              color: '#0284c7',
              fontWeight: 600,
            }}
          >
            {agentUpdate.updating ? (
              <>
                <Loader2 size={13} className="uxigl-spin" />
                <span>Ajan Güncelleniyor...</span>
              </>
            ) : (
              <>
                <RefreshCw size={13} />
                <span>
                  Ajanı Güncelle{agentUpdate.latestVersion ? ` (v${agentUpdate.latestVersion})` : ''}
                </span>
              </>
            )}
          </button>
        )}
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        {activeTab === 'scan' && (
          <>
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

            {scanPhase === 'initializing' ? (
              <button className="uxigl-btn uxigl-btn-primary" disabled>
                <Loader2 size={14} className="uxigl-spin" /> Cihaz Hazırlanıyor...
              </button>
            ) : !scanning ? (
              <button className="uxigl-btn uxigl-btn-primary" onClick={onStart}>
                <Play size={14} /> Okumayı Başlat
              </button>
            ) : (
              <button className="uxigl-btn uxigl-btn-danger" onClick={onStop}>
                <Square size={14} /> Durdur
              </button>
            )}
          </>
        )}

        <button
          type="button"
          className="uxigl-btn"
          onClick={handleCopyDiag}
          title="Ham cihaz ve okuma tanı bilgisini panoya kopyala"
          style={{ padding: '7px 11px', borderRadius: 999 }}
        >
          {diagCopied ? <Check size={14} color="#10b981" /> : <Bug size={14} />}
          <span>{diagCopied ? 'Kopyalandı' : 'Tanı'}</span>
        </button>

        <button
          type="button"
          className={`uxigl-btn ${themeStudioOpen ? 'uxigl-btn-primary' : ''}`}
          onClick={onToggleThemeStudio}
          title="uxiGL Tema Stüdyosu"
          style={{ padding: '7px 12px', borderRadius: 999 }}
        >
          <Palette size={14} />
          <span>Tema</span>
        </button>
      </div>
    </header>
  );
};
