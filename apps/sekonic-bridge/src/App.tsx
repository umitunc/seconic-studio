import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { SidebarNav, NavTab } from './components/layout/SidebarNav';
import { TopControlBar } from './components/layout/TopControlBar';
import { DeviceTelemetryCard, PortOption } from './components/cockpit/DeviceTelemetryCard';
import { KpiStrip } from './components/cockpit/KpiStrip';
import { LiveScanGrid } from './components/cockpit/LiveScanGrid';
import { AnswerDetailsPanel } from './components/cockpit/AnswerDetailsPanel';
import { SheetCanvasPreview } from './components/cockpit/SheetCanvasPreview';
import { ExceptionAlertBar } from './components/cockpit/ExceptionAlertBar';
import { HeatmapReviewModal } from './components/review/HeatmapReviewModal';
import { FormDesignerView } from './components/designer/FormDesignerView';
import { ScanHistoryView } from './components/history/ScanHistoryView';
import { SettingsView, HardwareSettings } from './components/settings/SettingsView';
import { createSimulatedRow } from './services/bridgeClient';
import { ScannedRow, ThresholdPolicy } from './types/scanner';

type TauriInternals = {
  invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
};

function getTauriInvoke(): TauriInternals['invoke'] | null {
  const w = window as unknown as { __TAURI_INTERNALS__?: TauriInternals };
  return w.__TAURI_INTERNALS__?.invoke ?? null;
}

const ROUTE_TO_TAB: Record<string, NavTab> = {
  '/': 'scan',
  '/scan': 'scan',
  '/history': 'history',
  '/forms': 'forms',
  '/settings': 'settings',
};

const TAB_TO_ROUTE: Record<NavTab, string> = {
  scan: '/scan',
  history: '/history',
  forms: '/forms',
  settings: '/settings',
};

function readTabFromUrl(): NavTab {
  return ROUTE_TO_TAB[window.location.pathname] ?? 'scan';
}

export const App: React.FC = () => {
  const [activeTab, setActiveTabState] = useState<NavTab>(readTabFromUrl);

  const setActiveTab = useCallback((tab: NavTab) => {
    setActiveTabState(tab);
    const targetUrl = TAB_TO_ROUTE[tab];
    if (window.location.pathname !== targetUrl) {
      window.history.pushState({ tab }, '', targetUrl);
    }
  }, []);

  useEffect(() => {
    if (window.location.pathname === '/') {
      window.history.replaceState({ tab: 'scan' }, '', '/scan');
    }
    const onPopState = () => {
      setActiveTabState(readTabFromUrl());
    };
    window.addEventListener('popstate', onPopState);
    return () => window.removeEventListener('popstate', onPopState);
  }, []);
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    const saved = localStorage.getItem('sekonic-theme');
    return saved === 'dark' ? 'dark' : 'light';
  });
  const [sidebarWidth, setSidebarWidth] = useState<number>(() => {
    const saved = Number(localStorage.getItem('sekonic-sidebar-w'));
    return saved >= 160 && saved <= 360 ? saved : 220;
  });
  const [scanning, setScanning] = useState(false);
  const [simulatorMode, setSimulatorMode] = useState(false);
  const [dropoutLed, setDropoutLed] = useState('Red');
  const [ports, setPorts] = useState<PortOption[]>([]);
  const [selectedPort, setSelectedPort] = useState('');
  const [baudRate, setBaudRate] = useState(38400);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [rows, setRows] = useState<ScannedRow[]>([]);
  const [selectedRow, setSelectedRow] = useState<ScannedRow | null>(null);
  const [showRightPanel, setShowRightPanel] = useState(false);
  const [reviewRow, setReviewRow] = useState<ScannedRow | null>(null);
  const [policy, setPolicy] = useState<ThresholdPolicy>({
    minMarkDensity: 48,
    strongMarkDensity: 115,
    erasureRatio: 0.42,
    confidenceFloor: 0.65,
  });
  const [hwSettings, setHwSettings] = useState<HardwareSettings>({
    scannerModel: 'sekonic',
    separationEnabled: false,
    separationColumn: 28,
    duplex: false,
    subStackerEnabled: true,
    stopOnError: true,
    manualFeedTimeoutSec: 2,
    densityEnabled: true,
    densityUpperLimit: 22,
    barcodeEnabled: true,
    language: 'tr',
    printerEnabled: false,
    printerPrintSeq: false,
    printerPrintOkundu: false,
    printerPrintDateTime: false,
    printerPrintOkunmadi: false,
    printerPrintFileName: false,
    printerPosMm: 2283,
    printerText: 'OKUNDU',
    assessmentApiUrl: 'wss://olcme.kurum.edu.tr/v1/omr/stream',
  });

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('sekonic-theme', theme);
  }, [theme]);

  const refreshPorts = useCallback(async () => {
    const invoke = getTauriInvoke();
    if (!invoke) return;
    try {
      const list = await invoke<PortOption[]>('list_available_ports');
      setPorts(list);
      if (list.length > 0 && !selectedPort) {
        setSelectedPort(list[0].id);
      }
    } catch (e) {
      setErrorMsg(String(e));
    }
  }, [selectedPort]);

  useEffect(() => {
    void refreshPorts();
  }, [refreshPorts]);

  useEffect(() => {
    const w = window as unknown as {
      __TAURI__?: { event?: { listen: (ev: string, cb: (e: { payload: unknown }) => void) => Promise<() => void> } };
    };
    let unlistenScanned: (() => void) | undefined;
    let unlistenErr: (() => void) | undefined;

    if (w.__TAURI__?.event) {
      w.__TAURI__.event.listen('sheet:scanned', (e) => {
        try {
          const parsed = JSON.parse(String(e.payload)) as {
            seq_no: number;
            decoded_fields?: Record<string, string>;
            answers?: Record<string, string>;
            missing_required_field?: boolean;
            needs_operator_review?: boolean;
            warnings?: string[];
            raw_matrix_b64?: string;
          };
          const row: ScannedRow = {
            id: parsed.seq_no,
            seqNo: parsed.seq_no,
            studentId: parsed.decoded_fields?.student_id || '????????',
            studentName: parsed.missing_required_field ? 'Tanımsız' : 'Okundu',
            booklet: parsed.decoded_fields?.kitapcik || 'A',
            stacker: parsed.missing_required_field ? 'sub' : 'main',
            confidence: parsed.needs_operator_review ? 72 : 98,
            status: parsed.missing_required_field
              ? 'Kayıtsız'
              : (parsed.warnings?.length ?? 0) > 0
                ? 'Silgi Kurtarıldı'
                : 'Tamam',
            answersSummary: Object.values(parsed.answers ?? {}).join(' '),
            rawMatrixB64: parsed.raw_matrix_b64 ?? '',
          };
          setRows((prev) => [row, ...prev]);
          setSelectedRow((cur) => cur ?? row);
        } catch {
          // ignore
        }
      }).then((fn) => {
        unlistenScanned = fn;
      });

      w.__TAURI__.event.listen('scanner:error', (e) => {
        setErrorMsg(String(e.payload));
        setScanning(false);
      }).then((fn) => {
        unlistenErr = fn;
      });
    }

    return () => {
      unlistenScanned?.();
      unlistenErr?.();
    };
  }, []);

  useEffect(() => {
    if (!scanning || getTauriInvoke() || !simulatorMode) return;
    const timer = setInterval(() => {
      setRows((prev) => {
        const nextSeq = prev.length + 1;
        return [createSimulatedRow(nextSeq), ...prev];
      });
    }, 550);
    return () => clearInterval(timer);
  }, [scanning, simulatorMode]);

  const handleStart = async () => {
    setErrorMsg(null);
    if (activeTab !== 'scan') setActiveTab('scan');
    const invoke = getTauriInvoke();
    if (invoke) {
      try {
        await invoke('start_scan', {
          portId: selectedPort,
          baudRate,
          simulatorMode,
        });
        setScanning(true);
      } catch (e) {
        setErrorMsg(String(e));
      }
    } else if (!simulatorMode) {
      const nav = navigator as unknown as { serial?: { requestPort: () => Promise<unknown> } };
      if (nav.serial) {
        try {
          await nav.serial.requestPort();
          setScanning(true);
        } catch (e) {
          setErrorMsg(`WebSerial seçilmedi: ${String(e)}`);
        }
      } else {
        setErrorMsg('Gerçek COM port için Ayarlar -> Bağlantı veya Tauri masaüstü penceresini kullanın.');
      }
    } else {
      setScanning(true);
    }
  };

  const handleStop = async () => {
    const invoke = getTauriInvoke();
    if (invoke) {
      await invoke('stop_scan');
    }
    setScanning(false);
  };

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'F2') {
        e.preventDefault();
        const flagged = rows.find((r) => r.stacker === 'sub' || r.status !== 'Tamam') ?? selectedRow;
        if (flagged) setReviewRow(flagged);
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [rows, selectedRow]);

  const kpis = useMemo(() => {
    const main = rows.filter((r) => r.stacker === 'main').length;
    const sub = rows.filter((r) => r.stacker === 'sub').length;
    const erasure = rows.filter((r) => r.status === 'Silgi Kurtarıldı').length;
    return { total: rows.length, main, sub, erasure };
  }, [rows]);

  const startSidebarResize = (e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startW = sidebarWidth;
    const onMove = (ev: MouseEvent) => {
      const next = Math.max(160, Math.min(360, startW + (ev.clientX - startX)));
      setSidebarWidth(next);
      localStorage.setItem('sekonic-sidebar-w', String(next));
    };
    const onUp = () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  };

  return (
    <div style={{ minHeight: '100vh', padding: 14, display: 'flex', gap: 4 }}>
      <SidebarNav
        activeTab={activeTab}
        onSelectTab={setActiveTab}
        theme={theme}
        onToggleTheme={() => setTheme((t) => (t === 'dark' ? 'light' : 'dark'))}
        width={sidebarWidth}
      />
      <div
        onMouseDown={startSidebarResize}
        title="Kenar çubuğunu yeniden boyutlandır"
        style={{
          width: 8,
          cursor: 'col-resize',
          flexShrink: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
        }}
      >
        <div style={{ width: 2, height: 36, borderRadius: 2, background: 'var(--uxigl-border)' }} />
      </div>

      <div style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <TopControlBar
          activeTab={activeTab}
          scanning={scanning}
          simulatorMode={simulatorMode}
          selectedPort={selectedPort}
          onStart={() => void handleStart()}
          onStop={() => void handleStop()}
        />

        {activeTab === 'scan' && (
          <div style={{ display: 'flex', gap: 12, flex: 1, minWidth: 0 }}>
            <div style={{ width: 270, flexShrink: 0, display: 'flex', flexDirection: 'column', gap: 12 }}>
              <DeviceTelemetryCard
                scanning={scanning}
                simulatorMode={simulatorMode}
                onToggleSimulator={setSimulatorMode}
                dropoutLed={dropoutLed}
                onDropoutChange={setDropoutLed}
                ports={ports}
                selectedPort={selectedPort}
                onSelectPort={setSelectedPort}
                baudRate={baudRate}
                onBaudRateChange={setBaudRate}
                onRefreshPorts={() => void refreshPorts()}
                errorMsg={errorMsg}
              />
              <SheetCanvasPreview
                row={selectedRow}
                theme={theme}
                onOpenReview={() => {
                  if (selectedRow) setReviewRow(selectedRow);
                }}
              />
            </div>

            <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 12, minWidth: 0 }}>
              <KpiStrip
                total={kpis.total}
                mainCount={kpis.main}
                subCount={kpis.sub}
                erasureCount={kpis.erasure}
              />
              <LiveScanGrid
                rows={rows}
                theme={theme}
                onSelectRow={(row) => {
                  setSelectedRow(row);
                  setShowRightPanel(true);
                }}
                onCellEdit={(row, colId, val) => {
                  setRows((prev) =>
                    prev.map((r) => (r.id === row.id ? { ...r, [colId]: val } : r)),
                  );
                }}
              />
              <ExceptionAlertBar
                subStackerCount={kpis.sub}
                onReviewClick={() => {
                  const nextSub = rows.find((r) => r.stacker === 'sub') ?? selectedRow;
                  if (nextSub) setReviewRow(nextSub);
                }}
              />
            </div>

            {showRightPanel && (
              <AnswerDetailsPanel
                row={selectedRow}
                onClose={() => setShowRightPanel(false)}
              />
            )}
          </div>
        )}

        {activeTab === 'history' && <ScanHistoryView theme={theme} />}

        {activeTab === 'forms' && (
          <FormDesignerView theme={theme} policy={policy} onPolicyChange={setPolicy} />
        )}

        {activeTab === 'settings' && (
          <SettingsView
            simulatorMode={simulatorMode}
            onToggleSimulator={setSimulatorMode}
            ports={ports}
            selectedPort={selectedPort}
            onSelectPort={setSelectedPort}
            baudRate={baudRate}
            onBaudRateChange={setBaudRate}
            onRefreshPorts={() => void refreshPorts()}
            dropoutLed={dropoutLed}
            onDropoutChange={setDropoutLed}
            policy={policy}
            onPolicyChange={setPolicy}
            hw={hwSettings}
            onHwChange={setHwSettings}
          />
        )}
      </div>

      <HeatmapReviewModal
        row={reviewRow}
        theme={theme}
        onClose={() => setReviewRow(null)}
        onResolveAnswer={(rowId) => {
          setRows((prev) =>
            prev.map((r) =>
              r.id === rowId ? { ...r, status: 'Tamam', stacker: 'main', confidence: 100 } : r,
            ),
          );
          setReviewRow(null);
        }}
        onRouteSubStacker={(rowId) => {
          setRows((prev) =>
            prev.map((r) => (r.id === rowId ? { ...r, stacker: 'sub' } : r)),
          );
          setReviewRow(null);
        }}
      />
    </div>
  );
};
