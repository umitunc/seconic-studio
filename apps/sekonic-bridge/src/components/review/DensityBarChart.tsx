import React from 'react';

interface DensityBarChartProps {
  densities: number[];
  minThreshold?: number;
  strongThreshold?: number;
}

export const DensityBarChart: React.FC<DensityBarChartProps> = ({
  densities,
  minThreshold = 48,
  strongThreshold = 115,
}) => {
  const labels = ['A', 'B', 'C', 'D', 'E'];

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
      <div style={{ fontSize: 12, color: '#94a3b8' }}>0–255 Optik Koyuluk Dağılımı</div>
      {densities.map((val, idx) => {
        const pct = Math.min(100, Math.round((val / 255) * 100));
        const color =
          val >= strongThreshold ? '#10b981' : val >= minThreshold ? '#f59e0b' : '#475569';
        return (
          <div key={idx} style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 12 }}>
            <span style={{ width: 16, fontWeight: 700 }}>{labels[idx] ?? idx + 1}</span>
            <div style={{ flex: 1, height: 10, borderRadius: 999, background: '#1e293b', overflow: 'hidden' }}>
              <div style={{ width: `${pct}%`, height: '100%', background: color }} />
            </div>
            <span style={{ width: 32, textAlign: 'right', color: 'var(--uxigl-text-muted)', fontWeight: 600 }}>{val}</span>
          </div>
        );
      })}
    </div>
  );
};
