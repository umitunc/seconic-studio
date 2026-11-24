import React from 'react';
import { AlertTriangle } from 'lucide-react';

interface ExceptionAlertBarProps {
  subStackerCount: number;
  onReviewClick: () => void;
}

export const ExceptionAlertBar: React.FC<ExceptionAlertBarProps> = ({
  subStackerCount,
  onReviewClick,
}) => {
  if (subStackerCount === 0) return null;

  return (
    <div
      className="uxigl-glass-card"
      style={{
        padding: '10px 16px',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        background: 'rgba(245, 158, 11, 0.14)',
        borderColor: 'rgba(245, 158, 11, 0.45)',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, fontSize: 13, color: '#b45309', fontWeight: 500 }}>
        <AlertTriangle size={16} color="#d97706" />
        <span>
          <strong>{subStackerCount} kağıt</strong> Alt Tepsi&apos;ye (Sub-Stacker) ayrıldı veya inceleme bekliyor.
        </span>
      </div>
      <button className="uxigl-btn" onClick={onReviewClick}>
        Şüpheli Kağıtları İncele (F2)
      </button>
    </div>
  );
};
