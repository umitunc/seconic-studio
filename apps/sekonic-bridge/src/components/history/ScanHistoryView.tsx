import React, { useMemo } from 'react';
import { SheetGL, appleDark, appleLight, trTR, type ColumnDef } from 'sheetgl';

interface ScanHistoryViewProps {
  theme: 'light' | 'dark';
}

const SAMPLE_SESSIONS = [
  {
    id: 'ses_2026_01',
    date: '16.11.2026 14:20',
    examName: '2026 LGS Deneme-3 (Sayısal)',
    device: 'SR-6500 (USB)',
    totalSheets: 1420,
    mainStacker: 1402,
    subStacker: 18,
    syncStatus: 'Senkronize (%100)',
  },
  {
    id: 'ses_2026_02',
    date: '15.11.2026 09:45',
    examName: '2026 TYT Genel Tarama-2',
    device: 'SR-3500 (COM3)',
    totalSheets: 860,
    mainStacker: 851,
    subStacker: 9,
    syncStatus: 'Senkronize (%100)',
  },
  {
    id: 'ses_2026_03',
    date: '12.11.2026 16:10',
    examName: '8. Sınıf Matematik Kazanım Testi',
    device: 'SR-55D CIS (USB)',
    totalSheets: 315,
    mainStacker: 312,
    subStacker: 3,
    syncStatus: 'Yerel SQLite Yedekli',
  },
];

export const ScanHistoryView: React.FC<ScanHistoryViewProps> = ({ theme }) => {
  const columns = useMemo<ColumnDef[]>(
    () => [
      { id: 'date', title: 'Tarih / Saat', type: 'text', width: 145 },
      { id: 'examName', title: 'Sınav / Oturum Adı', type: 'text', width: 250 },
      { id: 'device', title: 'Okuyucu', type: 'text', width: 150 },
      { id: 'totalSheets', title: 'Toplam Kağıt', type: 'number', width: 110 },
      { id: 'mainStacker', title: 'Ana Tepsi', type: 'number', width: 100 },
      { id: 'subStacker', title: 'Alt Tepsi', type: 'number', width: 95 },
      { id: 'syncStatus', title: 'Ölçme App Durumu', type: 'text', width: 170 },
    ],
    [],
  );

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12, flex: 1 }}>
      <div className="uxigl-glass-card" style={{ padding: '14px 18px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <div style={{ fontWeight: 700, fontSize: 15 }}>Geçmiş Okuma Oturumları</div>
          <div style={{ fontSize: 12, color: 'var(--uxigl-text-muted)' }}>
            Tüm oturumlar ham 0–255 matrisleriyle yerel SQLite WAL veritabanında saklanır.
          </div>
        </div>
      </div>

      <div className="uxigl-glass-card" style={{ flex: 1, minHeight: 420, padding: 8, overflow: 'hidden' }}>
        <SheetGL
          dataSource={SAMPLE_SESSIONS}
          columns={columns}
          theme={theme === 'dark' ? appleDark : appleLight}
          locale={trTR}
          selectionMode="single"
          showFormulaBar={false}
          toolbar={{
            visible: true,
            showBrand: false,
            showAddRow: false,
            showDuplicateRow: false,
            showDeleteRow: false,
            allowDisplayModeSwitch: false,
            showSelectionModeSwitch: false,
            showPermissionsSwitch: false,
            showColumnSettings: false,
            showLocaleSwitch: false,
            showThemeSwitch: false,
            showGroupBy: false,
            showUndoRedo: false,
            showSearch: true,
            showClearFilters: true,
            showExportCsv: false,
            showExportExcel: false,
          }}
          summaryRow={{ showTotalCount: true, showSelectionStats: false, showColumnSummaries: false }}
        />
      </div>
    </div>
  );
};
