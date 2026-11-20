import React, { useEffect, useRef, useState } from 'react';
import { Monitor, Moon, Palette, RotateCcw, Sun, X } from 'lucide-react';

export type ThemeMode = 'light' | 'dark' | 'system';
export type FontFamilyKey = 'system' | 'readable' | 'mono';
export type SeedPaletteKey = 'graphite' | 'sapphire' | 'iris' | 'emerald' | 'coral';

export interface UiThemePrefs {
  mode: ThemeMode;
  fontSize: number;
  fontFamily: FontFamilyKey;
  palette: SeedPaletteKey;
  radius: number;
  sidebarOpen: boolean;
}

export const DEFAULT_THEME_PREFS: UiThemePrefs = {
  mode: 'light',
  fontSize: 13,
  fontFamily: 'system',
  palette: 'graphite',
  radius: 12,
  sidebarOpen: true,
};

export const FONT_STACKS: Record<FontFamilyKey, string> = {
  system: "-apple-system, BlinkMacSystemFont, 'Inter', 'Segoe UI', sans-serif",
  readable: "'Segoe UI', Tahoma, Arial, sans-serif",
  mono: "'Cascadia Code', 'JetBrains Mono', Consolas, monospace",
};

export const SEED_PALETTES: Record<
  SeedPaletteKey,
  {
    name: string;
    subtitle: string;
    dots: [string, string, string];
    accent: string;
    accentSoft: string;
    primaryLight: string;
    primaryDark: string;
  }
> = {
  graphite: {
    name: 'Graphite Classic',
    subtitle: 'Minimal Studio',
    dots: ['#18181b', '#e4e4e7', '#52525b'],
    accent: '#38bdf8',
    accentSoft: 'rgba(24, 24, 27, 0.07)',
    primaryLight: '#18181b',
    primaryDark: '#e4e4e7',
  },
  sapphire: {
    name: 'Sapphire Blue',
    subtitle: 'Liquid Glass',
    dots: ['#2563eb', '#dbeafe', '#4f46e5'],
    accent: '#2563eb',
    accentSoft: 'rgba(37, 99, 235, 0.12)',
    primaryLight: '#2563eb',
    primaryDark: '#60a5fa',
  },
  iris: {
    name: 'Royal Iris',
    subtitle: 'Dynamic Tonal',
    dots: ['#7c3aed', '#ede9fe', '#6d28d9'],
    accent: '#7c3aed',
    accentSoft: 'rgba(124, 58, 237, 0.12)',
    primaryLight: '#7c3aed',
    primaryDark: '#a78bfa',
  },
  emerald: {
    name: 'Emerald Mint',
    subtitle: 'Fresh Focus',
    dots: ['#059669', '#d1fae5', '#0d9488'],
    accent: '#059669',
    accentSoft: 'rgba(5, 150, 105, 0.12)',
    primaryLight: '#059669',
    primaryDark: '#34d399',
  },
  coral: {
    name: 'Solar Coral',
    subtitle: 'Warm Contrast',
    dots: ['#e11d48', '#ffe4e6', '#f97316'],
    accent: '#e11d48',
    accentSoft: 'rgba(225, 29, 72, 0.12)',
    primaryLight: '#e11d48',
    primaryDark: '#fb7185',
  },
};

interface UxiglThemeStudioProps {
  open: boolean;
  onClose: () => void;
  prefs: UiThemePrefs;
  resolvedMode: 'light' | 'dark';
  sidebarWidth: number;
  onChangeSidebarWidth: (w: number) => void;
  onChangePrefs: (next: UiThemePrefs) => void;
  onReset: () => void;
}

export const UxiglThemeStudio: React.FC<UxiglThemeStudioProps> = ({
  open,
  onClose,
  prefs,
  resolvedMode,
  sidebarWidth,
  onChangeSidebarWidth,
  onChangePrefs,
  onReset,
}) => {
  const [pos, setPos] = useState<{ x: number; y: number }>(() => ({
    x: Math.max(24, window.innerWidth - 460),
    y: 66,
  }));
  const [size, setSize] = useState<{ w: number; h: number }>({ w: 420, h: 640 });
  const dragRef = useRef<{ startX: number; startY: number; origX: number; origY: number } | null>(null);

  useEffect(() => {
    if (!open) return;
    setPos((cur) => ({
      x: Math.min(Math.max(12, cur.x), Math.max(12, window.innerWidth - size.w - 12)),
      y: Math.min(Math.max(12, cur.y), Math.max(12, window.innerHeight - 160)),
    }));
  }, [open, size.w]);

  const startDrag = (e: React.MouseEvent) => {
    if ((e.target as HTMLElement).closest('button')) return;
    e.preventDefault();
    dragRef.current = { startX: e.clientX, startY: e.clientY, origX: pos.x, origY: pos.y };
    const onMove = (ev: MouseEvent) => {
      if (!dragRef.current) return;
      const nx = Math.max(8, Math.min(window.innerWidth - 160, dragRef.current.origX + (ev.clientX - dragRef.current.startX)));
      const ny = Math.max(8, Math.min(window.innerHeight - 80, dragRef.current.origY + (ev.clientY - dragRef.current.startY)));
      setPos({ x: nx, y: ny });
    };
    const onUp = () => {
      dragRef.current = null;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  };

  const startResize = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const startX = e.clientX;
    const startY = e.clientY;
    const origW = size.w;
    const origH = size.h;
    const onMove = (ev: MouseEvent) => {
      setSize({
        w: Math.max(340, Math.min(640, origW + (ev.clientX - startX))),
        h: Math.max(380, Math.min(window.innerHeight - 40, origH + (ev.clientY - startY))),
      });
    };
    const onUp = () => {
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    };
    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  };

  if (!open) return null;

  return (
    <div
      className="uxigl-glass-card"
      style={{
        position: 'fixed',
        left: pos.x,
        top: pos.y,
        width: size.w,
        height: size.h,
        maxHeight: 'calc(100vh - 24px)',
        zIndex: 120,
        display: 'flex',
        flexDirection: 'column',
        borderRadius: 20,
        boxShadow: '0 24px 60px rgba(0, 0, 0, 0.28)',
        overflow: 'hidden',
      }}
    >
      {/* Draggable Header */}
      <div
        onMouseDown={startDrag}
        style={{
          padding: '14px 16px 12px',
          borderBottom: '1px solid var(--uxigl-border)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: 10,
          cursor: 'grab',
          userSelect: 'none',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
          <div
            className="uxigl-subcard"
            style={{
              width: 34,
              height: 34,
              borderRadius: 10,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <Palette size={17} color="var(--uxu-accent, #38bdf8)" />
          </div>
          <div>
            <span style={{ fontWeight: 700, fontSize: 14 }}>Tema Ayarları</span>
          </div>
        </div>

        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <button
            type="button"
            className="uxigl-btn"
            style={{ width: 30, height: 30, padding: 0, borderRadius: 999 }}
            onClick={onReset}
            title="Sıfırla"
          >
            <RotateCcw size={13} />
          </button>
          <button
            type="button"
            className="uxigl-btn"
            style={{ width: 30, height: 30, padding: 0, borderRadius: 999 }}
            onClick={onClose}
            title="Kapat"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Scrollable Body */}
      <div style={{ flex: 1, overflowY: 'auto', padding: 16, display: 'flex', flexDirection: 'column', gap: 16 }}>
        {/* Görünüm Modu */}
        <section style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 700, fontSize: 12 }}>Görünüm</span>
            <span className="uxigl-pill" style={{ fontSize: 10, padding: '2px 8px' }}>
              {resolvedMode === 'dark' ? 'Karanlık' : 'Aydınlık'}
            </span>
          </div>
          <div
            className="uxigl-subcard"
            style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1fr', gap: 4, padding: 4, borderRadius: 12 }}
          >
            {(
              [
                { id: 'light', label: 'Aydınlık', icon: <Sun size={13} /> },
                { id: 'dark', label: 'Karanlık', icon: <Moon size={13} /> },
                { id: 'system', label: 'Sistem', icon: <Monitor size={13} /> },
              ] as const
            ).map((opt) => {
              const active = prefs.mode === opt.id;
              return (
                <button
                  key={opt.id}
                  type="button"
                  onClick={() => onChangePrefs({ ...prefs, mode: opt.id })}
                  className={active ? 'uxigl-btn' : ''}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    gap: 6,
                    padding: '6px 8px',
                    borderRadius: 9,
                    border: active ? '1px solid var(--uxigl-border)' : '1px solid transparent',
                    background: active ? 'var(--uxigl-btn-bg)' : 'transparent',
                    color: 'var(--uxigl-text)',
                    fontWeight: active ? 700 : 500,
                    fontSize: 12,
                    cursor: 'pointer',
                  }}
                >
                  {opt.icon}
                  <span>{opt.label}</span>
                </button>
              );
            })}
          </div>
        </section>

        {/* Yazı Boyutu */}
        <section style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 600, fontSize: 12 }}>Yazı Boyutu</span>
            <span className="uxigl-pill" style={{ fontSize: 11, padding: '2px 8px' }}>
              {prefs.fontSize}px
            </span>
          </div>
          <input
            type="range"
            className="uxigl-range"
            min={11}
            max={18}
            step={1}
            value={prefs.fontSize}
            onChange={(e) => onChangePrefs({ ...prefs, fontSize: Number(e.target.value) })}
            style={{ '--range-pct': `${((prefs.fontSize - 11) / (18 - 11)) * 100}%` } as React.CSSProperties}
          />
          <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
            {[11, 12, 13, 15, 16, 18].map((sz) => (
              <button
                key={sz}
                type="button"
                className={`uxigl-btn ${prefs.fontSize === sz ? 'uxigl-btn-primary' : ''}`}
                style={{ padding: '4px 10px', fontSize: 11, borderRadius: 999 }}
                onClick={() => onChangePrefs({ ...prefs, fontSize: sz })}
              >
                {sz}px
              </button>
            ))}
          </div>
        </section>

        {/* Yazı Tipi */}
        <section style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 600, fontSize: 12 }}>Yazı Tipi</span>
          </div>
          <select
            className="uxigl-select"
            value={prefs.fontFamily}
            onChange={(e) => onChangePrefs({ ...prefs, fontFamily: e.target.value as FontFamilyKey })}
          >
            <option value="system">Modern Sistem</option>
            <option value="readable">Klasik Okunaklı</option>
            <option value="mono">Eş Aralıklı (Monospace)</option>
          </select>
        </section>

        {/* Kenar Çubuğu */}
        <section style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 600, fontSize: 12 }}>Kenar Çubuğu</span>
            <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
              <button
                type="button"
                className={`uxigl-btn ${prefs.sidebarOpen ? 'uxigl-btn-primary' : ''}`}
                style={{ padding: '2px 8px', fontSize: 11, borderRadius: 999 }}
                onClick={() => onChangePrefs({ ...prefs, sidebarOpen: !prefs.sidebarOpen })}
              >
                {prefs.sidebarOpen ? 'Açık' : 'Gizli'}
              </button>
              <span className="uxigl-pill" style={{ fontSize: 11, padding: '2px 8px' }}>
                {sidebarWidth}px
              </span>
            </div>
          </div>
          <input
            type="range"
            className="uxigl-range"
            min={160}
            max={340}
            step={5}
            value={sidebarWidth}
            onChange={(e) => onChangeSidebarWidth(Number(e.target.value))}
            style={{ '--range-pct': `${((sidebarWidth - 160) / (340 - 160)) * 100}%` } as React.CSSProperties}
          />
          <div style={{ display: 'flex', gap: 6, flexWrap: 'wrap' }}>
            {[180, 220, 256, 300].map((w) => (
              <button
                key={w}
                type="button"
                className={`uxigl-btn ${sidebarWidth === w ? 'uxigl-btn-primary' : ''}`}
                style={{ padding: '4px 10px', fontSize: 11, borderRadius: 999 }}
                onClick={() => onChangeSidebarWidth(w)}
              >
                {w}px
              </button>
            ))}
          </div>
        </section>

        {/* Köşe Yuvarlaklığı */}
        <section style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <span style={{ fontWeight: 600, fontSize: 12 }}>Köşe Yuvarlaklığı</span>
            <span className="uxigl-pill" style={{ fontSize: 11, padding: '2px 8px' }}>
              {prefs.radius}px
            </span>
          </div>
          <input
            type="range"
            className="uxigl-range"
            min={4}
            max={20}
            step={2}
            value={prefs.radius}
            onChange={(e) => onChangePrefs({ ...prefs, radius: Number(e.target.value) })}
            style={{ '--range-pct': `${((prefs.radius - 4) / (20 - 4)) * 100}%` } as React.CSSProperties}
          />
        </section>

        {/* Renk Paleti */}
        <section style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <span style={{ fontWeight: 700, fontSize: 12 }}>Renk Paleti</span>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
            {(Object.keys(SEED_PALETTES) as SeedPaletteKey[]).map((key) => {
              const p = SEED_PALETTES[key];
              const selected = prefs.palette === key;
              return (
                <button
                  key={key}
                  type="button"
                  onClick={() => onChangePrefs({ ...prefs, palette: key })}
                  className="uxigl-subcard"
                  style={{
                    padding: '10px 11px',
                    textAlign: 'left',
                    cursor: 'pointer',
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 4,
                    border: selected ? `1.5px solid ${p.accent}` : '1px solid var(--uxigl-border)',
                    background: selected ? p.accentSoft : 'var(--uxigl-surface-sub)',
                    color: 'var(--uxigl-text)',
                  }}
                >
                  <div style={{ display: 'flex', gap: 5, marginBottom: 2 }}>
                    {p.dots.map((d, i) => (
                      <span
                        key={i}
                        style={{
                          width: 14,
                          height: 14,
                          borderRadius: 999,
                          background: d,
                          border: '1px solid rgba(0,0,0,0.15)',
                        }}
                      />
                    ))}
                  </div>
                  <span style={{ fontWeight: 700, fontSize: 12 }}>{p.name}</span>
                  <span style={{ fontSize: 10, color: 'var(--uxigl-text-muted)' }}>{p.subtitle}</span>
                </button>
              );
            })}
          </div>
        </section>
      </div>

      {/* Footer + Bottom-Right Resize Grip */}
      <div
        style={{
          padding: '10px 16px',
          borderTop: '1px solid var(--uxigl-border)',
          display: 'flex',
          alignItems: 'center',
          gap: 8,
          position: 'relative',
        }}
      >
        <button
          type="button"
          className="uxigl-btn"
          style={{ flex: 1, borderRadius: 999 }}
          onClick={onReset}
        >
          <RotateCcw size={13} />
          <span>Varsayılana Sıfırla</span>
        </button>
        <div
          onMouseDown={startResize}
          title="Boyutlandır"
          style={{
            position: 'absolute',
            right: 4,
            bottom: 4,
            width: 16,
            height: 16,
            cursor: 'nwse-resize',
            display: 'flex',
            alignItems: 'flex-end',
            justifyContent: 'flex-end',
            opacity: 0.5,
          }}
        >
          <svg width="10" height="10" viewBox="0 0 10 10">
            <path d="M9 1L1 9M9 5L5 9" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
          </svg>
        </div>
      </div>
    </div>
  );
};

