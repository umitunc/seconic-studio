import React, { useEffect, useRef } from 'react';
import { renderSheetHeatmapCanvas } from '@kurum/sekonic-web-sdk';
import { ScannedRow } from '../../types/scanner';

interface SheetCanvasPreviewProps {
  row: ScannedRow | null;
  rows?: number;
  cols?: number;
  theme: 'light' | 'dark';
  onOpenReview: () => void;
}

export const SheetCanvasPreview: React.FC<SheetCanvasPreviewProps> = ({
  row,
  rows = 24,
  cols = 10,
  theme,
  onOpenReview,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    if (canvasRef.current && row) {
      renderSheetHeatmapCanvas(canvasRef.current, row.rawMatrixB64, rows, cols, 48, 115, theme);
    }
  }, [row, rows, cols, theme]);

  return (
    <div className="uxigl-glass-card" style={{ padding: 14, flex: 1, display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span style={{ fontWeight: 600, fontSize: 13 }}>0–255 Analog Form Isı Haritası</span>
        {row && (
          <button className="uxigl-btn" style={{ padding: '4px 10px', fontSize: 11 }} onClick={onOpenReview}>
            Yakınlaştır (F2)
          </button>
        )}
      </div>

      {row ? (
        <>
          <div style={{ fontSize: 11, color: '#94a3b8' }}>
            #{row.seqNo} • {row.studentId} ({row.studentName}) • Güven: %{row.confidence}
          </div>
          <canvas
            ref={canvasRef}
            width={280}
            height={320}
            style={{
              width: '100%',
              height: 'auto',
              borderRadius: 10,
              border: '1px solid rgba(148,163,184,0.2)',
            }}
          />
        </>
      ) : (
        <div style={{ flex: 1, display: 'grid', placeItems: 'center', color: '#64748b', fontSize: 12 }}>
          Önizlemek için listeden bir kağıt seçin
        </div>
      )}
    </div>
  );
};
