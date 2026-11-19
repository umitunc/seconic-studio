import React, { useMemo } from 'react';
import { SheetGL, appleDark, appleLight, trTR, type ColumnDef } from 'sheetgl';
import { ScannedRow } from '../../types/scanner';

interface LiveScanGridProps {
  rows: ScannedRow[];
  theme: 'light' | 'dark';
  onSelectRow: (row: ScannedRow) => void;
  onCellEdit: (row: ScannedRow, columnId: string, value: unknown) => void;
}

export const LiveScanGrid: React.FC<LiveScanGridProps> = ({
  rows,
  theme,
  onSelectRow,
  onCellEdit,
}) => {
  const columns = useMemo<ColumnDef[]>(
    () => [
      { id: 'seqNo', title: '#Sıra', type: 'number', width: 70 },
      { id: 'studentId', title: 'Öğrenci No', type: 'text', width: 115 },
      { id: 'studentName', title: 'Ad Soyad', type: 'text', width: 165 },
      { id: 'booklet', title: 'Kitapçık', type: 'text', width: 80 },
      {
        id: 'stacker',
        title: 'Tepsi',
        type: 'select',
        width: 85,
        options: [
          { value: 'main', label: 'Ana', bg: '#e4e4e7', fg: '#18181b', darkBg: '#27272a', darkFg: '#f4f4f5' },
          { value: 'sub', label: 'Alt', bg: '#fee2e2', fg: '#b91c1c', darkBg: 'rgba(239,68,68,0.22)', darkFg: '#fca5a5' },
        ],
      },
      { id: 'confidence', title: 'Güven (%)', type: 'progress', width: 115 },
      {
        id: 'status',
        title: 'Durum',
        type: 'select',
        width: 145,
        options: [
          { value: 'Tamam', label: 'Tamam', bg: '#dcfce7', fg: '#15803d', darkBg: 'rgba(16,185,129,0.2)', darkFg: '#6ee7b7' },
          { value: 'Silgi Kurtarıldı', label: 'Silgi Kurtarıldı', bg: '#fef3c7', fg: '#b45309', darkBg: 'rgba(245,158,11,0.2)', darkFg: '#fcd34d' },
          { value: 'Çift İşaret', label: 'Çift İşaret', bg: '#ffedd5', fg: '#c2410c', darkBg: 'rgba(249,115,22,0.2)', darkFg: '#fdba74' },
          { value: 'Kayıtsız', label: 'Kayıtsız', bg: '#fee2e2', fg: '#b91c1c', darkBg: 'rgba(239,68,68,0.22)', darkFg: '#fca5a5' },
        ],
      },
      { id: 'answersSummary', title: 'Cevap Özeti', type: 'text', width: 200 },
    ],
    [],
  );

  return (
    <div className="uxigl-glass-card" style={{ flex: 1, minHeight: 380, overflow: 'hidden', padding: 8 }}>
      <SheetGL
        dataSource={rows}
        columns={columns}
        theme={theme === 'dark' ? appleDark : appleLight}
        locale={trTR}
        batchEditMode={true}
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
          showUndoRedo: true,
          showSearch: true,
          showClearFilters: true,
          showExportCsv: false,
          showExportExcel: false,
        }}
        summaryRow={{ showTotalCount: true, showSelectionStats: false, showColumnSummaries: false }}
        onSelectionChanged={(_keys, selected) => {
          if (selected.length > 0) {
            onSelectRow(selected[0] as ScannedRow);
          }
        }}
        onCellChange={(row, colId, val) => onCellEdit(row as ScannedRow, colId, val)}
      />
    </div>
  );
};
