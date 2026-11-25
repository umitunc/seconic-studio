import React, { useEffect, useRef, useState } from 'react';
import {
  Trash2,
  Download,
  Upload,
  MousePointer,
  SquareDashedMousePointer,
  ZoomIn,
  ZoomOut,
  FileText,
} from 'lucide-react';
import { ThresholdPolicy } from '../../types/scanner';
import {
  DEFAULT_TEMPLATES,
  DesignerField,
  FIELD_COLORS as COLORS,
  FormTemplate,
  parseFmtOrBldm,
  serializeTemplateToFmt,
} from '../../utils/formTemplates';
import { decodeAutoTurkishText, encodeWindows1254 } from '../../utils/encoding';

export type { DesignerField, FormTemplate };

const DEFAULT_TEMPLATE = DEFAULT_TEMPLATES[0];

interface FormDesignerViewProps {
  theme: 'light' | 'dark';
  policy: ThresholdPolicy;
  onPolicyChange: (p: ThresholdPolicy) => void;
  templates: FormTemplate[];
  activeTemplateId: string;
  onTemplatesChange: React.Dispatch<React.SetStateAction<FormTemplate[]>>;
  onSelectTemplate: (id: string) => void;
}

export const FormDesignerView: React.FC<FormDesignerViewProps> = ({
  theme,
  policy,
  templates,
  activeTemplateId,
  onTemplatesChange: setTemplates,
  onSelectTemplate: setActiveTemplateId,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const canvasWrapRef = useRef<HTMLDivElement | null>(null);
  const [templateSearch, setTemplateSearch] = useState('');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [toolMode, setToolMode] = useState<'select' | 'draw'>('select');
  const [zoom, setZoom] = useState<number>(1);
  const [tplListWidth, setTplListWidth] = useState<number>(() => {
    const saved = Number(localStorage.getItem('sekonic-tpllist-w'));
    return saved >= 180 && saved <= 460 ? saved : 250;
  });
  const [dragStart, setDragStart] = useState<{ c: number; r: number } | null>(null);
  const [dragEnd, setDragEnd] = useState<{ c: number; r: number } | null>(null);
  const [moveOffset, setMoveOffset] = useState<{ dc: number; dr: number } | null>(null);

  const activeTemplate = templates.find((t) => t.id === activeTemplateId) ?? templates[0] ?? DEFAULT_TEMPLATE;
  const fields = activeTemplate.fields;
  const gridDims = { rows: activeTemplate.rows, cols: activeTemplate.cols };
  const selectedField = fields.find((f) => f.id === selectedId) ?? null;

  const updateActiveTemplate = (updater: (tpl: FormTemplate) => FormTemplate) => {
    setTemplates((prev) => prev.map((t) => (t.id === activeTemplate.id ? updater(t) : t)));
  };

  const setFields = (updater: (prev: DesignerField[]) => DesignerField[]) => {
    updateActiveTemplate((tpl) => ({ ...tpl, fields: updater(tpl.fields) }));
  };

  const toGridCoords = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const canvas = canvasRef.current;
    if (!canvas) return { c: 0, r: 0 };
    const rect = canvas.getBoundingClientRect();
    const x = ((e.clientX - rect.left) / rect.width) * gridDims.cols;
    const y = ((e.clientY - rect.top) / rect.height) * gridDims.rows;
    return {
      c: Math.max(0, Math.min(gridDims.cols - 1, Math.floor(x))),
      r: Math.max(0, Math.min(gridDims.rows - 1, Math.floor(y))),
    };
  };

  useEffect(() => {
    const el = canvasWrapRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const delta = e.deltaY < 0 ? 0.15 : -0.15;
      setZoom((z) => Math.min(2.5, Math.max(0.75, +(z + delta).toFixed(2))));
    };
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.key === 'Delete' || e.key === 'Backspace') && selectedId) {
        const tag = (e.target as HTMLElement)?.tagName;
        if (tag === 'INPUT' || tag === 'SELECT' || tag === 'TEXTAREA') return;
        setFields((prev) => prev.filter((f) => f.id !== selectedId));
        setSelectedId(null);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [selectedId, activeTemplate.id]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const cssW = Math.max(360, Math.round(rect.width));
    const cssH = Math.round(cssW * 0.72);
    if (canvas.width !== Math.round(cssW * dpr) || canvas.height !== Math.round(cssH * dpr)) {
      canvas.width = Math.round(cssW * dpr);
      canvas.height = Math.round(cssH * dpr);
    }
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.save();
    ctx.scale(dpr, dpr);
    ctx.fillStyle = theme === 'dark' ? '#141518' : '#ffffff';
    ctx.fillRect(0, 0, cssW, cssH);

    ctx.strokeStyle = theme === 'dark' ? 'rgba(255, 255, 255, 0.06)' : 'rgba(24, 24, 27, 0.07)';
    ctx.lineWidth = 1;
    const cw = cssW / gridDims.cols;
    const ch = cssH / gridDims.rows;

    for (let c = 0; c <= gridDims.cols; c++) {
      const x = Math.round(c * cw) + 0.5;
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, cssH);
      ctx.stroke();
    }
    for (let r = 0; r <= gridDims.rows; r++) {
      const y = Math.round(r * ch) + 0.5;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(cssW, y);
      ctx.stroke();
    }

    // Draw scanner reference timing stripes along the left sensor edge (col 0)
    ctx.fillStyle = theme === 'dark' ? 'rgba(244, 244, 245, 0.55)' : 'rgba(24, 24, 27, 0.65)';
    for (let r = 0; r < gridDims.rows; r++) {
      const barH = Math.max(2, ch * 0.46);
      const barW = Math.max(4, Math.min(10, cw * 0.55));
      const y = r * ch + (ch - barH) / 2;
      ctx.fillRect(2, y, barW, barH);
    }

    for (const f of fields) {
      const isSel = f.id === selectedId;
      const x = Math.round(f.startCol * cw) + 0.5;
      const y = Math.round(f.startRow * ch) + 0.5;
      const w = Math.round(f.cols * cw);
      const h = Math.round(f.rows * ch);

      ctx.fillStyle = f.color;
      ctx.fillRect(x, y, w, h);

      const radius = Math.max(1.4, Math.min(cw, ch) * 0.28);
      ctx.strokeStyle = theme === 'dark' ? 'rgba(255,255,255,0.22)' : 'rgba(24,24,27,0.24)';
      ctx.lineWidth = 0.8;
      for (let rr = 0; rr < f.rows; rr++) {
        for (let cc = 0; cc < f.cols; cc++) {
          const cx = (f.startCol + cc) * cw + cw / 2;
          const cy = (f.startRow + rr) * ch + ch / 2;
          ctx.beginPath();
          ctx.arc(cx, cy, radius, 0, Math.PI * 2);
          ctx.stroke();
        }
      }

      ctx.strokeStyle = isSel
        ? theme === 'dark' ? '#ffffff' : '#18181b'
        : theme === 'dark' ? 'rgba(255,255,255,0.38)' : 'rgba(24,24,27,0.4)';
      ctx.lineWidth = isSel ? 2 : 1;
      ctx.strokeRect(x, y, w, h);

      ctx.fillStyle = theme === 'dark' ? 'rgba(17,18,20,0.82)' : 'rgba(255,255,255,0.88)';
      ctx.font = '700 11px -apple-system, BlinkMacSystemFont, Inter, sans-serif';
      const tw = ctx.measureText(f.label).width + 8;
      ctx.fillRect(x + 2, y + 2, Math.min(w - 4, tw), 16);
      ctx.fillStyle = theme === 'dark' ? '#f4f4f5' : '#18181b';
      ctx.fillText(f.label, x + 6, y + 14, Math.max(10, w - 10));
    }

    if (toolMode === 'draw' && dragStart && dragEnd) {
      const sc = Math.min(dragStart.c, dragEnd.c);
      const sr = Math.min(dragStart.r, dragEnd.r);
      const cols = Math.abs(dragEnd.c - dragStart.c) + 1;
      const rows = Math.abs(dragEnd.r - dragStart.r) + 1;
      ctx.fillStyle = 'rgba(24, 24, 27, 0.2)';
      ctx.fillRect(Math.round(sc * cw), Math.round(sr * ch), Math.round(cols * cw), Math.round(rows * ch));
    }
    ctx.restore();
  }, [theme, fields, gridDims, selectedId, toolMode, dragStart, dragEnd, zoom]);

  const handleMouseDown = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const pt = toGridCoords(e);
    if (toolMode === 'select') {
      const hit = [...fields].reverse().find(
        (f) =>
          pt.c >= f.startCol &&
          pt.c < f.startCol + f.cols &&
          pt.r >= f.startRow &&
          pt.r < f.startRow + f.rows,
      );
      if (hit) {
        setSelectedId(hit.id);
        setMoveOffset({ dc: pt.c - hit.startCol, dr: pt.r - hit.startRow });
      } else {
        setSelectedId(null);
      }
      return;
    }
    setDragStart(pt);
    setDragEnd(pt);
  };

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const pt = toGridCoords(e);
    if (toolMode === 'select' && moveOffset && selectedId) {
      setFields((prev) =>
        prev.map((f) =>
          f.id === selectedId
            ? {
                ...f,
                startCol: Math.max(0, Math.min(gridDims.cols - f.cols, pt.c - moveOffset.dc)),
                startRow: Math.max(0, Math.min(gridDims.rows - f.rows, pt.r - moveOffset.dr)),
              }
            : f,
        ),
      );
      return;
    }
    if (toolMode === 'draw' && dragStart) {
      setDragEnd(pt);
    }
  };

  const handleMouseUp = () => {
    setMoveOffset(null);
    if (toolMode === 'draw' && dragStart && dragEnd) {
      const sc = Math.min(dragStart.c, dragEnd.c);
      const sr = Math.min(dragStart.r, dragEnd.r);
      const cols = Math.abs(dragEnd.c - dragStart.c) + 1;
      const rows = Math.abs(dragEnd.r - dragStart.r) + 1;
      const newId = `f_${Date.now()}`;
      setFields((prev) => [
        ...prev,
        {
          id: newId,
          key: `alan_${prev.length + 1}`,
          label: `Yeni Alan ${prev.length + 1}`,
          fieldType: 'subject_questions',
          direction: 'Y',
          alphabet: 'ABCDE',
          startCol: sc,
          startRow: sr,
          cols,
          rows,
          color: COLORS[prev.length % COLORS.length],
        },
      ]);
      setSelectedId(newId);
      setToolMode('select');
    }
    setDragStart(null);
    setDragEnd(null);
  };

  const handleMultiFileImport = async (fileList: FileList | null) => {
    if (!fileList || fileList.length === 0) return;
    const imported: FormTemplate[] = [];
    for (const file of Array.from(fileList)) {
      const buf = await file.arrayBuffer();
      const txt = decodeAutoTurkishText(buf);
      const parsed = parseFmtOrBldm(txt);
      if (parsed) {
        const baseName = file.name.replace(/\.(fmt|bldm)$/i, '');
        imported.push({
          id: `tpl_${Date.now()}_${Math.random().toString(36).slice(2, 7)}`,
          name: baseName,
          rows: parsed.rows,
          cols: parsed.cols,
          orientation: parsed.orientation,
          markThreshold: parsed.markThreshold,
          multiMarkChar: parsed.multiMarkChar,
          blankChar: parsed.blankChar,
          fields: parsed.fields,
        });
      }
    }
    if (imported.length > 0) {
      setTemplates((prev) => [...imported, ...prev]);
      setActiveTemplateId(imported[0].id);
      setSelectedId(imported[0].fields[0]?.id ?? null);
    }
  };

  const updateSelected = (patch: Partial<DesignerField>) => {
    if (!selectedId) return;
    setFields((prev) => prev.map((f) => (f.id === selectedId ? { ...f, ...patch } : f)));
  };

  const exportFmt = () => {
    const fmtText = serializeTemplateToFmt(activeTemplate);
    const ansiBytes = encodeWindows1254(fmtText);
    const blob = new Blob([ansiBytes.buffer as ArrayBuffer], { type: 'text/plain;charset=windows-1254' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `${activeTemplate.name}.fmt`;
    a.click();
  };

  const exportJson = () => {
    const payload = {
      schema_version: 1,
      form_id: activeTemplate.name,
      orientation: activeTemplate.orientation ?? 'D',
      grid: { front_rows: gridDims.rows, front_cols: gridDims.cols },
      threshold_policy: policy,
      fields,
    };
    const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `${activeTemplate.name}.json`;
    a.click();
  };

  const visibleTemplates = templates.filter((t) =>
    t.name.toLowerCase().includes(templateSearch.toLowerCase()),
  );

  const startTplListResize = (e: React.MouseEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startW = tplListWidth;
    const onMove = (ev: MouseEvent) => {
      const next = Math.max(180, Math.min(460, startW + (ev.clientX - startX)));
      setTplListWidth(next);
      localStorage.setItem('sekonic-tpllist-w', String(next));
    };
    const onUp = () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  };

  return (
    <div style={{ display: 'flex', gap: 6, flex: 1, minWidth: 0 }}>
      {/* 1. Sol Şablon Listesi (Sidebar'ın hemen sağında, resize edilebilir) */}
      <div className="uxigl-glass-card" style={{ width: tplListWidth, flexShrink: 0, padding: 12, display: 'flex', flexDirection: 'column', gap: 10, minWidth: 0 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <span style={{ fontWeight: 700, fontSize: 13 }}>Şablonlar ({templates.length})</span>
          <label className="uxigl-btn uxigl-btn-primary" style={{ padding: '4px 8px', fontSize: 11 }}>
            <Upload size={12} /> Çoklu Ekle
            <input
              type="file"
              accept=".bldm,.fmt"
              multiple
              style={{ display: 'none' }}
              onChange={(e) => {
                void handleMultiFileImport(e.target.files);
                e.target.value = '';
              }}
            />
          </label>
        </div>

        <input
          type="text"
          className="uxigl-select"
          placeholder="Şablon ara..."
          value={templateSearch}
          onChange={(e) => setTemplateSearch(e.target.value)}
        />

        <div style={{ flex: 1, overflowY: 'auto', paddingRight: 8, display: 'flex', flexDirection: 'column', gap: 6, maxHeight: 'calc(100vh - 210px)' }}>
          {visibleTemplates.map((tpl) => {
            const isActive = tpl.id === activeTemplate.id;
            return (
              <div
                key={tpl.id}
                onClick={() => {
                  setActiveTemplateId(tpl.id);
                  setSelectedId(tpl.fields[0]?.id ?? null);
                }}
                className="uxigl-subcard"
                style={{
                  padding: '8px 9px',
                  cursor: 'pointer',
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  gap: 6,
                  borderColor: isActive ? 'var(--uxigl-text)' : undefined,
                  background: isActive ? 'var(--uxigl-accent-soft)' : undefined,
                }}
              >
                <div style={{ minWidth: 0 }}>
                  <div style={{ fontWeight: 700, fontSize: 12, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', display: 'flex', alignItems: 'center', gap: 5 }}>
                    <FileText size={12} /> {tpl.name}
                  </div>
                  <div style={{ fontSize: 10, color: 'var(--uxigl-text-muted)', marginTop: 2 }}>
                    {tpl.rows}×{tpl.cols} • {tpl.orientation === 'T' ? 'Ters (T)' : 'Düz (D)'} • {tpl.fields.length} alan
                  </div>
                </div>
                {templates.length > 1 && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      const next = templates.filter((x) => x.id !== tpl.id);
                      setTemplates(next);
                      if (isActive && next[0]) {
                        setActiveTemplateId(next[0].id);
                      }
                    }}
                    style={{ background: 'transparent', border: 'none', color: '#ef4444', cursor: 'pointer', flexShrink: 0 }}
                    title="Şablonu Sil"
                  >
                    <Trash2 size={12} />
                  </button>
                )}
              </div>
            );
          })}
        </div>
      </div>

      {/* Şablon Listesi Resize Tutamacı */}
      <div
        onMouseDown={startTplListResize}
        title="Şablon listesini yeniden boyutlandır"
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

      {/* 2. Orta Optik Izgara Tasarımcısı */}
      <div className="uxigl-glass-card" style={{ flex: 1, padding: 14, display: 'flex', flexDirection: 'column', gap: 10, minWidth: 0 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: 8 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap', minWidth: 0 }}>
            <input
              type="text"
              className="uxigl-select"
              value={activeTemplate.name}
              onChange={(e) => updateActiveTemplate((t) => ({ ...t, name: e.target.value }))}
              style={{ fontWeight: 700, fontSize: 13, width: 220 }}
              title="Şablon adını düzenle"
            />
            <label style={{ display: 'inline-flex', alignItems: 'center', gap: 4, fontSize: 11, color: 'var(--uxigl-text-muted)' }}>
              Çizgi
              <input
                type="number"
                className="uxigl-select"
                min={5}
                max={120}
                value={gridDims.rows}
                onChange={(e) =>
                  updateActiveTemplate((t) => ({ ...t, rows: Math.max(5, Math.min(120, Number(e.target.value) || 51)) }))
                }
                style={{ width: 56, padding: '4px 6px', fontSize: 11 }}
                title="Okuma çizgisi (satır) sayısı"
              />
            </label>
            <label style={{ display: 'inline-flex', alignItems: 'center', gap: 4, fontSize: 11, color: 'var(--uxigl-text-muted)' }}>
              Sütun
              <input
                type="number"
                className="uxigl-select"
                min={5}
                max={48}
                value={gridDims.cols}
                onChange={(e) =>
                  updateActiveTemplate((t) => ({ ...t, cols: Math.max(5, Math.min(48, Number(e.target.value) || 46)) }))
                }
                style={{ width: 54, padding: '4px 6px', fontSize: 11 }}
                title="Sütun sayısı (en fazla 48)"
              />
            </label>
            <select
              className="uxigl-select"
              value={activeTemplate.orientation === 'T' ? 'T' : 'D'}
              onChange={(e) =>
                updateActiveTemplate((t) => ({ ...t, orientation: e.target.value === 'T' ? 'T' : 'D' }))
              }
              style={{ fontSize: 11, padding: '4px 8px' }}
              title="Okuma çizgileri düz kağıtta soldaysa Düz (D), sağdaysa Ters (T - 180°)"
            >
              <option value="D">Düz (D • Sol Şerit)</option>
              <option value="T">Ters (T • Sağ Şerit 180°)</option>
            </select>
          </div>
          <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
            <button
              className={`uxigl-btn ${toolMode === 'select' ? 'uxigl-btn-primary' : ''}`}
              style={{ fontSize: 12, padding: '6px 10px' }}
              onClick={() => setToolMode('select')}
            >
              <MousePointer size={13} /> Seç
            </button>
            <button
              className={`uxigl-btn ${toolMode === 'draw' ? 'uxigl-btn-primary' : ''}`}
              style={{ fontSize: 12, padding: '6px 10px' }}
              onClick={() => setToolMode('draw')}
            >
              <SquareDashedMousePointer size={13} /> Çiz
            </button>
            <button
              className="uxigl-btn"
              style={{ fontSize: 12, padding: '6px 8px' }}
              onClick={() => setZoom((z) => Math.max(0.75, +(z - 0.25).toFixed(2)))}
            >
              <ZoomOut size={13} />
            </button>
            <button
              className="uxigl-btn"
              style={{ fontSize: 12, padding: '6px 8px' }}
              onClick={() => setZoom(1)}
            >
              %{Math.round(zoom * 100)}
            </button>
            <button
              className="uxigl-btn"
              style={{ fontSize: 12, padding: '6px 8px' }}
              onClick={() => setZoom((z) => Math.min(2.5, +(z + 0.25).toFixed(2)))}
            >
              <ZoomIn size={13} />
            </button>
            <button className="uxigl-btn" style={{ fontSize: 12, padding: '6px 10px' }} onClick={exportFmt} title=".fmt olarak indir">
              <Download size={13} /> .fmt
            </button>
            <button className="uxigl-btn" style={{ fontSize: 12, padding: '6px 10px' }} onClick={exportJson} title="JSON olarak indir">
              <Download size={13} /> JSON
            </button>
          </div>
        </div>

        <div ref={canvasWrapRef} style={{ overflow: 'auto', maxHeight: 'calc(100vh - 190px)', borderRadius: 10, border: '1px solid var(--uxigl-border)' }}>
          <canvas
            ref={canvasRef}
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
            style={{
              width: `${Math.round(zoom * 100)}%`,
              aspectRatio: '1 / 0.72',
              display: 'block',
              cursor: toolMode === 'draw' ? 'crosshair' : moveOffset ? 'grabbing' : 'pointer',
            }}
          />
        </div>
      </div>

      {/* 3. Sağ Alan Özellikleri & Tanımlı Alanlar */}
      <div style={{ width: 270, flexShrink: 0, display: 'flex', flexDirection: 'column', gap: 12, minWidth: 0 }}>
        <div className="uxigl-glass-card" style={{ padding: 14, display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 700, fontSize: 13 }}>Seçili Alan Özellikleri</span>
            {selectedField && (
              <button
                className="uxigl-btn uxigl-btn-danger"
                style={{ padding: '3px 8px', fontSize: 11 }}
                onClick={() => {
                  setFields((prev) => prev.filter((x) => x.id !== selectedField.id));
                  setSelectedId(null);
                }}
              >
                <Trash2 size={12} /> Sil
              </button>
            )}
          </div>
          {selectedField ? (
            <>
              <input
                type="text"
                className="uxigl-select"
                value={selectedField.label}
                onChange={(e) => updateSelected({ label: e.target.value })}
              />
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6 }}>
                <select
                  className="uxigl-select"
                  value={selectedField.fieldType}
                  onChange={(e) => updateSelected({ fieldType: e.target.value as DesignerField['fieldType'] })}
                >
                  <option value="subject_questions">Soru Bloğu</option>
                  <option value="student_id">Kimlik / Ad</option>
                  <option value="booklet_type">Kitapçık</option>
                </select>
                <select
                  className="uxigl-select"
                  value={selectedField.direction}
                  onChange={(e) => updateSelected({ direction: e.target.value as 'D' | 'Y' })}
                >
                  <option value="D">Dikey (D)</option>
                  <option value="Y">Yatay (Y)</option>
                </select>
              </div>
              <input
                type="text"
                className="uxigl-select"
                value={selectedField.alphabet}
                onChange={(e) => updateSelected({ alphabet: e.target.value })}
                placeholder="Alfabe (örn. 12345 / ABCDE)"
              />
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6, fontSize: 11 }}>
                <label>
                  Başlangıç Satır
                  <input
                    type="number"
                    className="uxigl-select"
                    style={{ width: '100%', marginTop: 2 }}
                    min={1}
                    max={gridDims.rows}
                    value={selectedField.startRow + 1}
                    onChange={(e) => updateSelected({ startRow: Math.max(0, Number(e.target.value) - 1) })}
                  />
                </label>
                <label>
                  Başlangıç Sütun
                  <input
                    type="number"
                    className="uxigl-select"
                    style={{ width: '100%', marginTop: 2 }}
                    min={1}
                    max={gridDims.cols}
                    value={selectedField.startCol + 1}
                    onChange={(e) => updateSelected({ startCol: Math.max(0, Number(e.target.value) - 1) })}
                  />
                </label>
                <label>
                  Satır Sayısı
                  <input
                    type="number"
                    className="uxigl-select"
                    style={{ width: '100%', marginTop: 2 }}
                    min={1}
                    max={gridDims.rows}
                    value={selectedField.rows}
                    onChange={(e) => updateSelected({ rows: Math.max(1, Number(e.target.value)) })}
                  />
                </label>
                <label>
                  Sütun Sayısı
                  <input
                    type="number"
                    className="uxigl-select"
                    style={{ width: '100%', marginTop: 2 }}
                    min={1}
                    max={gridDims.cols}
                    value={selectedField.cols}
                    onChange={(e) => updateSelected({ cols: Math.max(1, Number(e.target.value)) })}
                  />
                </label>
              </div>
            </>
          ) : (
            <div style={{ fontSize: 12, color: 'var(--uxigl-text-muted)' }}>
              Düzenlemek veya silmek için ızgaradan ya da aşağıdan bir alan seçin.
            </div>
          )}
        </div>

        <div className="uxigl-glass-card" style={{ padding: 14, display: 'flex', flexDirection: 'column', gap: 6, flex: 1, overflowY: 'auto', overflowX: 'hidden', paddingRight: 10 }}>
          <div style={{ fontWeight: 700, fontSize: 13 }}>Tanımlı Alanlar ({fields.length})</div>
          {fields.map((f) => (
            <div
              key={f.id}
              onClick={() => setSelectedId(f.id)}
              className="uxigl-subcard"
              style={{
                padding: '7px 9px',
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                gap: 6,
                fontSize: 11,
                cursor: 'pointer',
                borderColor: f.id === selectedId ? 'var(--uxigl-text)' : undefined,
              }}
            >
              <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                <strong>{f.label}</strong> • R{f.startRow + 1}..{f.startRow + f.rows}:C{f.startCol + 1}..{f.startCol + f.cols}
              </span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setFields((prev) => prev.filter((x) => x.id !== f.id));
                  if (selectedId === f.id) setSelectedId(null);
                }}
                style={{ background: 'transparent', border: 'none', color: '#ef4444', cursor: 'pointer', flexShrink: 0 }}
                title="Alanı Sil"
              >
                <Trash2 size={13} />
              </button>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
