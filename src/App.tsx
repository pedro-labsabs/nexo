import React, { useState, useEffect, useMemo, useRef } from 'react';
import { Snapshot, Repository, Relationship, PotentialAssessment, UserAnnotations } from './shared/types.js';
import { validateSnapshot } from './shared/validation.js';
import { generateMarkdownReport } from './shared/report.js';
import demoSnapshotRaw from './demo/demo-snapshot.json';
import { Header } from './components/Header.js';
import { OverviewMetrics } from './components/OverviewMetrics.js';
import { FiltersBar, FilterState } from './components/FiltersBar.js';
import { RepositoryTable } from './components/RepositoryTable.js';
import { ProjectGraph } from './components/ProjectGraph.js';
import { RepositoryDetailDrawer } from './components/RepositoryDetailDrawer.js';
import { AlertCircle, AlertTriangle, CheckCircle2 } from 'lucide-react';

export const App: React.FC = () => {
  const [snapshot, setSnapshot] = useState<Snapshot>(demoSnapshotRaw as unknown as Snapshot);
  const [theme, setTheme] = useState<'light' | 'dark'>('dark');
  const [selectedRepo, setSelectedRepo] = useState<Repository | null>(null);
  const [activeTab, setActiveTab] = useState<'table' | 'graph'>('table');
  const [importNotification, setImportNotification] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);

  // Manage theme class on HTML root
  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [theme]);

  // Filter state
  const [filters, setFilters] = useState<FilterState>({
    search: '',
    organization: 'all',
    visibility: 'all',
    language: 'all',
    affiliation: 'all',
    sortBy: 'pushed',
    sortOrder: 'desc',
  });

  // Extract available languages
  const availableLanguages = useMemo(() => {
    const langs = new Set<string>();
    for (const r of snapshot.repositories) {
      if (r.primaryLanguage) langs.add(r.primaryLanguage);
    }
    return Array.from(langs).sort();
  }, [snapshot.repositories]);

  // Filtered and sorted repositories
  const filteredRepositories = useMemo(() => {
    return snapshot.repositories
      .filter((repo) => {
        // Search
        if (filters.search) {
          const term = filters.search.toLowerCase();
          const matchName = repo.fullName.toLowerCase().includes(term);
          const matchDesc = repo.description?.toLowerCase().includes(term);
          if (!matchName && !matchDesc) return false;
        }

        // Org
        if (filters.organization !== 'all') {
          if (repo.owner.login.toLowerCase() !== filters.organization.toLowerCase()) {
            return false;
          }
        }

        // Visibility
        if (filters.visibility === 'public' && repo.isPrivate) return false;
        if (filters.visibility === 'private' && !repo.isPrivate) return false;

        // Language
        if (filters.language !== 'all') {
          if (repo.primaryLanguage !== filters.language) return false;
        }

        return true;
      })
      .sort((a, b) => {
        let cmp = 0;
        if (filters.sortBy === 'pushed') {
          cmp = new Date(a.pushedAt).getTime() - new Date(b.pushedAt).getTime();
        } else if (filters.sortBy === 'stars') {
          cmp = a.stargazersCount - b.stargazersCount;
        } else if (filters.sortBy === 'size') {
          cmp = a.sizeKb - b.sizeKb;
        } else if (filters.sortBy === 'score') {
          cmp = (a.indicators?.overallQualityScore || 0) - (b.indicators?.overallQualityScore || 0);
        } else if (filters.sortBy === 'name') {
          cmp = a.name.localeCompare(b.name);
        }
        return filters.sortOrder === 'asc' ? cmp : -cmp;
      });
  }, [snapshot.repositories, filters]);

  // Handler for adding manual relationships
  const handleAddManualRelationship = (newRel: Relationship) => {
    setSnapshot((prev) => {
      const currentManual = prev.userAnnotations?.manualRelationships || [];
      const updatedUserAnnotations: UserAnnotations = {
        schemaVersion: '1.0',
        exportedAt: new Date().toISOString(),
        manualRelationships: [...currentManual, newRel],
        potentialAssessments: prev.userAnnotations?.potentialAssessments || {},
      };

      return {
        ...prev,
        relationships: [...prev.relationships, newRel],
        userAnnotations: updatedUserAnnotations,
      };
    });
  };

  // Handler for saving potential assessment
  const handleSavePotentialAssessment = (assessment: PotentialAssessment) => {
    setSnapshot((prev) => {
      const currentAssessments = prev.userAnnotations?.potentialAssessments || {};
      const updatedUserAnnotations: UserAnnotations = {
        schemaVersion: '1.0',
        exportedAt: new Date().toISOString(),
        manualRelationships: prev.userAnnotations?.manualRelationships || [],
        potentialAssessments: {
          ...currentAssessments,
          [String(assessment.repositoryId)]: assessment,
        },
      };

      return {
        ...prev,
        userAnnotations: updatedUserAnnotations,
      };
    });
  };

  // Export Markdown Report
  const handleExportReport = () => {
    const md = generateMarkdownReport(snapshot);
    const blob = new Blob([md], { type: 'text/markdown;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `nexo-relatorio-${snapshot.metadata.authenticatedUser}-${Date.now()}.md`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Export JSON Snapshot
  const handleExportSnapshot = () => {
    const json = JSON.stringify(snapshot, null, 2);
    const blob = new Blob([json], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `nexo-snapshot-${snapshot.metadata.authenticatedUser}-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // File import handler
  const handleFileImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (evt) => {
      try {
        const parsed = JSON.parse(evt.target?.result as string);
        const validation = validateSnapshot(parsed);
        if (validation.success) {
          setSnapshot(validation.data);
          setImportNotification({
            type: 'success',
            message: `Snapshot importado com sucesso! (${validation.data.repositories.length} repositórios).`,
          });
        } else {
          setImportNotification({
            type: 'error',
            message: `Falha na validação do snapshot: ${validation.errors.slice(0, 3).join('; ')}`,
          });
        }
      } catch (err: any) {
        setImportNotification({
          type: 'error',
          message: `Arquivo JSON inválido ou corrompido: ${err?.message || err}`,
        });
      }
    };
    reader.readAsText(file);
    e.target.value = '';
  };

  const resetToDemo = () => {
    setSnapshot(demoSnapshotRaw as unknown as Snapshot);
    setImportNotification({
      type: 'success',
      message: 'Base de dados restaurada para a demonstração sintética.',
    });
  };

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-slate-950 text-slate-900 dark:text-slate-100 flex flex-col font-sans transition-colors">
      <Header
        metadata={snapshot.metadata}
        theme={theme}
        onToggleTheme={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
        onResetToDemo={resetToDemo}
        onExportSnapshot={handleExportSnapshot}
        onImportClick={() => fileInputRef.current?.click()}
        onExportReport={handleExportReport}
      />

      <input
        type="file"
        ref={fileInputRef}
        onChange={handleFileImport}
        accept=".json"
        className="hidden"
      />

      <main className="flex-1 max-w-7xl w-full mx-auto p-4 sm:p-6">
        {/* Notification Banner */}
        {importNotification && (
          <div
            className={`mb-4 p-3 rounded-xl border flex items-center justify-between text-xs transition-all ${
              importNotification.type === 'success'
                ? 'bg-emerald-50 dark:bg-emerald-950/40 border-emerald-200 dark:border-emerald-800 text-emerald-800 dark:text-emerald-300'
                : 'bg-rose-50 dark:bg-rose-950/40 border-rose-200 dark:border-rose-800 text-rose-800 dark:text-rose-300'
            }`}
          >
            <div className="flex items-center gap-2">
              {importNotification.type === 'success' ? (
                <CheckCircle2 className="w-4 h-4 shrink-0" />
              ) : (
                <AlertCircle className="w-4 h-4 shrink-0" />
              )}
              <span>{importNotification.message}</span>
            </div>
            <button
              onClick={() => setImportNotification(null)}
              className="font-bold ml-3 text-slate-500 hover:text-slate-800 dark:hover:text-slate-200"
            >
              ✕
            </button>
          </div>
        )}

        {/* Sync Warnings Banner if any */}
        {snapshot.metadata.syncWarnings.length > 0 && (
          <div className="mb-4 p-3 bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-800 rounded-xl text-xs text-amber-800 dark:text-amber-300 flex items-start gap-2">
            <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5 text-amber-600 dark:text-amber-400" />
            <div>
              <span className="font-bold">Avisos da Sincronização:</span>
              <ul className="list-disc pl-4 mt-0.5 space-y-0.5 text-[11px]">
                {snapshot.metadata.syncWarnings.map((w, idx) => (
                  <li key={idx}>{w}</li>
                ))}
              </ul>
            </div>
          </div>
        )}

        {/* Overview Metric Cards */}
        <OverviewMetrics
          repositories={snapshot.repositories}
          organizations={snapshot.organizations}
          relationships={snapshot.relationships}
        />

        {/* View Switcher and Filters */}
        <div className="flex items-center justify-between gap-4 mb-3">
          <div className="flex items-center bg-slate-200 dark:bg-slate-800 p-1 rounded-lg">
            <button
              onClick={() => setActiveTab('table')}
              className={`px-3 py-1.5 text-xs font-semibold rounded-md transition-colors ${
                activeTab === 'table'
                  ? 'bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 shadow-sm'
                  : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100'
              }`}
            >
              Tabela de Repositórios ({filteredRepositories.length})
            </button>
            <button
              onClick={() => setActiveTab('graph')}
              className={`px-3 py-1.5 text-xs font-semibold rounded-md transition-colors ${
                activeTab === 'graph'
                  ? 'bg-white dark:bg-slate-900 text-slate-900 dark:text-slate-100 shadow-sm'
                  : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-100'
              }`}
            >
              Mapa Interativo do Grafo
            </button>
          </div>
        </div>

        {/* Filters */}
        <FiltersBar
          filters={filters}
          onChange={setFilters}
          organizations={snapshot.organizations}
          availableLanguages={availableLanguages}
        />

        {/* Tab View: Table vs Graph */}
        {activeTab === 'table' ? (
          <RepositoryTable
            repositories={filteredRepositories}
            onSelectRepo={(r) => setSelectedRepo(r)}
            selectedRepoId={selectedRepo?.id}
          />
        ) : (
          <ProjectGraph
            repositories={filteredRepositories}
            organizations={snapshot.organizations}
            relationships={snapshot.relationships}
            onSelectNode={(r) => setSelectedRepo(r)}
            onAddManualRelationship={handleAddManualRelationship}
            theme={theme}
          />
        )}
      </main>

      {/* Drawer */}
      <RepositoryDetailDrawer
        repo={selectedRepo}
        onClose={() => setSelectedRepo(null)}
        potentialAssessment={
          selectedRepo
            ? snapshot.userAnnotations?.potentialAssessments?.[String(selectedRepo.id)]
            : undefined
        }
        onSavePotentialAssessment={handleSavePotentialAssessment}
        relationships={snapshot.relationships}
        allRepositories={snapshot.repositories}
      />
    </div>
  );
};
