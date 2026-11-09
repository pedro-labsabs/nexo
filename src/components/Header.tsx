import React from 'react';
import { Sun, Moon, GitBranch, RefreshCw, Download, Upload, FileText, BookmarkPlus, BookmarkCheck } from 'lucide-react';
import { SnapshotMetadata } from '../shared/types.js';

interface HeaderProps {
  metadata: SnapshotMetadata;
  theme: 'light' | 'dark';
  onToggleTheme: () => void;
  onResetToDemo: () => void;
  onExportSnapshot: () => void;
  onImportClick: () => void;
  onExportAnnotations: () => void;
  onImportAnnotationsClick: () => void;
  onExportReport: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  metadata,
  theme,
  onToggleTheme,
  onResetToDemo,
  onExportSnapshot,
  onImportClick,
  onExportAnnotations,
  onImportAnnotationsClick,
  onExportReport,
}) => {
  return (
    <header className="border-b border-slate-200 dark:border-slate-800 bg-white dark:bg-slate-900 sticky top-0 z-30 px-4 py-3 shadow-sm transition-colors">
      <div className="max-w-7xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-indigo-600 rounded-lg text-white shadow-md shadow-indigo-500/20">
            <GitBranch className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h1 className="text-xl font-bold tracking-tight text-slate-900 dark:text-slate-100">NEXO</h1>
              <span className="px-2 py-0.5 text-xs font-semibold bg-indigo-100 text-indigo-800 dark:bg-indigo-950 dark:text-indigo-300 rounded-full">
                Observatório GitHub
              </span>
            </div>
            <p className="text-xs text-slate-500 dark:text-slate-400">
              Usuário: <span className="font-medium text-slate-700 dark:text-slate-200">@{metadata.authenticatedUser}</span> • Referência: {new Date(metadata.referenceDate).toLocaleDateString('pt-BR')}
            </p>
          </div>
        </div>

        <div className="flex items-center flex-wrap gap-2">
          <button
            onClick={onResetToDemo}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-slate-700 dark:text-slate-200 bg-slate-100 dark:bg-slate-800 hover:bg-slate-200 dark:hover:bg-slate-700 rounded-md transition-colors"
            title="Recarregar base de demonstração sintética (preserva anotações reais no navegador)"
          >
            <RefreshCw className="w-3.5 h-3.5" />
            Demonstração
          </button>

          {/* Explicit Export and Import Annotations */}
          <button
            onClick={onImportAnnotationsClick}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-purple-700 dark:text-purple-300 bg-purple-50 dark:bg-purple-950/50 hover:bg-purple-100 dark:hover:bg-purple-900/60 border border-purple-200 dark:border-purple-800 rounded-md transition-colors"
            title="Importar e mesclar arquivo JSON exclusivo de anotações e relações manuais"
          >
            <BookmarkPlus className="w-3.5 h-3.5" />
            Importar Anotações
          </button>

          <button
            onClick={onExportAnnotations}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-purple-700 dark:text-purple-300 bg-purple-50 dark:bg-purple-950/50 hover:bg-purple-100 dark:hover:bg-purple-900/60 border border-purple-200 dark:border-purple-800 rounded-md transition-colors"
            title="Exportar arquivo JSON contendo apenas anotações e relações manuais (backup seguro sem metadados privados)"
          >
            <BookmarkCheck className="w-3.5 h-3.5" />
            Exportar Anotações
          </button>

          <button
            onClick={onImportClick}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-slate-700 dark:text-slate-200 bg-slate-100 dark:bg-slate-800 hover:bg-slate-200 dark:hover:bg-slate-700 rounded-md transition-colors"
            title="Importar arquivo de snapshot JSON completo do GitHub"
          >
            <Upload className="w-3.5 h-3.5" />
            Importar Snapshot
          </button>

          <button
            onClick={onExportSnapshot}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-slate-700 dark:text-slate-200 bg-slate-100 dark:bg-slate-800 hover:bg-slate-200 dark:hover:bg-slate-700 rounded-md transition-colors"
            title="Exportar snapshot JSON completo com anotações atuais"
          >
            <Download className="w-3.5 h-3.5" />
            Exportar JSON
          </button>

          <button
            onClick={onExportReport}
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-md transition-colors shadow-sm"
            title="Exportar relatório completo em Markdown"
          >
            <FileText className="w-3.5 h-3.5" />
            Relatório MD
          </button>

          <button
            onClick={onToggleTheme}
            className="p-1.5 text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-md transition-colors"
            aria-label="Alternar tema claro/escuro"
          >
            {theme === 'dark' ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
          </button>
        </div>
      </div>
    </header>
  );
};
