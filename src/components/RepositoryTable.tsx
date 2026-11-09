import React from 'react';
import { Repository } from '../shared/types.js';
import { Lock, Globe, GitFork, AlertCircle, HelpCircle } from 'lucide-react';

interface RepositoryTableProps {
  repositories: Repository[];
  onSelectRepo: (repo: Repository) => void;
  selectedRepoId?: number;
}

export const RepositoryTable: React.FC<RepositoryTableProps> = ({
  repositories,
  onSelectRepo,
  selectedRepoId,
}) => {
  if (repositories.length === 0) {
    return (
      <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl p-8 text-center">
        <AlertCircle className="w-8 h-8 text-slate-400 mx-auto mb-2" />
        <h3 className="text-sm font-semibold text-slate-700 dark:text-slate-300">
          Nenhum repositório encontrado
        </h3>
        <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
          Tente ajustar os filtros de busca, visibilidade ou linguagem acima.
        </p>
      </div>
    );
  }

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm transition-colors">
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse text-xs">
          <thead>
            <tr className="border-b border-slate-200 dark:border-slate-800 bg-slate-50/75 dark:bg-slate-800/50 text-slate-500 dark:text-slate-400 font-semibold uppercase tracking-wider">
              <th className="py-3 px-4">Repositório</th>
              <th className="py-3 px-3">Visibilidade</th>
              <th className="py-3 px-3">Linguagem</th>
              <th className="py-3 px-3">Testes / CI</th>
              <th className="py-3 px-3">Escore Geral</th>
              <th className="py-3 px-3">Última Atividade</th>
              <th className="py-3 px-4 text-right">Ações</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
            {repositories.map((repo) => {
              const isSelected = selectedRepoId === repo.id;
              const overallScore = repo.indicators?.overallQualityScore ?? 0;
              const hasTests = repo.treeSignals.hasTests;
              const hasCI = repo.treeSignals.hasCI;
              const treeUninspectedOrTruncated = !repo.treeSignals.treeInspected || repo.treeSignals.treeTruncated;

              const testStatus = repo.indicators?.testInfrastructure.status;
              const isTestUnknown = !hasTests && (treeUninspectedOrTruncated || testStatus === 'unknown');

              const ciStatus = repo.indicators?.ciConfiguration.status;
              const isCIUnknown = !hasCI && (treeUninspectedOrTruncated || ciStatus === 'unknown');

              return (
                <tr
                  key={repo.id}
                  onClick={() => onSelectRepo(repo)}
                  className={`hover:bg-slate-50 dark:hover:bg-slate-800/60 cursor-pointer transition-colors ${
                    isSelected ? 'bg-indigo-50/70 dark:bg-indigo-950/40' : ''
                  }`}
                >
                  <td className="py-3 px-4">
                    <div className="font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-1.5">
                      <span>{repo.name}</span>
                      {repo.fork && <GitFork className="w-3 h-3 text-slate-400" />}
                    </div>
                    <div className="text-[11px] text-slate-500 dark:text-slate-400 truncate max-w-xs">
                      {repo.fullName}
                    </div>
                  </td>

                  <td className="py-3 px-3">
                    {repo.isPrivate ? (
                      <span className="inline-flex items-center gap-1 text-[11px] font-medium text-amber-700 dark:text-amber-400 bg-amber-50 dark:bg-amber-950/50 px-2 py-0.5 rounded">
                        <Lock className="w-3 h-3" /> Privado
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1 text-[11px] font-medium text-emerald-700 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/50 px-2 py-0.5 rounded">
                        <Globe className="w-3 h-3" /> Público
                      </span>
                    )}
                  </td>

                  <td className="py-3 px-3">
                    <span className="font-medium text-slate-700 dark:text-slate-300">
                      {repo.primaryLanguage || '—'}
                    </span>
                  </td>

                  <td className="py-3 px-3">
                    <div className="flex items-center gap-1.5">
                      {/* TEST Indicator Badge */}
                      {hasTests ? (
                        <span
                          className="text-[10px] font-bold px-1.5 py-0.5 rounded bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300"
                          title="Arquivos de teste detectados"
                        >
                          TEST
                        </span>
                      ) : isTestUnknown ? (
                        <span
                          className="inline-flex items-center gap-0.5 text-[10px] font-bold px-1.5 py-0.5 rounded bg-amber-50 text-amber-700 dark:bg-amber-950/60 dark:text-amber-300 border border-amber-200 dark:border-amber-800"
                          title="Inspeção parcial ou não inspecionado: presença de testes desconhecida"
                        >
                          <HelpCircle className="w-2.5 h-2.5" /> TEST?
                        </span>
                      ) : (
                        <span
                          className="text-[10px] font-bold px-1.5 py-0.5 rounded bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400"
                          title="Nenhum teste detectado na árvore completa"
                        >
                          TEST
                        </span>
                      )}

                      {/* CI Indicator Badge */}
                      {hasCI ? (
                        <span
                          className="text-[10px] font-bold px-1.5 py-0.5 rounded bg-blue-100 text-blue-800 dark:bg-blue-950 dark:text-blue-300"
                          title="Workflows de CI detectados"
                        >
                          CI
                        </span>
                      ) : isCIUnknown ? (
                        <span
                          className="inline-flex items-center gap-0.5 text-[10px] font-bold px-1.5 py-0.5 rounded bg-amber-50 text-amber-700 dark:bg-amber-950/60 dark:text-amber-300 border border-amber-200 dark:border-amber-800"
                          title="Inspeção parcial ou não inspecionado: presença de CI desconhecida"
                        >
                          <HelpCircle className="w-2.5 h-2.5" /> CI?
                        </span>
                      ) : (
                        <span
                          className="text-[10px] font-bold px-1.5 py-0.5 rounded bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-400"
                          title="Nenhum CI detectado na árvore completa"
                        >
                          CI
                        </span>
                      )}
                    </div>
                  </td>

                  <td className="py-3 px-3">
                    <div className="flex items-center gap-2">
                      <div className="w-12 bg-slate-200 dark:bg-slate-700 h-2 rounded-full overflow-hidden">
                        <div
                          className={`h-full ${
                            overallScore >= 70
                              ? 'bg-emerald-500'
                              : overallScore >= 40
                              ? 'bg-amber-500'
                              : 'bg-rose-500'
                          }`}
                          style={{ width: `${overallScore}%` }}
                        />
                      </div>
                      <span className="font-semibold text-slate-700 dark:text-slate-200">
                        {overallScore}
                      </span>
                    </div>
                  </td>

                  <td className="py-3 px-3 text-slate-500 dark:text-slate-400 whitespace-nowrap">
                    {new Date(repo.pushedAt).toLocaleDateString('pt-BR')}
                  </td>

                  <td className="py-3 px-4 text-right">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onSelectRepo(repo);
                      }}
                      className="px-2.5 py-1 text-xs font-medium text-indigo-600 dark:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-950/60 rounded transition-colors"
                    >
                      Detalhes →
                    </button>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
};
