import React, { useState, useEffect, useMemo, useRef } from 'react';
import {
  Snapshot,
  Repository,
  Relationship,
  PotentialAssessment,
  UserAnnotations,
  RelationshipTombstone,
} from './shared/types.js';
import { validateSnapshot, validateAnnotations } from './shared/validation.js';
import { generateMarkdownReport } from './shared/report.js';
import { mergeUserAnnotations } from './shared/annotations.js';
import { loadScopedAnnotations, saveScopedAnnotations, clearScopedAnnotations } from './shared/storage.js';
import demoSnapshotRaw from './demo/demo-snapshot.json';
import { Header } from './components/Header.js';
import { OverviewMetrics } from './components/OverviewMetrics.js';
import { FiltersBar, FilterState } from './components/FiltersBar.js';
import { RepositoryTable } from './components/RepositoryTable.js';
import { ProjectGraph } from './components/ProjectGraph.js';
import { RepositoryDetailDrawer } from './components/RepositoryDetailDrawer.js';
import { AlertCircle, AlertTriangle, CheckCircle2, Trash2 } from 'lucide-react';

export const App: React.FC = () => {
  const [snapshot, setSnapshot] = useState<Snapshot>(() => {
    const initial = demoSnapshotRaw as unknown as Snapshot;
    // Check if there are user annotations already stored for demo
    const stored = loadScopedAnnotations(initial.metadata.authenticatedUser);
    if (stored) {
      const mergedAnno = mergeUserAnnotations(initial.userAnnotations || {
        schemaVersion: '1.0',
        exportedAt: new Date().toISOString(),
        authenticatedUser: initial.metadata.authenticatedUser,
        manualRelationships: [],
        deletedRelationships: [],
        potentialAssessments: {},
      }, stored);

      // Rebuild initial relationships including manual ones not tombstones
      const deletedIds = new Set(mergedAnno.deletedRelationships?.map((t) => t.id) || []);
      const activeManual = mergedAnno.manualRelationships.filter((r) => !deletedIds.has(r.id));
      const autoRels = initial.relationships.filter((r) => r.provenance === 'automatic');

      return {
        ...initial,
        relationships: [...autoRels, ...activeManual],
        userAnnotations: mergedAnno,
      };
    }
    return initial;
  });

  const [theme, setTheme] = useState<'light' | 'dark'>('dark');
  const [selectedRepo, setSelectedRepo] = useState<Repository | null>(null);
  const [activeTab, setActiveTab] = useState<'table' | 'graph'>('table');
  const [importNotification, setImportNotification] = useState<{ type: 'success' | 'error'; message: string } | null>(null);

  const fileInputRef = useRef<HTMLInputElement>(null);
  const annotationsFileInputRef = useRef<HTMLInputElement>(null);

  // Manage theme class on HTML root
  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [theme]);

  // Save annotations to localStorage whenever userAnnotations change
  useEffect(() => {
    if (snapshot.userAnnotations) {
      const saveResult = saveScopedAnnotations(snapshot.userAnnotations, snapshot.metadata.authenticatedUser);
      if (!saveResult.success) {
        setImportNotification({
          type: 'error',
          message: saveResult.error || 'Falha ao salvar anotações no navegador.',
        });
      }
    }
  }, [snapshot.userAnnotations, snapshot.metadata.authenticatedUser]);

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

  // Handler for adding a manual relationship
  const handleAddManualRelationship = (newRel: Relationship) => {
    setSnapshot((prev) => {
      const currentManual = prev.userAnnotations?.manualRelationships || [];
      const updatedUserAnnotations: UserAnnotations = {
        schemaVersion: '1.0',
        exportedAt: new Date().toISOString(),
        authenticatedUser: prev.metadata.authenticatedUser,
        manualRelationships: [...currentManual.filter((r) => r.id !== newRel.id), newRel],
        deletedRelationships: prev.userAnnotations?.deletedRelationships || [],
        potentialAssessments: prev.userAnnotations?.potentialAssessments || {},
      };

      const autoRels = prev.relationships.filter((r) => r.provenance === 'automatic');
      return {
        ...prev,
        relationships: [...autoRels, ...updatedUserAnnotations.manualRelationships],
        userAnnotations: updatedUserAnnotations,
      };
    });
  };

  // Handler for updating a manual relationship
  const handleUpdateManualRelationship = (updatedRel: Relationship) => {
    setSnapshot((prev) => {
      const currentManual = prev.userAnnotations?.manualRelationships || [];
      const updatedManual = currentManual.map((r) => (r.id === updatedRel.id ? updatedRel : r));

      const updatedUserAnnotations: UserAnnotations = {
        schemaVersion: '1.0',
        exportedAt: new Date().toISOString(),
        authenticatedUser: prev.metadata.authenticatedUser,
        manualRelationships: updatedManual,
        deletedRelationships: prev.userAnnotations?.deletedRelationships || [],
        potentialAssessments: prev.userAnnotations?.potentialAssessments || {},
      };

      const autoRels = prev.relationships.filter((r) => r.provenance === 'automatic');
      return {
        ...prev,
        relationships: [...autoRels, ...updatedManual],
        userAnnotations: updatedUserAnnotations,
      };
    });
  };

  // Handler for deleting a manual relationship with a tombstone
  const handleDeleteManualRelationship = (relId: string) => {
    setSnapshot((prev) => {
      const currentManual = prev.userAnnotations?.manualRelationships || [];
      const targetRel = currentManual.find((r) => r.id === relId);
      if (!targetRel) return prev;

      const newTombstone: RelationshipTombstone = {
        id: targetRel.id,
        source: targetRel.source,
        target: targetRel.target,
        type: targetRel.type,
        deletedAt: new Date().toISOString(),
      };

      const remainingManual = currentManual.filter((r) => r.id !== relId);
      const updatedTombstones = [...(prev.userAnnotations?.deletedRelationships || []), newTombstone];

      const updatedUserAnnotations: UserAnnotations = {
        schemaVersion: '1.0',
        exportedAt: new Date().toISOString(),
        authenticatedUser: prev.metadata.authenticatedUser,
        manualRelationships: remainingManual,
        deletedRelationships: updatedTombstones,
        potentialAssessments: prev.userAnnotations?.potentialAssessments || {},
      };

      const autoRels = prev.relationships.filter((r) => r.provenance === 'automatic');
      return {
        ...prev,
        relationships: [...autoRels, ...remainingManual],
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
        authenticatedUser: prev.metadata.authenticatedUser,
        manualRelationships: prev.userAnnotations?.manualRelationships || [],
        deletedRelationships: prev.userAnnotations?.deletedRelationships || [],
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

  // Export JSON Annotations only (no private repo data, easy backup)
  const handleExportAnnotations = () => {
    const annotationsToExport: UserAnnotations = snapshot.userAnnotations || {
      schemaVersion: '1.0',
      exportedAt: new Date().toISOString(),
      authenticatedUser: snapshot.metadata.authenticatedUser,
      manualRelationships: [],
      deletedRelationships: [],
      potentialAssessments: {},
    };

    const json = JSON.stringify(annotationsToExport, null, 2);
    const blob = new Blob([json], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `nexo-annotations-${snapshot.metadata.authenticatedUser}-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Snapshot file import handler (with deterministic annotations preservation and merging)
  const handleFileImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (evt) => {
      try {
        const parsed = JSON.parse(evt.target?.result as string);
        const validation = validateSnapshot(parsed);
        if (validation.success) {
          const incomingSnapshot = validation.data;
          const targetUser = incomingSnapshot.metadata.authenticatedUser;

          // Load any existing stored annotations for this user to avoid data loss
          const localStored = loadScopedAnnotations(targetUser);
          const currentAnnotations = localStored || (
            snapshot.metadata.authenticatedUser === targetUser ? snapshot.userAnnotations : undefined
          );

          let finalAnnotations = incomingSnapshot.userAnnotations;
          if (currentAnnotations && incomingSnapshot.userAnnotations) {
            finalAnnotations = mergeUserAnnotations(currentAnnotations, incomingSnapshot.userAnnotations);
          } else if (currentAnnotations) {
            finalAnnotations = currentAnnotations;
          }

          // Combine automatic relationships from incoming snapshot with merged active manual relationships
          const autoRels = incomingSnapshot.relationships.filter((r) => r.provenance === 'automatic');
          const deletedIds = new Set(finalAnnotations?.deletedRelationships?.map((t) => t.id) || []);
          const activeManual = (finalAnnotations?.manualRelationships || []).filter((r) => !deletedIds.has(r.id));

          const mergedSnapshot: Snapshot = {
            ...incomingSnapshot,
            relationships: [...autoRels, ...activeManual],
            userAnnotations: finalAnnotations,
          };

          setSnapshot(mergedSnapshot);
          setImportNotification({
            type: 'success',
            message: `Snapshot importado com sucesso! (${mergedSnapshot.repositories.length} repositórios, ${activeManual.length} relações manuais mescladas).`,
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

  // Annotations file import handler
  const handleAnnotationsFileImport = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (evt) => {
      try {
        const parsed = JSON.parse(evt.target?.result as string);
        const validation = validateAnnotations(parsed);
        if (validation.success) {
          const incomingAnnotations = validation.data;
          const currentAnnotations = snapshot.userAnnotations || {
            schemaVersion: '1.0',
            exportedAt: new Date().toISOString(),
            authenticatedUser: snapshot.metadata.authenticatedUser,
            manualRelationships: [],
            deletedRelationships: [],
            potentialAssessments: {},
          };

          const merged = mergeUserAnnotations(currentAnnotations, incomingAnnotations);
          const autoRels = snapshot.relationships.filter((r) => r.provenance === 'automatic');
          const deletedIds = new Set(merged.deletedRelationships?.map((t) => t.id) || []);
          const activeManual = merged.manualRelationships.filter((r) => !deletedIds.has(r.id));

          setSnapshot((prev) => ({
            ...prev,
            relationships: [...autoRels, ...activeManual],
            userAnnotations: merged,
          }));

          setImportNotification({
            type: 'success',
            message: `Anotações importadas e mescladas com sucesso! (${activeManual.length} relações manuais ativas).`,
          });
        } else {
          setImportNotification({
            type: 'error',
            message: `Falha na validação do arquivo de anotações: ${validation.errors.slice(0, 3).join('; ')}`,
          });
        }
      } catch (err: any) {
        setImportNotification({
          type: 'error',
          message: `Arquivo de anotações inválido: ${err?.message || err}`,
        });
      }
    };
    reader.readAsText(file);
    e.target.value = '';
  };

  const resetToDemo = () => {
    const demo = demoSnapshotRaw as unknown as Snapshot;
    // Restore demo data while retaining any annotations stored specifically for demo context
    const storedDemoAnno = loadScopedAnnotations(demo.metadata.authenticatedUser);
    const finalDemoAnno = storedDemoAnno || demo.userAnnotations;

    const autoRels = demo.relationships.filter((r) => r.provenance === 'automatic');
    const deletedIds = new Set(finalDemoAnno?.deletedRelationships?.map((t) => t.id) || []);
    const activeManual = (finalDemoAnno?.manualRelationships || []).filter((r) => !deletedIds.has(r.id));

    setSnapshot({
      ...demo,
      relationships: [...autoRels, ...activeManual],
      userAnnotations: finalDemoAnno,
    });

    setImportNotification({
      type: 'success',
      message: 'Base de dados restaurada para o contexto da demonstração sintética.',
    });
  };

  // Explicit wipe of all annotations with confirmation
  const handleClearAllAnnotations = () => {
    if (window.confirm('Tem certeza de que deseja apagar permanentemente todas as suas anotações e avaliações deste portfólio no navegador?')) {
      clearScopedAnnotations(snapshot.metadata.authenticatedUser);
      setSnapshot((prev) => {
        const autoRels = prev.relationships.filter((r) => r.provenance === 'automatic');
        return {
          ...prev,
          relationships: autoRels,
          userAnnotations: {
            schemaVersion: '1.0',
            exportedAt: new Date().toISOString(),
            authenticatedUser: prev.metadata.authenticatedUser,
            manualRelationships: [],
            deletedRelationships: [],
            potentialAssessments: {},
          },
        };
      });
      setImportNotification({
        type: 'success',
        message: 'Anotações deste portfólio foram removidas do navegador.',
      });
    }
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
        onExportAnnotations={handleExportAnnotations}
        onImportAnnotationsClick={() => annotationsFileInputRef.current?.click()}
        onExportReport={handleExportReport}
      />

      {/* Hidden file inputs for snapshot and annotations */}
      <input
        type="file"
        ref={fileInputRef}
        onChange={handleFileImport}
        accept=".json"
        className="hidden"
      />
      <input
        type="file"
        ref={annotationsFileInputRef}
        onChange={handleAnnotationsFileImport}
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

        {/* Sync Status Banner */}
        {snapshot.metadata.incompleteSync ? (
          <div className="mb-4 p-3.5 bg-amber-50 dark:bg-amber-950/40 border border-amber-300 dark:border-amber-800 rounded-xl text-xs text-amber-900 dark:text-amber-200 flex items-start gap-2.5">
            <AlertTriangle className="w-4 h-4 shrink-0 mt-0.5 text-amber-600 dark:text-amber-400" />
            <div className="flex-1">
              <span className="font-bold block text-[13px]">
                Status da Coleta: Sincronização Parcial / Incompleta
              </span>
              <p className="text-[11px] text-amber-800 dark:text-amber-300 mt-0.5">
                Existem lacunas materiais na coleta do GitHub (permissões, árvores truncadas ou limites de API). Os dados parciais estão disponíveis, e os indicadores não confirmados são classificados como Desconhecidos.
              </p>
              {snapshot.metadata.syncWarnings.length > 0 && (
                <ul className="list-disc pl-4 mt-1.5 space-y-0.5 text-[11px]">
                  {snapshot.metadata.syncWarnings.map((w, idx) => (
                    <li key={idx}>{w}</li>
                  ))}
                </ul>
              )}
            </div>
          </div>
        ) : (
          <div className="mb-3 px-3 py-1.5 bg-emerald-50/60 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-900 rounded-lg text-[11px] text-emerald-800 dark:text-emerald-300 flex items-center justify-between">
            <span className="flex items-center gap-1.5">
              <CheckCircle2 className="w-3.5 h-3.5 text-emerald-600 dark:text-emerald-400" />
              Sincronização completa de repositórios e relacionamentos.
            </span>
            <button
              onClick={handleClearAllAnnotations}
              className="text-[10px] text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 flex items-center gap-1 transition-colors"
              title="Apagar anotações locais deste portfólio"
            >
              <Trash2 className="w-3 h-3" />
              Limpar anotações locais
            </button>
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
            onUpdateManualRelationship={handleUpdateManualRelationship}
            onDeleteManualRelationship={handleDeleteManualRelationship}
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
