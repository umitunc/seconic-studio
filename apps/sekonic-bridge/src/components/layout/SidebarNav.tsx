import React from 'react';
import { ScanLine, History, FileSpreadsheet, Settings, Sun, Moon } from 'lucide-react';
import { SekonicStudioLogo } from './SekonicLogo';

export type NavTab = 'scan' | 'history' | 'forms' | 'settings';

interface SidebarNavProps {
  activeTab: NavTab;
  onSelectTab: (tab: NavTab) => void;
  theme: 'light' | 'dark';
  onToggleTheme: () => void;
  width?: number;
}

const NAV_ITEMS: Array<{ id: NavTab; label: string; icon: React.ReactNode }> = [
  { id: 'scan', label: 'Canlı Okuma', icon: <ScanLine size={17} /> },
  { id: 'history', label: 'Geçmiş Okumalar', icon: <History size={17} /> },
  { id: 'forms', label: 'Form Şablonları', icon: <FileSpreadsheet size={17} /> },
  { id: 'settings', label: 'Cihaz & Ayarlar', icon: <Settings size={17} /> },
];

export const SidebarNav: React.FC<SidebarNavProps> = ({
  activeTab,
  onSelectTab,
  theme,
  onToggleTheme,
  width = 220,
}) => {
  return (
    <aside
      className="uxigl-glass-card"
      style={{
        width,
        flexShrink: 0,
        padding: 14,
        display: 'flex',
        flexDirection: 'column',
        justifyContent: 'space-between',
        gap: 16,
      }}
    >
      <div style={{ display: 'flex', flexDirection: 'column', gap: 18 }}>
        <div style={{ padding: '6px 2px' }}>
          <SekonicStudioLogo width="100%" />
        </div>

        <nav style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          {NAV_ITEMS.map((item) => {
            const active = activeTab === item.id;
            return (
              <button
                key={item.id}
                onClick={() => onSelectTab(item.id)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                  padding: '10px 12px',
                  borderRadius: 9,
                  border: active
                    ? '1px solid var(--uxigl-border)'
                    : '1px solid transparent',
                  background: active ? 'var(--uxigl-accent-soft)' : 'transparent',
                  color: 'var(--uxigl-text)',
                  fontWeight: active ? 700 : 500,
                  fontSize: 13,
                  cursor: 'pointer',
                  textAlign: 'left',
                  transition: 'background 0.15s ease',
                }}
              >
                {item.icon}
                <span>{item.label}</span>
              </button>
            );
          })}
        </nav>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        <button
          className="uxigl-btn"
          onClick={onToggleTheme}
          style={{ width: '100%', justifyContent: 'flex-start' }}
        >
          {theme === 'dark' ? <Sun size={15} /> : <Moon size={15} />}
          <span>{theme === 'dark' ? 'Açık Tema' : 'Koyu Tema'}</span>
        </button>
        <div style={{ fontSize: 11, color: 'var(--uxigl-text-muted)', paddingLeft: 6 }}>
          v0.1.2 • Köprü: :17441
        </div>
      </div>
    </aside>
  );
};
