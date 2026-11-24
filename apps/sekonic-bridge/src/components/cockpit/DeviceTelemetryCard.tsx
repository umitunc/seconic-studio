import React, { useEffect, useMemo, useRef, useState } from 'react';
import { Cpu, Download, FileText, Gauge, RefreshCw, ShieldCheck, AlertTriangle, Upload, Search, ChevronDown, Check, X, Loader2, HelpCircle } from 'lucide-react';
import { AgentInfo, AgentUpdateInfo, ScanPhase } from '../../services/localAgentClient';
import { FormTemplate } from '../../utils/formTemplates';

export interface PortOption {
  id: string;
  label: string;
  kind: 'serial' | 'usb';
}

interface DeviceTelemetryCardProps {
  scanning: boolean;
  scanPhase: ScanPhase;
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
  agentUpdate: AgentUpdateInfo;
  onUpdateAgent: () => void;
  onCheckAgentUpdate: () => void;
  onReconnectAgent: () => void;
  templates: FormTemplate[];
  activeTemplateId: string;
  onSelectTemplate: (id: string) => void;
  onChangeTemplateOrientation?: (orientation: 'D' | 'T') => void;
  onImportFmtFile: (file: File) => void;
  onOpenDesigner: () => void;
}

export const DeviceTelemetryCard: React.FC<DeviceTelemetryCardProps> = ({
  scanning,
  scanPhase,
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
  agentUpdate,
  onUpdateAgent,
  onCheckAgentUpdate,
  onReconnectAgent,
  templates,
  activeTemplateId,
  onSelectTemplate,
  onChangeTemplateOrientation,
  onImportFmtFile,
  onOpenDesigner,
}) => {
  const fmtInputRef = useRef<HTMLInputElement | null>(null);
  const comboRef = useRef<HTMLDivElement | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const isBrowserWithoutAgent = agentInfo.state === 'disconnected' || agentInfo.state === 'connecting';
  const activeTemplate = templates.find((t) => t.id === activeTemplateId) ?? templates[0];

  const [tplOpen, setTplOpen] = useState(false);
  const [tplQuery, setTplQuery] = useState('');
  const [highlightIdx, setHighlightIdx] = useState(0);
  const [showOrientationHelp, setShowOrientationHelp] = useState(true);

  const filteredTemplates = useMemo(() => {
    const q = tplQuery.trim().toLocaleLowerCase('tr-TR');
    if (!q) return templates;
    return templates.filter((t) => {
      const hay = `${t.name} ${t.rows}x${t.cols} ${t.rows}×${t.cols}`.toLocaleLowerCase('tr-TR');
      return hay.includes(q);
    });
  }, [templates, tplQuery]);

  useEffect(() => {
    setHighlightIdx(0);
  }, [tplQuery, tplOpen]);

  useEffect(() => {
    const onOutside = (e: MouseEvent) => {
      if (comboRef.current && !comboRef.current.contains(e.target as Node)) {
        setTplOpen(false);
        setTplQuery('');
      }
    };
    document.addEventListener('mousedown', onOutside);
    return () => document.removeEventListener('mousedown', onOutside);
  }, []);

  const commitTemplate = (id: string) => {
    onSelectTemplate(id);
    setTplOpen(false);
    setTplQuery('');
    inputRef.current?.blur();
  };

  return (
    <div className="uxigl-glass-card" style={{ padding: 14, display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontWeight: 600, fontSize: 13, display: 'flex', alignItems: 'center', gap: 6 }}>
          <Cpu size={15} color="#38bdf8" /> Cihaz & Form Bağlantısı
        </span>
        <span
          style={{
            fontSize: 12,
            display: 'inline-flex',
            alignItems: 'center',
            gap: 5,
            color:
              scanPhase === 'initializing'
                ? '#f59e0b'
                : scanning
                  ? '#34d399'
                  : '#94a3b8',
            fontWeight: scanPhase === 'initializing' ? 600 : 400,
          }}
        >
          {scanPhase === 'initializing' ? (
            <>
              <Loader2 size={12} className="uxigl-spin" /> Hazırlanıyor...
            </>
          ) : scanning ? (
            '● Aktif'
          ) : (
            '○ Beklemede'
          )}
        </span>
      </div>

      {/* Optik Form Şablonu Seçimi */}
      <div className="uxigl-subcard" style={{ display: 'flex', flexDirection: 'column', gap: 6, padding: 10 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 11, color: 'var(--uxigl-text-muted)' }}>
          <span style={{ display: 'flex', alignItems: 'center', gap: 4, fontWeight: 600 }}>
            <FileText size={12} color="#38bdf8" /> Optik Form Şablonu
          </span>
          <div style={{ display: 'flex', gap: 4 }}>
            <input
              ref={fmtInputRef}
              type="file"
              accept=".fmt,.bldm,.txt"
              style={{ display: 'none' }}
              onChange={(e) => {
                const f = e.target.files?.[0];
                if (f) onImportFmtFile(f);
                e.target.value = '';
              }}
            />
            <button
              type="button"
              className="uxigl-btn"
              style={{ padding: '2px 6px', fontSize: 11 }}
              onClick={() => fmtInputRef.current?.click()}
              title="Yeni .fmt veya .bldm şablonu yükle"
            >
              <Upload size={11} /> .fmt
            </button>
            <button
              type="button"
              className="uxigl-btn"
              style={{ padding: '2px 6px', fontSize: 11 }}
              onClick={onOpenDesigner}
              title="Form Tasarımcısında düzenle"
            >
              Düzenle
            </button>
          </div>
        </div>
        <div ref={comboRef} style={{ position: 'relative' }}>
          <div style={{ position: 'relative', display: 'flex', alignItems: 'center' }}>
            <Search
              size={12}
              style={{
                position: 'absolute',
                left: 9,
                color: 'var(--uxigl-text-muted)',
                pointerEvents: 'none',
              }}
            />
            <input
              ref={inputRef}
              type="text"
              className="uxigl-select"
              disabled={scanning}
              value={tplOpen ? tplQuery : activeTemplate?.name ?? ''}
              placeholder={activeTemplate ? activeTemplate.name : 'Optik form ara veya seç...'}
              onFocus={() => {
                if (scanning) return;
                setTplOpen(true);
                setTplQuery('');
              }}
              onChange={(e) => {
                setTplQuery(e.target.value);
                if (!tplOpen) setTplOpen(true);
              }}
              onKeyDown={(e) => {
                if (!tplOpen) {
                  if (e.key === 'ArrowDown' || e.key === 'Enter') {
                    e.preventDefault();
                    setTplOpen(true);
                  }
                  return;
                }
                if (e.key === 'ArrowDown') {
                  e.preventDefault();
                  setHighlightIdx((i) => Math.min(filteredTemplates.length - 1, i + 1));
                } else if (e.key === 'ArrowUp') {
                  e.preventDefault();
                  setHighlightIdx((i) => Math.max(0, i - 1));
                } else if (e.key === 'Enter') {
                  e.preventDefault();
                  const target = filteredTemplates[highlightIdx];
                  if (target) commitTemplate(target.id);
                } else if (e.key === 'Escape') {
                  e.preventDefault();
                  setTplOpen(false);
                  setTplQuery('');
                  inputRef.current?.blur();
                }
              }}
              style={{
                width: '100%',
                paddingLeft: 26,
                paddingRight: 26,
                textOverflow: 'ellipsis',
              }}
            />
            {tplOpen && tplQuery ? (
              <button
                type="button"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => setTplQuery('')}
                title="Temizle"
                style={{
                  position: 'absolute',
                  right: 6,
                  background: 'transparent',
                  border: 'none',
                  color: 'var(--uxigl-text-muted)',
                  cursor: 'pointer',
                  padding: 2,
                  display: 'flex',
                }}
              >
                <X size={12} />
              </button>
            ) : (
              <button
                type="button"
                disabled={scanning}
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => {
                  if (scanning) return;
                  setTplOpen((o) => {
                    const next = !o;
                    if (next) {
                      setTplQuery('');
                      setTimeout(() => inputRef.current?.focus(), 0);
                    }
                    return next;
                  });
                }}
                style={{
                  position: 'absolute',
                  right: 6,
                  background: 'transparent',
                  border: 'none',
                  color: 'var(--uxigl-text-muted)',
                  cursor: scanning ? 'default' : 'pointer',
                  padding: 2,
                  display: 'flex',
                }}
              >
                <ChevronDown size={13} />
              </button>
            )}
          </div>

          {tplOpen && !scanning && (
            <div
              className="uxigl-glass-card"
              style={{
                position: 'absolute',
                top: 'calc(100% + 4px)',
                left: 0,
                right: 0,
                maxHeight: 230,
                overflowY: 'auto',
                zIndex: 60,
                padding: 4,
                display: 'flex',
                flexDirection: 'column',
                gap: 2,
                boxShadow: '0 12px 28px rgba(0, 0, 0, 0.28)',
                background: 'var(--uxigl-surface, #ffffff)',
              }}
            >
              {filteredTemplates.length === 0 ? (
                <div style={{ padding: '8px 10px', fontSize: 11, color: 'var(--uxigl-text-muted)', textAlign: 'center' }}>
                  Eşleşen şablon bulunamadı
                </div>
              ) : (
                filteredTemplates.map((t, idx) => {
                  const isSelected = t.id === activeTemplate?.id;
                  const isHighlighted = idx === highlightIdx;
                  return (
                    <button
                      key={t.id}
                      type="button"
                      onMouseDown={(e) => e.preventDefault()}
                      onMouseEnter={() => setHighlightIdx(idx)}
                      onClick={() => commitTemplate(t.id)}
                      style={{
                        display: 'flex',
                        flexDirection: 'column',
                        alignItems: 'flex-start',
                        gap: 2,
                        width: '100%',
                        padding: '6px 8px',
                        borderRadius: 6,
                        border: 'none',
                        cursor: 'pointer',
                        textAlign: 'left',
                        background: isHighlighted
                          ? 'rgba(56, 189, 248, 0.16)'
                          : isSelected
                            ? 'rgba(56, 189, 248, 0.08)'
                            : 'transparent',
                        color: 'inherit',
                      }}
                    >
                      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', width: '100%', gap: 6 }}>
                        <span
                          style={{
                            fontSize: 12,
                            fontWeight: isSelected ? 600 : 500,
                            overflow: 'hidden',
                            textOverflow: 'ellipsis',
                            whiteSpace: 'nowrap',
                          }}
                        >
                          {t.name}
                        </span>
                        {isSelected && <Check size={12} color="#38bdf8" style={{ flexShrink: 0 }} />}
                      </div>
                      <span style={{ fontSize: 10, color: 'var(--uxigl-text-muted)' }}>
                        {t.rows}×{t.cols} · {t.orientation === 'T' ? 'Sağ Şerit (T - 180°)' : 'Sol Şerit (D)'} · {t.fields.length} alan
                      </span>
                    </button>
                  );
                })
              )}
            </div>
          )}
        </div>
        {activeTemplate && (
          <>
            <div style={{ fontSize: 11, color: 'var(--uxigl-text-muted)', display: 'flex', justifyContent: 'space-between' }}>
              <span>Izgara: {activeTemplate.rows} çizgi × {activeTemplate.cols} sütun</span>
              <span>{activeTemplate.fields.length} alan</span>
            </div>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', gap: 6, marginTop: 2 }}>
              <span style={{ fontSize: 11, color: 'var(--uxigl-text-muted)', display: 'inline-flex', alignItems: 'center', gap: 4 }}>
                <span>Şerit / Okuma Yönü</span>
                <button
                  type="button"
                  onClick={() => setShowOrientationHelp((v) => !v)}
                  title="Şerit yönü ve kağıt yerleştirme rehberini göster/gizle"
                  style={{
                    background: 'transparent',
                    border: 'none',
                    padding: 0,
                    cursor: 'pointer',
                    color: showOrientationHelp ? '#38bdf8' : 'var(--uxigl-text-muted)',
                    display: 'inline-flex',
                    alignItems: 'center',
                  }}
                >
                  <HelpCircle size={13} />
                </button>
              </span>
              <select
                className="uxigl-select"
                disabled={scanning}
                style={{ fontSize: 11, padding: '3px 6px', width: 168 }}
                value={activeTemplate.orientation === 'T' ? 'T' : 'D'}
                onChange={(e) => onChangeTemplateOrientation?.(e.target.value === 'T' ? 'T' : 'D')}
                title="Kağıdın okuma çizgileri (siyah şeritler) düz tutulduğunda soldaysa Düz (D), sağdaysa Ters (T) seçilir."
              >
                <option value="D">Sol Şerit • Düz (D)</option>
                <option value="T">Sağ Şerit • Ters (T - 180°)</option>
              </select>
            </div>
            {showOrientationHelp && (
              <div
                style={{
                  marginTop: 4,
                  padding: '8px 9px',
                  borderRadius: 8,
                  border: '1px solid var(--uxigl-border)',
                  background: 'rgba(56, 189, 248, 0.06)',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: 7,
                }}
              >
                <div style={{ fontSize: 10.5, lineHeight: 1.38, color: 'var(--uxigl-text-muted)' }}>
                  Cihazın siyah şerit sensörü <strong style={{ color: 'var(--uxigl-text)' }}>sol taraftadır</strong>. Kağıdı tepsiye koyarken şeritler daima{' '}
                  <strong style={{ color: '#0284c7' }}>sol kenara</strong> gelmelidir:
                </div>

                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6 }}>
                  {/* Sol Şerit • Düz (D) Görseli */}
                  <button
                    type="button"
                    disabled={scanning}
                    onClick={() => onChangeTemplateOrientation?.('D')}
                    style={{
                      textAlign: 'left',
                      padding: '6px',
                      borderRadius: 7,
                      cursor: scanning ? 'default' : 'pointer',
                      border:
                        activeTemplate.orientation !== 'T'
                          ? '1.5px solid #0ea5e9'
                          : '1px solid var(--uxigl-border)',
                      background:
                        activeTemplate.orientation !== 'T'
                          ? 'rgba(14, 165, 233, 0.12)'
                          : 'rgba(148, 163, 184, 0.06)',
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      gap: 4,
                      color: 'inherit',
                    }}
                  >
                    <svg width="88" height="72" viewBox="0 0 88 72" aria-label="Sol şerit düz form yerleşimi">
                      {/* Sensör işareti */}
                      <rect x="2" y="10" width="4" height="54" rx="2" fill="#0ea5e9" opacity="0.3" />
                      {/* Kağıt */}
                      <rect x="10" y="10" width="54" height="58" rx="3" fill="#ffffff" stroke="#94a3b8" strokeWidth="1.2" />
                      {/* Sol tarafta siyah okuma şeritleri */}
                      {[14, 20, 26, 32, 38, 44, 50, 56, 62].map((y) => (
                        <rect key={y} x="13" y={y} width="4.5" height="3" rx="0.6" fill="#0f172a" />
                      ))}
                      {/* Düz başlık ve baloncuklar */}
                      <rect x="21" y="15" width="38" height="5" rx="1" fill="#e11d48" opacity="0.82" />
                      <text x="40" y="19" textAnchor="middle" fontSize="4.5" fontWeight="700" fill="#ffffff">
                        AD SOYAD
                      </text>
                      {[25, 31, 37, 43, 49, 55].map((y, idx) => (
                        <g key={y}>
                          <circle cx="24" cy={y} r="1.7" fill={idx % 2 === 0 ? '#0f172a' : '#cbd5e1'} />
                          <circle cx="30" cy={y} r="1.7" fill="#cbd5e1" />
                          <circle cx="36" cy={y} r="1.7" fill={idx % 2 === 1 ? '#0f172a' : '#cbd5e1'} />
                          <circle cx="46" cy={y} r="1.7" fill="#cbd5e1" />
                          <circle cx="52" cy={y} r="1.7" fill={idx === 2 ? '#0f172a' : '#cbd5e1'} />
                          <circle cx="58" cy={y} r="1.7" fill="#cbd5e1" />
                        </g>
                      ))}
                      {/* Giriş yönü oku */}
                      <path d="M74 56 L74 18 M70 23 L74 17 L78 23" stroke="#10b981" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round" />
                      <text x="74" y="66" textAnchor="middle" fontSize="6" fontWeight="700" fill="#10b981">
                        GİRİŞ
                      </text>
                    </svg>
                    <div style={{ fontSize: 10, fontWeight: 700, textAlign: 'center' }}>
                      Düz (D) • Sol Şerit
                    </div>
                    <div style={{ fontSize: 9.5, color: 'var(--uxigl-text-muted)', textAlign: 'center', lineHeight: 1.25 }}>
                      Yazılar düz, siyah şeritler solda okutulur.
                    </div>
                  </button>

                  {/* Sağ Şerit • Ters (T - 180°) Görseli */}
                  <button
                    type="button"
                    disabled={scanning}
                    onClick={() => onChangeTemplateOrientation?.('T')}
                    style={{
                      textAlign: 'left',
                      padding: '6px',
                      borderRadius: 7,
                      cursor: scanning ? 'default' : 'pointer',
                      border:
                        activeTemplate.orientation === 'T'
                          ? '1.5px solid #f59e0b'
                          : '1px solid var(--uxigl-border)',
                      background:
                        activeTemplate.orientation === 'T'
                          ? 'rgba(245, 158, 11, 0.13)'
                          : 'rgba(148, 163, 184, 0.06)',
                      display: 'flex',
                      flexDirection: 'column',
                      alignItems: 'center',
                      gap: 4,
                      color: 'inherit',
                    }}
                  >
                    <svg width="88" height="72" viewBox="0 0 88 72" aria-label="Sağ şerit ters form yerleşimi">
                      {/* Sensör işareti */}
                      <rect x="2" y="10" width="4" height="54" rx="2" fill="#f59e0b" opacity="0.35" />
                      {/* 180° çevrilmiş kağıt */}
                      <rect x="10" y="10" width="54" height="58" rx="3" fill="#ffffff" stroke="#94a3b8" strokeWidth="1.2" />
                      {/* Çevrildiği için sola gelen siyah şeritler */}
                      {[14, 20, 26, 32, 38, 44, 50, 56, 62].map((y) => (
                        <rect key={y} x="13" y={y} width="4.5" height="3" rx="0.6" fill="#0f172a" />
                      ))}
                      {/* Baş aşağı duran başlık (180° döndürülmüş) */}
                      <rect x="21" y="58" width="38" height="5" rx="1" fill="#e11d48" opacity="0.82" />
                      <g transform="rotate(180 40 60.5)">
                        <text x="40" y="62.2" textAnchor="middle" fontSize="4.5" fontWeight="700" fill="#ffffff">
                          AD SOYAD
                        </text>
                      </g>
                      {[17, 23, 29, 35, 41, 47].map((y, idx) => (
                        <g key={y}>
                          <circle cx="24" cy={y} r="1.7" fill={idx === 1 ? '#0f172a' : '#cbd5e1'} />
                          <circle cx="30" cy={y} r="1.7" fill="#cbd5e1" />
                          <circle cx="36" cy={y} r="1.7" fill={idx % 2 === 0 ? '#0f172a' : '#cbd5e1'} />
                          <circle cx="46" cy={y} r="1.7" fill="#cbd5e1" />
                          <circle cx="52" cy={y} r="1.7" fill={idx % 2 === 1 ? '#0f172a' : '#cbd5e1'} />
                          <circle cx="58" cy={y} r="1.7" fill="#cbd5e1" />
                        </g>
                      ))}
                      {/* 180° çevirme + giriş oku */}
                      <text x="74" y="16" textAnchor="middle" fontSize="6.5" fontWeight="700" fill="#d97706">
                        180°
                      </text>
                      <path d="M74 56 L74 23 M70 28 L74 22 L78 28" stroke="#10b981" strokeWidth="2" fill="none" strokeLinecap="round" strokeLinejoin="round" />
                      <text x="74" y="66" textAnchor="middle" fontSize="6" fontWeight="700" fill="#10b981">
                        GİRİŞ
                      </text>
                    </svg>
                    <div style={{ fontSize: 10, fontWeight: 700, textAlign: 'center' }}>
                      Ters (T) • Sağ Şerit
                    </div>
                    <div style={{ fontSize: 9.5, color: 'var(--uxigl-text-muted)', textAlign: 'center', lineHeight: 1.25 }}>
                      Şerit sağda basılıdır; 180° baş aşağı konur.
                    </div>
                  </button>
                </div>
              </div>
            )}
          </>
        )}
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
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  gap: 6,
                  fontSize: 11,
                  color: '#10b981',
                  background: 'rgba(16, 185, 129, 0.12)',
                  padding: '5px 8px',
                  borderRadius: 6,
                  fontWeight: 600,
                }}
              >
                <span style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
                  <ShieldCheck size={13} />
                  <span>Yerel Ajan Bağlı {agentInfo.version ? `(v${agentInfo.version})` : ''}</span>
                </span>
                {(agentUpdate.available || agentUpdate.updating) ? (
                  <button
                    type="button"
                    className="uxigl-btn"
                    disabled={agentUpdate.updating}
                    onClick={onUpdateAgent}
                    style={{
                      padding: '2px 8px',
                      fontSize: 11,
                      background: 'rgba(14, 165, 233, 0.16)',
                      borderColor: 'rgba(14, 165, 233, 0.45)',
                      color: '#0284c7',
                    }}
                  >
                    {agentUpdate.updating ? (
                      <>
                        <Loader2 size={11} className="uxigl-spin" />
                        <span>Güncelleniyor...</span>
                      </>
                    ) : (
                      <>
                        <RefreshCw size={11} />
                        <span>
                          Ajanı Güncelle{agentUpdate.latestVersion ? ` (v${agentUpdate.latestVersion})` : ''}
                        </span>
                      </>
                    )}
                  </button>
                ) : (
                  <button
                    type="button"
                    className="uxigl-btn"
                    disabled={agentUpdate.checking}
                    onClick={onCheckAgentUpdate}
                    title="Yeni ajan sürümü olup olmadığını kontrol et"
                    style={{
                      padding: '2px 8px',
                      fontSize: 10,
                      whiteSpace: 'nowrap',
                      flexShrink: 0,
                      opacity: agentUpdate.checkedUpToDate ? 1 : 0.9,
                      color: agentUpdate.checkedUpToDate ? '#059669' : undefined,
                      borderColor: agentUpdate.checkedUpToDate ? 'rgba(16, 185, 129, 0.45)' : undefined,
                    }}
                  >
                    {agentUpdate.checking ? (
                      <>
                        <Loader2 size={10} className="uxigl-spin" />
                        <span>Kontrol...</span>
                      </>
                    ) : agentUpdate.checkedUpToDate ? (
                      <>
                        <Check size={10} />
                        <span>Güncelsiniz</span>
                      </>
                    ) : (
                      <>
                        <RefreshCw size={10} />
                        <span>Kontrol Et</span>
                      </>
                    )}
                  </button>
                )}
              </div>
              {agentUpdate.error && (
                <div
                  style={{
                    fontSize: 11,
                    color: '#b45309',
                    background: 'rgba(245, 158, 11, 0.14)',
                    border: '1px solid rgba(245, 158, 11, 0.35)',
                    padding: '6px 8px',
                    borderRadius: 6,
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 4,
                  }}
                >
                  <span>{agentUpdate.error}</span>
                  <a
                    href={`/downloads/sekonic-agent.exe?t=${Date.now()}`}
                    download="sekonic-agent.exe"
                    style={{ color: '#0284c7', fontWeight: 600, textDecoration: 'underline' }}
                  >
                    Yeni sekonic-agent.exe dosyasını manuel indir
                  </a>
                </div>
              )}
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
                  href={`/downloads/sekonic-agent.exe?t=${Date.now()}`}
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
            <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 12, gap: 8 }}>
              <span style={{ color: 'var(--uxigl-text-muted)' }}>Baud Rate</span>
              <select
                className="uxigl-select"
                style={{ width: 130 }}
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
            <Gauge size={14} color="#10b981" /> {scanning ? (simulatorMode ? '133 kağıt/dk' : '108 kağıt/dk') : '0 kağıt/dk'}
          </div>
        </div>
        <div className="uxigl-subcard" style={{ padding: 8 }}>
          <div style={{ color: 'var(--uxigl-text-muted)' }}>Hopper</div>
          <div style={{ fontWeight: 700, fontSize: 15 }}>
            {scanning ? 'Okunuyor' : 'Hazır'}
          </div>
        </div>
      </div>

      <label style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: 12, gap: 8 }}>
        <span style={{ color: 'var(--uxigl-text-muted)' }}>Dropout LED Rengi</span>
        <select
          className="uxigl-select"
          style={{ width: 130 }}
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
