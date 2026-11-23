import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { SidebarNav, NavTab } from './components/layout/SidebarNav';
import { TopControlBar } from './components/layout/TopControlBar';
import {
  DEFAULT_THEME_PREFS,
  FONT_STACKS,
  SEED_PALETTES,
  UiThemePrefs,
  UxiglThemeStudio,
} from './components/layout/UxiglThemeStudio';
import { exportScannedRows } from './utils/exportScans';
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
import {
  AgentInfo,
  AgentUpdateInfo,
  isNewerSemver,
  LocalAgentClient,
  RawGradedSheet,
  ScanPhase,
  ScannerDiagnosticInfo,
} from './services/localAgentClient';

declare const __APP_VERSION__: string;
const APP_VERSION = typeof __APP_VERSION__ !== 'undefined' ? __APP_VERSION__ : '0.2.8';
import { ScannedRow, ThresholdPolicy } from './types/scanner';
import {
  FormTemplate,
  loadActiveTemplateId,
  loadTemplates,
  parseFmtOrBldm,
  saveActiveTemplateId,
  saveTemplates,
  templateToOmrSchema,
} from './utils/formTemplates';

function findDecodedField(
  decoded: Record<string, string>,
  exactKeys: string[],
  partialTerms: string[],
): string {
  for (const k of exactKeys) {
    if (decoded[k] !== undefined) return decoded[k].trim();
  }
  const entry = Object.entries(decoded).find(([k]) => {
    const lower = k.toLocaleLowerCase('tr-TR');
    return partialTerms.some((term) => lower.includes(term));
  });
  return entry ? entry[1].trim() : '';
}

function mapRawSheetToRow(parsed: RawGradedSheet, template?: FormTemplate | null): ScannedRow {
  const decoded = parsed.decoded_fields ?? {};
  const answersMap = parsed.answers ?? {};

  const studentId = findDecodedField(
    decoded,
    ['student_id', 'ogrenci_no', 'öğrenci_no'],
    ['ogrenci', 'öğrenci', 'student'],
  );
  const studentName = findDecodedField(
    decoded,
    ['adi_soyadi', 'adı_soyadı', 'ad_soyad', 'student_name'],
    ['adi', 'adı', 'soyad', 'name'],
  );
  const booklet = findDecodedField(
    decoded,
    ['kitapcik', 'kitapçık', 'booklet'],
    ['kitap', 'booklet'],
  );

  const answersSummary = template
    ? template.fields
        .filter((f) => f.fieldType === 'subject_questions')
        .map((f) => answersMap[f.key] ?? '')
        .join('')
    : Object.values(answersMap).join('');

  return {
    id: parsed.seq_no,
    seqNo: parsed.seq_no,
    studentId,
    studentName,
    booklet,
    stacker: parsed.missing_required_field ? 'sub' : 'main',
    confidence: parsed.needs_operator_review ? 72 : 98,
    status: parsed.missing_required_field
      ? 'Kayıtsız'
      : (parsed.warnings?.length ?? 0) > 0
        ? 'Silgi Kurtarıldı'
        : 'Tamam',
    answersSummary,
    decodedFields: decoded,
    answers: answersMap,
    rawMatrixB64: parsed.raw_matrix_b64 ?? '',
    rawWireSample: parsed.raw_wire_sample,
  };
}

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
  const [themePrefs, setThemePrefs] = useState<UiThemePrefs>(() => {
    try {
      const raw = localStorage.getItem('sekonic-ui-prefs');
      if (raw) {
        const parsed = JSON.parse(raw) as Partial<UiThemePrefs>;
        return {
          mode: parsed.mode === 'dark' || parsed.mode === 'system' ? parsed.mode : 'light',
          fontSize: typeof parsed.fontSize === 'number' && parsed.fontSize >= 11 && parsed.fontSize <= 18 ? parsed.fontSize : 13,
          fontFamily: parsed.fontFamily === 'readable' || parsed.fontFamily === 'mono' ? parsed.fontFamily : 'system',
          palette: parsed.palette && parsed.palette in SEED_PALETTES ? parsed.palette : 'graphite',
          radius: typeof parsed.radius === 'number' && parsed.radius >= 4 && parsed.radius <= 20 ? parsed.radius : 12,
          sidebarOpen: typeof parsed.sidebarOpen === 'boolean' ? parsed.sidebarOpen : true,
        };
      }
    } catch {
      // ignore
    }
    const legacy = localStorage.getItem('sekonic-theme');
    return { ...DEFAULT_THEME_PREFS, mode: legacy === 'dark' ? 'dark' : 'light' };
  });
  const [themeStudioOpen, setThemeStudioOpen] = useState(false);
  const [systemDark, setSystemDark] = useState<boolean>(() =>
    typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches,
  );
  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = (e: MediaQueryListEvent) => setSystemDark(e.matches);
    mq.addEventListener('change', onChange);
    return () => mq.removeEventListener('change', onChange);
  }, []);
  const theme: 'light' | 'dark' =
    themePrefs.mode === 'system' ? (systemDark ? 'dark' : 'light') : themePrefs.mode;
  const [sidebarWidth, setSidebarWidth] = useState<number>(() => {
    const saved = Number(localStorage.getItem('sekonic-sidebar-w'));
    return saved >= 160 && saved <= 360 ? saved : 220;
  });
  const [cockpitLeftWidth, setCockpitLeftWidth] = useState<number>(() => {
    const saved = Number(localStorage.getItem('sekonic-cockpit-left-w'));
    return saved >= 240 && saved <= 520 ? saved : 300;
  });
  const [scanning, setScanning] = useState(false);
  const [scanPhase, setScanPhase] = useState<ScanPhase>('idle');
  const [simulatorMode, setSimulatorMode] = useState(false);
  const [dropoutLed, setDropoutLed] = useState('Red');
  const [ports, setPorts] = useState<PortOption[]>([]);
  const [selectedPort, setSelectedPort] = useState('');
  const [baudRate, setBaudRate] = useState(38400);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const [scannerDiag, setScannerDiag] = useState<ScannerDiagnosticInfo>({});
  const [templates, setTemplates] = useState<FormTemplate[]>(() => loadTemplates());
  const [activeTemplateId, setActiveTemplateId] = useState<string>(() =>
    loadActiveTemplateId(loadTemplates()),
  );
  const [agentInfo, setAgentInfo] = useState<AgentInfo>(() => ({
    state: getTauriInvoke() ? 'tauri' : 'connecting',
  }));
  const [agentUpdate, setAgentUpdate] = useState<AgentUpdateInfo>({ available: false });
  const agentRef = useRef<LocalAgentClient | null>(null);

  useEffect(() => {
    saveTemplates(templates);
  }, [templates]);

  useEffect(() => {
    saveActiveTemplateId(activeTemplateId);
  }, [activeTemplateId]);

  const activeTemplate = useMemo(
    () => templates.find((t) => t.id === activeTemplateId) ?? templates[0] ?? null,
    [templates, activeTemplateId],
  );
  const activeTemplateRef = useRef<FormTemplate | null>(activeTemplate);
  useEffect(() => {
    activeTemplateRef.current = activeTemplate;
  }, [activeTemplate]);

  const handleImportFmtFile = useCallback((file: File) => {
    const reader = new FileReader();
    reader.onload = () => {
      const text = typeof reader.result === 'string' ? reader.result : '';
      const parsed = parseFmtOrBldm(text);
      if (!parsed || parsed.fields.length === 0) {
        setErrorMsg(`'${file.name}' geçerli bir .fmt / .bldm şablonu olarak okunamadı.`);
        return;
      }
      const id = `tpl_${Date.now()}`;
      const name = file.name.replace(/\.(fmt|bldm|txt)$/i, '');
      const nextTpl: FormTemplate = {
        id,
        name,
        rows: parsed.rows,
        cols: parsed.cols,
        fields: parsed.fields,
      };
      setTemplates((prev) => [nextTpl, ...prev]);
      setActiveTemplateId(id);
      setErrorMsg(null);
    };
    reader.readAsText(file, 'windows-1254');
  }, []);
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
    barcodeEnabled: false,
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
    const root = document.documentElement;
    const pal = SEED_PALETTES[themePrefs.palette] ?? SEED_PALETTES.graphite;
    root.setAttribute('data-theme', theme);
    root.style.setProperty('--uxigl-font-size', `${themePrefs.fontSize}px`);
    root.style.setProperty('--uxu-font-scale', (themePrefs.fontSize / 13).toFixed(3));
    root.style.setProperty('--uxigl-font-family', FONT_STACKS[themePrefs.fontFamily]);
    root.style.setProperty('--uxigl-radius', `${themePrefs.radius}px`);
    root.style.setProperty('--uxu-accent', pal.accent);
    root.style.setProperty('--uxigl-accent-soft', pal.accentSoft);
    root.style.setProperty('--uxigl-primary-bg', theme === 'dark' ? pal.primaryDark : pal.primaryLight);
    root.style.setProperty('--uxu-sidebar-width', `${sidebarWidth}px`);
    root.style.fontSize = `${themePrefs.fontSize}px`;
    try {
      localStorage.setItem('sekonic-ui-prefs', JSON.stringify(themePrefs));
      localStorage.setItem('sekonic-theme', theme);
    } catch {
      // ignore
    }
  }, [themePrefs, theme, sidebarWidth]);

  const refreshPorts = useCallback(async () => {
    const invoke = getTauriInvoke();
    if (invoke) {
      try {
        const list = await invoke<PortOption[]>('list_available_ports');
        setPorts(list);
        if (list.length > 0 && !selectedPort) {
          setSelectedPort(list[0].id);
        }
      } catch (e) {
        setErrorMsg(String(e));
      }
      return;
    }

    const agent = agentRef.current;
    if (agent && agent.isConnected()) {
      try {
        const list = await agent.listPorts();
        setPorts(list);
        setSelectedPort((cur) => (cur && list.some((p) => p.id === cur) ? cur : list[0]?.id ?? ''));
        setErrorMsg(null);
      } catch (e) {
        setErrorMsg(String(e));
      }
    } else if (agent) {
      agent.reconnectNow();
    }
  }, [selectedPort]);

  const handleCheckAgentUpdate = useCallback(async () => {
    const agent = agentRef.current;
    if (!agent || !agent.isConnected()) return;
    setAgentUpdate((prev) => ({
      ...prev,
      checking: true,
      checkedUpToDate: false,
      error: undefined,
    }));
    const markUpToDate = (current?: string, latest?: string) => {
      setAgentUpdate({
        available: false,
        currentVersion: current,
        latestVersion: latest,
        checking: false,
        checkedUpToDate: true,
      });
      window.setTimeout(() => {
        setAgentUpdate((prev) => (prev.checkedUpToDate ? { ...prev, checkedUpToDate: false } : prev));
      }, 4000);
    };

    try {
      const res = await fetch(`/downloads/agent-version.json?t=${Date.now()}`);
      if (res.ok) {
        const manifest = (await res.json()) as { version?: string; download_url?: string };
        const latest = manifest.version ?? APP_VERSION;
        const current = agent.getInfo().version ?? '0.0.0';
        if (isNewerSemver(latest, current)) {
          setAgentUpdate({
            available: true,
            currentVersion: current,
            latestVersion: latest,
            downloadUrl: manifest.download_url ?? `${window.location.origin}/downloads/sekonic-agent.exe`,
            updating: false,
            checking: false,
            checkedUpToDate: false,
          });
          return;
        }
        markUpToDate(current, latest);
        return;
      }
    } catch {
      // fallback to agent check
    }
    try {
      const info = await agent.checkUpdate();
      if (info.available) {
        setAgentUpdate({ ...info, updating: false, checking: false, checkedUpToDate: false });
      } else {
        markUpToDate(info.currentVersion, info.latestVersion);
      }
    } catch {
      markUpToDate(agent.getInfo().version, APP_VERSION);
    }
  }, []);

  const handleUpdateAgent = useCallback(async () => {
    const agent = agentRef.current;
    if (!agent || !agent.isConnected()) return;
    setAgentUpdate((prev) => ({ ...prev, updating: true, error: undefined }));
    try {
      await agent.updateAgent(agentUpdate.downloadUrl);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setAgentUpdate((prev) => ({
        ...prev,
        updating: false,
        error: msg.includes('Geçersiz komut')
          ? 'Eski ajan sürümü otomatik güncellemeyi desteklemiyor; lütfen yeni .exe dosyasını indirip çalıştırın.'
          : msg,
      }));
    }
  }, [agentUpdate.downloadUrl]);

  useEffect(() => {
    if (getTauriInvoke()) {
      void refreshPorts();
      return;
    }

    const client = new LocalAgentClient({
      onStatusChange: (info) => {
        setAgentInfo(info);
        if (info.state === 'connected') {
          if (info.version && isNewerSemver(APP_VERSION, info.version)) {
            setAgentUpdate({
              available: true,
              currentVersion: info.version,
              latestVersion: APP_VERSION,
              downloadUrl: `${window.location.origin}/downloads/sekonic-agent.exe`,
              updating: false,
            });
          } else {
            setAgentUpdate({ available: false, updating: false });
          }
          void client.listPorts().then((list) => {
            setPorts(list);
            setSelectedPort((cur) => (cur && list.some((p) => p.id === cur) ? cur : list[0]?.id ?? ''));
            setErrorMsg(null);
          }).catch(() => {});
        } else if (info.state === 'disconnected') {
          setPorts([]);
        }
      },
      onAgentUpdate: (upd) => {
        setAgentUpdate((prev) => ({ ...prev, ...upd }));
      },
      onSheetScanned: (parsed) => {
        const row = mapRawSheetToRow(parsed, activeTemplateRef.current);
        setRows((prev) => [row, ...prev]);
        setSelectedRow((cur) => cur ?? row);
      },
      onScannerError: (msg) => {
        setErrorMsg(msg);
        setScanning(false);
        setScanPhase('idle');
      },
      onScannerStatus: (isScanning, phase) => {
        setScanning(isScanning);
        if (phase) {
          setScanPhase(phase);
        } else {
          setScanPhase(isScanning ? 'scanning' : 'idle');
        }
      },
      onScannerDiagnostic: (diag) => {
        setScannerDiag(diag);
      },
      onHopperEmpty: () => {
        setScanning(false);
        setScanPhase('idle');
      },
    });
    agentRef.current = client;
    client.connect();

    return () => {
      client.dispose();
      agentRef.current = null;
    };
  }, []);

  useEffect(() => {
    const w = window as unknown as {
      __TAURI__?: { event?: { listen: (ev: string, cb: (e: { payload: unknown }) => void) => Promise<() => void> } };
    };
    let unlistenScanned: (() => void) | undefined;
    let unlistenErr: (() => void) | undefined;

    if (w.__TAURI__?.event) {
      w.__TAURI__.event.listen('sheet:scanned', (e) => {
        try {
          const parsed = JSON.parse(String(e.payload)) as RawGradedSheet;
          const row = mapRawSheetToRow(parsed, activeTemplateRef.current);
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
    const hasNativeBackend = Boolean(getTauriInvoke()) || Boolean(agentRef.current?.isConnected());
    if (!scanning || hasNativeBackend || !simulatorMode) return;
    const timer = setInterval(() => {
      setRows((prev) => {
        const nextSeq = prev.length + 1;
        return [createSimulatedRow(nextSeq), ...prev];
      });
    }, 550);
    return () => clearInterval(timer);
  }, [scanning, simulatorMode]);

  const handleStart = async () => {
    if (scanPhase === 'initializing' || scanning) return;
    setErrorMsg(null);
    if (activeTab !== 'scan') setActiveTab('scan');
    const invoke = getTauriInvoke();
    if (invoke) {
      setScanPhase('initializing');
      try {
        await invoke('start_scan', {
          portId: selectedPort,
          baudRate,
          simulatorMode,
        });
        setScanning(true);
        setScanPhase('scanning');
      } catch (e) {
        setErrorMsg(String(e));
        setScanPhase('idle');
      }
      return;
    }

    const agent = agentRef.current;
    if (agent && agent.isConnected()) {
      if (!activeTemplate) {
        setErrorMsg('Lütfen önce bir optik form şablonu seçin veya .fmt dosyası yükleyin.');
        return;
      }
      setScanPhase('initializing');
      try {
        const schema = templateToOmrSchema(activeTemplate, policy, hwSettings.duplex, dropoutLed);
        await agent.startScan({
          portId: selectedPort,
          baudRate,
          simulatorMode,
          enableBarcode: hwSettings.barcodeEnabled,
          schema,
        });
      } catch (e) {
        setErrorMsg(e instanceof Error ? e.message : String(e));
        setScanning(false);
        setScanPhase('idle');
      }
      return;
    }

    if (simulatorMode) {
      setScanning(true);
      setScanPhase('scanning');
    } else {
      setErrorMsg(
        'Gerçek COM/USB tarayıcıya bağlanmak için Sekonic Yerel Ajanını (sekonic-agent.exe) indirip çalıştırın.',
      );
    }
  };

  const handleStop = async () => {
    const invoke = getTauriInvoke();
    if (invoke) {
      await invoke('stop_scan');
    } else if (agentRef.current?.isConnected()) {
      await agentRef.current.stopScan().catch(() => {});
    }
    setScanning(false);
    setScanPhase('idle');
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

  const startCockpitLeftResize = (e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startW = cockpitLeftWidth;
    const onMove = (ev: MouseEvent) => {
      const next = Math.max(240, Math.min(520, startW + (ev.clientX - startX)));
      setCockpitLeftWidth(next);
      localStorage.setItem('sekonic-cockpit-left-w', String(next));
    };
    const onUp = () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  };

  const handleCopyDiagnostic = useCallback(async () => {
    const agentSnap = await agentRef.current?.getDiagnostics();
    const ordered = [...rows].sort((a, b) => a.seqNo - b.seqNo);
    const snapSheets = Array.isArray(agentSnap?.recent_sheets)
      ? (agentSnap.recent_sheets as RawGradedSheet[]).map((s) => mapRawSheetToRow(s, activeTemplate))
      : [];
    const sourceRows = ordered.length > 0 ? ordered : snapSheets;

    const sampleSheets = sourceRows.slice(0, 2).map((r) => {
      const wire = r.rawWireSample ?? '';
      const body = wire.startsWith('MD') && wire.length >= 6 ? wire.slice(6) : wire;
      const tokens = body.includes(',') ? body.split(',').filter(Boolean) : [];
      return {
        seqNo: r.seqNo,
        studentId: r.studentId,
        studentName: r.studentName,
        booklet: r.booklet,
        decodedFields: r.decodedFields,
        answers: r.answers,
        wireStats: {
          totalLen: wire.length,
          header: wire.slice(0, 12),
          tokenCount: tokens.length,
          firstTokenLen: tokens[0]?.length ?? 0,
          uniqueChars: Array.from(new Set(body.replace(/,/g, ''))).sort().join(''),
        },
        rawWireSample: wire,
      };
    });
    const report = {
      capturedAt: new Date().toISOString(),
      agent: agentInfo,
      selectedPort,
      baudRate,
      simulatorMode,
      lastError: errorMsg,
      scanner: Object.keys(scannerDiag).length > 0 ? scannerDiag : agentSnap
        ? {
            modelName: agentSnap.model_name,
            rm16Level: agentSnap.rm_16level,
            initLog: agentSnap.init_log,
            lastError: agentSnap.last_error,
            totalScannedInAgent: agentSnap.total_scanned,
          }
        : {},
      policy,
      template: activeTemplate
        ? {
            name: activeTemplate.name,
            rows: activeTemplate.rows,
            cols: activeTemplate.cols,
            fields: activeTemplate.fields.map((f) => ({
              key: f.key,
              label: f.label,
              type: f.fieldType,
              dir: f.direction,
              r: [f.startRow, f.startRow + f.rows - 1],
              c: [f.startCol, f.startCol + f.cols - 1],
              alphaLen: f.alphabet.length,
            })),
          }
        : null,
      totalRows: sourceRows.length,
      sampleSheets,
    };
    const text = JSON.stringify(report, null, 2);
    if (navigator.clipboard?.writeText) {
      await navigator.clipboard.writeText(text);
    } else {
      const ta = document.createElement('textarea');
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      ta.remove();
    }
  }, [rows, agentInfo, selectedPort, baudRate, simulatorMode, errorMsg, scannerDiag, policy, activeTemplate]);

  return (
    <>
    <div className="uxigl-app-scale" style={{ minHeight: '100vh', padding: 14, display: 'flex', gap: 4 }}>
      {themePrefs.sidebarOpen && (
        <>
          <SidebarNav
            activeTab={activeTab}
            onSelectTab={setActiveTab}
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
        </>
      )}

      <div style={{ flex: 1, minWidth: 0, display: 'flex', flexDirection: 'column', gap: 12 }}>
        <TopControlBar
          activeTab={activeTab}
          scanning={scanning}
          scanPhase={scanPhase}
          simulatorMode={simulatorMode}
          selectedPort={selectedPort}
          activeTemplateName={activeTemplate?.name ?? ''}
          rowCount={rows.length}
          agentInfo={agentInfo}
          agentUpdate={agentUpdate}
          onUpdateAgent={() => void handleUpdateAgent()}
          themeStudioOpen={themeStudioOpen}
          onToggleThemeStudio={() => setThemeStudioOpen((o) => !o)}
          onCopyDiagnostic={() => void handleCopyDiagnostic()}
          onStart={() => void handleStart()}
          onStop={() => void handleStop()}
          onExport={(format) => exportScannedRows(rows, format, activeTemplate ?? undefined)}
        />

        {activeTab === 'scan' && (
          <div style={{ display: 'flex', gap: 4, flex: 1, minWidth: 0 }}>
            <div style={{ width: cockpitLeftWidth, flexShrink: 0, display: 'flex', flexDirection: 'column', gap: 12 }}>
              <DeviceTelemetryCard
                scanning={scanning}
                scanPhase={scanPhase}
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
                agentInfo={agentInfo}
                agentUpdate={agentUpdate}
                onUpdateAgent={() => void handleUpdateAgent()}
                onCheckAgentUpdate={() => void handleCheckAgentUpdate()}
                onReconnectAgent={() => agentRef.current?.reconnectNow()}
                templates={templates}
                activeTemplateId={activeTemplateId}
                onSelectTemplate={setActiveTemplateId}
                onImportFmtFile={handleImportFmtFile}
                onOpenDesigner={() => setActiveTab('forms')}
              />
              <SheetCanvasPreview
                row={selectedRow}
                theme={theme}
                onOpenReview={() => {
                  if (selectedRow) setReviewRow(selectedRow);
                }}
              />
            </div>

            <div
              onMouseDown={startCockpitLeftResize}
              title="Cihaz & Form panelini yeniden boyutlandır"
              style={{
                width: 8,
                cursor: 'col-resize',
                flexShrink: 0,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }}
            >
              <div style={{ width: 2, height: 42, borderRadius: 2, background: 'var(--uxigl-border)' }} />
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
          <FormDesignerView
            theme={theme}
            policy={policy}
            onPolicyChange={setPolicy}
            templates={templates}
            activeTemplateId={activeTemplateId}
            onTemplatesChange={setTemplates}
            onSelectTemplate={setActiveTemplateId}
          />
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

    <UxiglThemeStudio
      open={themeStudioOpen}
      onClose={() => setThemeStudioOpen(false)}
      prefs={themePrefs}
      resolvedMode={theme}
      sidebarWidth={sidebarWidth}
      onChangeSidebarWidth={(w) => {
        setSidebarWidth(w);
        localStorage.setItem('sekonic-sidebar-w', String(w));
      }}
      onChangePrefs={setThemePrefs}
      onReset={() => {
        setThemePrefs(DEFAULT_THEME_PREFS);
        setSidebarWidth(220);
        localStorage.setItem('sekonic-sidebar-w', '220');
      }}
    />
    </>
  );
};
