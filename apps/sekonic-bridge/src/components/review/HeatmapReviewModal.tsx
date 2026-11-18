import React, { useEffect, useRef, useState } from 'react';
import { ZoomIn, ZoomOut } from 'lucide-react';
import { renderSheetHeatmapCanvas } from '@kurum/sekonic-web-sdk';
import { ScannedRow } from '../../types/scanner';
import { DensityBarChart } from './DensityBarChart';

interface HeatmapReviewModalProps {
  row: ScannedRow | null;
  theme: 'light' | 'dark';
  onClose: () => void;
  onResolveAnswer: (rowId: number, option: string) => void;
  onRouteSubStacker: (rowId: number) => void;
}

export const HeatmapReviewModal: React.FC<HeatmapReviewModalProps> = ({
  row,
  theme,
  onClose,
  onResolveAnswer,
  onRouteSubStacker,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const canvasWrapRef = useRef<HTMLDivElement | null>(null);
  const [zoom, setZoom] = useState(1);

  useEffect(() => {
    const el = canvasWrapRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      const delta = e.deltaY < 0 ? 0.15 : -0.15;
      setZoom((z) => Math.min(2.5, Math.max(1, +(z + delta).toFixed(2))));
    };
    el.addEventListener('wheel', onWheel, { passive: false });
    return () => el.removeEventListener('wheel', onWheel);
  }, [row]);

  useEffect(() => {
    if (canvasRef.current && row) {
      renderSheetHeatmapCanvas(canvasRef.current, row.rawMatrixB64, 24, 10, 48, 115, theme);
    }
  }, [row, theme]);

  useEffect(() => {
    if (!row) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
      if (e.key === ' ' || e.key === 'Enter') {
        e.preventDefault();
        onResolveAnswer(row.id, row.flaggedQuestion?.selectedLabel ?? 'C');
      }
      if (['a', 'b', 'c', 'd', 'e'].includes(e.key.toLowerCase())) {
        onResolveAnswer(row.id, e.key.toUpperCase());
      }
      if (e.key.toLowerCase() === 's') {
        onRouteSubStacker(row.id);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [row, onClose, onResolveAnswer, onRouteSubStacker]);

  if (!row) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(2, 6, 23, 0.78)',
        backdropFilter: 'blur(8px)',
        display: 'grid',
        placeItems: 'center',
        zIndex: 50,
      }}
    >
      <div
        className="uxigl-glass-card"
        style={{ width: 860, maxWidth: '94vw', padding: 20, display: 'flex', flexDirection: 'column', gap: 16 }}
      >
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <div>
            <div style={{ fontWeight: 700, fontSize: 16 }}>
              Kağıt İnceleme #{row.seqNo} — {row.studentId} ({row.studentName})
            </div>
            <div style={{ fontSize: 12, color: '#94a3b8' }}>
              Kitapçık {row.booklet} • Güven: %{row.confidence} • Durum: {row.status}
            </div>
          </div>
          <button className="uxigl-btn" onClick={onClose}>
            Kapat (Esc)
          </button>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 18 }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
              <button className="uxigl-btn" style={{ padding: '4px 8px', fontSize: 11 }} onClick={() => setZoom((z) => Math.max(1, +(z - 0.25).toFixed(2)))}>
                <ZoomOut size={12} />
              </button>
              <button className="uxigl-btn" style={{ padding: '4px 8px', fontSize: 11 }} onClick={() => setZoom(1)}>
                %{Math.round(zoom * 100)}
              </button>
              <button className="uxigl-btn" style={{ padding: '4px 8px', fontSize: 11 }} onClick={() => setZoom((z) => Math.min(2.5, +(z + 0.25).toFixed(2)))}>
                <ZoomIn size={12} />
              </button>
            </div>
            <div ref={canvasWrapRef} style={{ overflow: 'auto', maxHeight: 380, borderRadius: 10, border: '1px solid var(--uxigl-border)' }}>
              <canvas
                ref={canvasRef}
                width={360}
                height={380}
                style={{ width: `${Math.round(zoom * 100)}%`, display: 'block' }}
              />
            </div>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'space-between', gap: 14 }}>
            <DensityBarChart
              densities={row.flaggedQuestion?.densities ?? [10, 14, 210, 12, 9]}
            />
            <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
              <div style={{ fontSize: 12, color: '#94a3b8' }}>Hızlı Klavye Onayı</div>
              <div style={{ display: 'flex', gap: 6 }}>
                {['A', 'B', 'C', 'D', 'E'].map((opt) => (
                  <button
                    key={opt}
                    className="uxigl-btn"
                    style={{ flex: 1 }}
                    onClick={() => onResolveAnswer(row.id, opt)}
                  >
                    [{opt}]
                  </button>
                ))}
              </div>
              <button
                className="uxigl-btn uxigl-btn-primary"
                onClick={() => onResolveAnswer(row.id, row.flaggedQuestion?.selectedLabel ?? 'C')}
              >
                [Space / Enter] Onayla ve Sonrakine Geç
              </button>
              <button
                className="uxigl-btn uxigl-btn-danger"
                onClick={() => onRouteSubStacker(row.id)}
              >
                [S] Alt Tepsiye (Sub-Stacker) Ayır
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
