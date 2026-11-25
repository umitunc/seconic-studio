import React from 'react';
import { X } from 'lucide-react';
import { ScannedRow } from '../../types/scanner';

interface AnswerDetailsPanelProps {
  row: ScannedRow | null;
  onClose: () => void;
}

export const AnswerDetailsPanel: React.FC<AnswerDetailsPanelProps> = ({ row, onClose }) => {
  if (!row) return null;

  const details =
    row.questionDetails && row.questionDetails.length > 0
      ? row.questionDetails
      : Array.from(row.answersSummary).map((ch, idx) => ({
          q: idx + 1,
          subject: undefined as string | undefined,
          answer: ch === ' ' ? '-' : ch,
          state: (ch === '*' ? 'multi' : ch === ' ' ? 'blank' : 'ok') as 'ok' | 'erased' | 'multi' | 'blank',
          density: ch === ' ' ? 0 : 205,
          erasedOption: undefined as string | undefined,
        }));

  return (
    <aside
      className="uxigl-glass-card"
      style={{
        width: 240,
        flexShrink: 0,
        padding: 12,
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
        maxHeight: 'calc(100vh - 110px)',
      }}
    >
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <div style={{ fontWeight: 700, fontSize: 13 }}>Cevap Detayı (#{row.seqNo})</div>
          <div style={{ fontSize: 11, color: 'var(--uxigl-text-muted)' }}>
            {row.studentId} • Kit: {row.booklet}
          </div>
        </div>
        <button
          className="uxigl-btn"
          style={{ padding: '4px 6px' }}
          onClick={onClose}
          title="Paneli Kapat"
        >
          <X size={13} />
        </button>
      </div>

      <div style={{ flex: 1, overflowY: 'auto', paddingRight: 8, display: 'flex', flexDirection: 'column', gap: 4 }}>
        {details.map((item, idx) => (
          <div
            key={`${item.subject ?? 'q'}_${item.q}_${idx}`}
            className="uxigl-subcard"
            style={{
              padding: '5px 9px',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              fontSize: 12,
              gap: 6,
            }}
          >
            <span style={{ color: 'var(--uxigl-text-muted)', fontWeight: 600, minWidth: 32, fontSize: 11 }}>
              {item.subject ? `${item.subject.slice(0, 4)} ` : ''}{item.q}.
            </span>
            <span style={{ fontWeight: 700, flex: 1 }}>
              {item.answer}
              {item.state === 'erased' && item.erasedOption ? (
                <span style={{ fontSize: 10, color: '#f59e0b', marginLeft: 6 }}>
                  (Silgi: {item.erasedOption})
                </span>
              ) : null}
              {item.state === 'multi' ? (
                <span style={{ fontSize: 10, color: '#ef4444', marginLeft: 6 }}>(Çift)</span>
              ) : null}
            </span>
            <span style={{ fontSize: 10, color: 'var(--uxigl-text-muted)' }}>
              {item.density}/255
            </span>
          </div>
        ))}
      </div>
    </aside>
  );
};
