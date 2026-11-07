import React from 'react';
import { Repository, Organization, Relationship } from '../shared/types.js';
import { FolderGit2, Building2, Share2, ShieldCheck, Star, AlertTriangle } from 'lucide-react';

interface OverviewMetricsProps {
  repositories: Repository[];
  organizations: Organization[];
  relationships: Relationship[];
}

export const OverviewMetrics: React.FC<OverviewMetricsProps> = ({
  repositories,
  organizations,
  relationships,
}) => {
  const totalStars = repositories.reduce((acc, r) => acc + r.stargazersCount, 0);
  const reposWithTests = repositories.filter((r) => r.treeSignals.hasTests).length;
  const reposWithCI = repositories.filter((r) => r.treeSignals.hasCI).length;
  const privateCount = repositories.filter((r) => r.isPrivate).length;
  const publicCount = repositories.length - privateCount;

  const verifiedRelCount = relationships.filter((rel) => rel.verified).length;
  const manualRelCount = relationships.filter((rel) => rel.provenance === 'manual').length;

  return (
    <section className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-3 mb-6">
      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-3.5 rounded-xl shadow-sm">
        <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 mb-1">
          <FolderGit2 className="w-4 h-4 text-indigo-500" />
          <span className="text-xs font-medium">Repositórios</span>
        </div>
        <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">{repositories.length}</div>
        <div className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">
          {publicCount} públicos • {privateCount} privados
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-3.5 rounded-xl shadow-sm">
        <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 mb-1">
          <Building2 className="w-4 h-4 text-blue-500" />
          <span className="text-xs font-medium">Organizações</span>
        </div>
        <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">{organizations.length}</div>
        <div className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">
          Vínculos institucionais
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-3.5 rounded-xl shadow-sm">
        <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 mb-1">
          <Share2 className="w-4 h-4 text-emerald-500" />
          <span className="text-xs font-medium">Relações</span>
        </div>
        <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">{relationships.length}</div>
        <div className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">
          {verifiedRelCount} verificadas • {manualRelCount} manuais
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-3.5 rounded-xl shadow-sm">
        <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 mb-1">
          <ShieldCheck className="w-4 h-4 text-amber-500" />
          <span className="text-xs font-medium">Sinais de Teste</span>
        </div>
        <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">{reposWithTests}</div>
        <div className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">
          Com arquivos detectados
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-3.5 rounded-xl shadow-sm">
        <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 mb-1">
          <Star className="w-4 h-4 text-yellow-500" />
          <span className="text-xs font-medium">Automação CI</span>
        </div>
        <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">{reposWithCI}</div>
        <div className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">
          Workflows configurados
        </div>
      </div>

      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-3.5 rounded-xl shadow-sm">
        <div className="flex items-center gap-2 text-slate-500 dark:text-slate-400 mb-1">
          <AlertTriangle className="w-4 h-4 text-purple-500" />
          <span className="text-xs font-medium">Estrelas Totais</span>
        </div>
        <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">{totalStars}</div>
        <div className="text-[11px] text-slate-500 dark:text-slate-400 mt-0.5">
          Alcance de estrelas GitHub
        </div>
      </div>
    </section>
  );
};
