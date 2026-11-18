import React from 'react';

interface KpiStripProps {
  total: number;
  mainCount: number;
  subCount: number;
  erasureCount: number;
}

export const KpiStrip: React.FC<KpiStripProps> = ({ total, mainCount, subCount, erasureCount }) => {
  const items = [
    { label: 'Toplam Okunan', value: total, color: '#38bdf8' },
    { label: 'Ana Tepsi (Geçerli)', value: mainCount, color: '#10b981' },
    { label: 'Alt Tepsi (Ayrılan)', value: subCount, color: '#ef4444' },
    { label: 'Silgi Kurtarılan', value: erasureCount, color: '#f59e0b' },
  ];

  return (
    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 10 }}>
      {items.map((item) => (
        <div key={item.label} className="uxigl-glass-card" style={{ padding: '10px 14px' }}>
          <div style={{ fontSize: 11, color: '#94a3b8', marginBottom: 4 }}>{item.label}</div>
          <div style={{ fontSize: 22, fontWeight: 700, color: item.color }}>{item.value}</div>
        </div>
      ))}
    </div>
  );
};
