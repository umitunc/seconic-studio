import React from 'react';
import { ThresholdPolicy } from '../../types/scanner';

interface FieldInspectorPanelProps {
  policy: ThresholdPolicy;
  onChange: (p: ThresholdPolicy) => void;
}

export const FieldInspectorPanel: React.FC<FieldInspectorPanelProps> = ({ policy, onChange }) => {
  return (
    <div className="uxigl-glass-card" style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 14 }}>
      <div style={{ fontWeight: 700, fontSize: 14 }}>Analog Eşik Politikası (0–255)</div>

      <label style={{ display: 'flex', flexDirection: 'column', gap: 4, fontSize: 12 }}>
        <span>Minimum İşaret Koyuluğu: {policy.minMarkDensity}</span>
        <input
          type="range"
          min={20}
          max={120}
          value={policy.minMarkDensity}
          onChange={(e) => onChange({ ...policy, minMarkDensity: Number(e.target.value) })}
        />
      </label>

      <label style={{ display: 'flex', flexDirection: 'column', gap: 4, fontSize: 12 }}>
        <span>Güçlü İşaret Eşiği: {policy.strongMarkDensity}</span>
        <input
          type="range"
          min={80}
          max={200}
          value={policy.strongMarkDensity}
          onChange={(e) => onChange({ ...policy, strongMarkDensity: Number(e.target.value) })}
        />
      </label>

      <label style={{ display: 'flex', flexDirection: 'column', gap: 4, fontSize: 12 }}>
        <span>Silgi Kurtarma Oranı: {policy.erasureRatio.toFixed(2)}</span>
        <input
          type="range"
          min={0.2}
          max={0.7}
          step={0.02}
          value={policy.erasureRatio}
          onChange={(e) => onChange({ ...policy, erasureRatio: Number(e.target.value) })}
        />
      </label>

      <label style={{ display: 'flex', flexDirection: 'column', gap: 4, fontSize: 12 }}>
        <span>İnceleme Güven Eşiği: {policy.confidenceFloor.toFixed(2)}</span>
        <input
          type="range"
          min={0.4}
          max={0.95}
          step={0.05}
          value={policy.confidenceFloor}
          onChange={(e) => onChange({ ...policy, confidenceFloor: Number(e.target.value) })}
        />
      </label>
    </div>
  );
};
