import React, { useState, useEffect } from 'react';
import {
  Repository,
  PotentialAssessment,
  Relationship,
  IndicatorDetail,
} from '../shared/types.js';
import {
  X,
  ExternalLink,
  Shield,
  AlertCircle,
  HelpCircle,
  CheckCircle2,
  XCircle,
  Save,
  Link2,
} from 'lucide-react';

interface RepositoryDetailDrawerProps {
  repo: Repository | null;
  onClose: () => void;
  potentialAssessment?: PotentialAssessment;
  onSavePotentialAssessment: (assessment: PotentialAssessment) => void;
  relationships: Relationship[];
  allRepositories: Repository[];
}

export const RepositoryDetailDrawer: React.FC<RepositoryDetailDrawerProps> = ({
  repo,
  onClose,
  potentialAssessment,
  onSavePotentialAssessment,
  relationships,
  allRepositories,
}) => {
  // Hooks called at top level unconditionally
  const [strategicValue, setStrategicValue] = useState<'low' | 'medium' | 'high' | 'critical'>('medium');
  const [commercialPotential, setCommercialPotential] = useState<
    'none' | 'experimental' | 'viable' | 'production_ready'
  >('experimental');
  const [targetAudience, setTargetAudience] = useState('');
  const [notes, setNotes] = useState('');
  const [tagsInput, setTagsInput] = useState('');
  const [savedSuccess, setSavedSuccess] = useState(false);

  useEffect(() => {
    if (potentialAssessment) {
      setStrategicValue(potentialAssessment.strategicValue);
      setCommercialPotential(potentialAssessment.commercialPotential);
      setTargetAudience(potentialAssessment.targetAudience || '');
      setNotes(potentialAssessment.notes || '');
      setTagsInput((potentialAssessment.tags || []).join(', '));
    } else {
      setStrategicValue('medium');
      setCommercialPotential('experimental');
      setTargetAudience('');
      setNotes('');
      setTagsInput('');
    }
  }, [potentialAssessment, repo]);

  if (!repo) return null;

  const handleSaveAssessment = (e: React.FormEvent) => {
    e.preventDefault();
    const tags = tagsInput
      .split(',')
      .map((t) => t.trim())
      .filter((t) => t.length > 0);

    onSavePotentialAssessment({
      repositoryId: repo.id,
      strategicValue,
      commercialPotential,
      targetAudience,
      notes,
      tags,
      updatedAt: new Date().toISOString(),
    });

    setSavedSuccess(true);
    setTimeout(() => setSavedSuccess(false), 2500);
  };

  const related = relationships.filter(
    (rel) => rel.source === String(repo.id) || rel.target === String(repo.id)
  );

  return (
    <div className="fixed inset-0 z-50 overflow-hidden bg-slate-900/50 backdrop-blur-sm flex justify-end">
      <div className="w-full max-w-2xl bg-white dark:bg-slate-900 h-full shadow-2xl flex flex-col overflow-y-auto border-l border-slate-200 dark:border-slate-800 transition-colors">
        {/* Drawer Header */}
        <div className="p-4 border-b border-slate-200 dark:border-slate-800 flex items-start justify-between sticky top-0 bg-white/95 dark:bg-slate-900/95 backdrop-blur z-10">
          <div>
            <div className="flex items-center gap-2">
              <span
                className={`text-[10px] font-bold px-2 py-0.5 rounded uppercase tracking-wider ${
                  repo.isPrivate
                    ? 'bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300'
                    : 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300'
                }`}
              >
                {repo.isPrivate ? 'Privado' : 'Público'}
              </span>
              {repo.archived && (
                <span className="text-[10px] font-bold px-2 py-0.5 rounded uppercase bg-rose-100 text-rose-800 dark:bg-rose-950 dark:text-rose-300">
                  Arquivado
                </span>
              )}
            </div>
            <h2 className="text-xl font-bold text-slate-900 dark:text-slate-100 mt-1">
              {repo.fullName}
            </h2>
            <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
              ID Imutável: <span className="font-mono">{repo.id}</span> • Branch: {repo.defaultBranch}
            </p>
          </div>

          <div className="flex items-center gap-2">
            <a
              href={repo.htmlUrl}
              target="_blank"
              rel="noreferrer"
              className="p-1.5 text-slate-500 hover:text-indigo-600 dark:hover:text-indigo-400 rounded-md transition-colors"
              title="Abrir no GitHub"
            >
              <ExternalLink className="w-5 h-5" />
            </a>
            <button
              onClick={onClose}
              className="p-1.5 text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-md transition-colors"
              aria-label="Fechar painel"
            >
              <X className="w-5 h-5" />
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="p-6 space-y-6 flex-1">
          {/* Description & Core Specs */}
          <div>
            <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider mb-2">Visão Geral</h3>
            <p className="text-sm text-slate-700 dark:text-slate-300 bg-slate-50 dark:bg-slate-800/50 p-3 rounded-lg border border-slate-200 dark:border-slate-800">
              {repo.description || 'Sem descrição cadastrada no GitHub.'}
            </p>

            <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 mt-3 text-center">
              <div className="p-2 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <span className="text-xs text-slate-500 block">Linguagem</span>
                <span className="text-sm font-semibold text-slate-800 dark:text-slate-200">
                  {repo.primaryLanguage || 'N/A'}
                </span>
              </div>
              <div className="p-2 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <span className="text-xs text-slate-500 block">Tamanho</span>
                <span className="text-sm font-semibold text-slate-800 dark:text-slate-200">
                  {repo.sizeKb} KB
                </span>
              </div>
              <div className="p-2 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <span className="text-xs text-slate-500 block">Estrelas</span>
                <span className="text-sm font-semibold text-slate-800 dark:text-slate-200">
                  ⭐ {repo.stargazersCount}
                </span>
              </div>
              <div className="p-2 bg-slate-50 dark:bg-slate-800 rounded-lg">
                <span className="text-xs text-slate-500 block">Issues Abertas</span>
                <span className="text-sm font-semibold text-slate-800 dark:text-slate-200">
                  {repo.openIssuesCount}
                </span>
              </div>
            </div>
          </div>

          {/* Explainable Indicators Section */}
          <div>
            <div className="flex items-center justify-between mb-3">
              <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1.5">
                <Shield className="w-4 h-4 text-indigo-500" />
                Indicadores Técnicos Explicáveis
              </h3>
              {repo.indicators && (
                <span className="text-xs font-bold px-2 py-0.5 rounded-full bg-indigo-100 text-indigo-800 dark:bg-indigo-950 dark:text-indigo-300">
                  Escore Geral: {repo.indicators.overallQualityScore}/100
                </span>
              )}
            </div>

            {repo.indicators ? (
              <div className="space-y-3">
                <IndicatorCard indicator={repo.indicators.testInfrastructure} />
                <IndicatorCard indicator={repo.indicators.ciConfiguration} />
                <IndicatorCard indicator={repo.indicators.documentation} />
                <IndicatorCard indicator={repo.indicators.activityRecency} />
                <IndicatorCard indicator={repo.indicators.maintenanceSignals} />
              </div>
            ) : (
              <p className="text-xs text-slate-500 italic">Indicadores não disponíveis para este repositório.</p>
            )}
          </div>

          {/* Relationships in Graph */}
          <div>
            <h3 className="text-xs font-bold text-slate-400 uppercase tracking-wider flex items-center gap-1.5 mb-2">
              <Link2 className="w-4 h-4 text-emerald-500" />
              Relacionamentos Conectados ({related.length})
            </h3>
            {related.length === 0 ? (
              <p className="text-xs text-slate-500 italic">Nenhum relacionamento explícito registrado.</p>
            ) : (
              <div className="space-y-2">
                {related.map((rel) => {
                  const isSource = rel.source === String(repo.id);
                  const otherId = isSource ? rel.target : rel.source;
                  const otherRepo = allRepositories.find((r) => String(r.id) === otherId);
                  const otherLabel = otherRepo ? otherRepo.fullName : `ID: ${otherId}`;

                  return (
                    <div
                      key={rel.id}
                      className="p-2.5 rounded-lg border border-slate-200 dark:border-slate-800 bg-slate-50 dark:bg-slate-800/40 text-xs"
                    >
                      <div className="flex items-center justify-between font-medium text-slate-800 dark:text-slate-200">
                        <span>
                          {isSource ? '→ Aponta para: ' : '← Originado por: '}
                          <span className="font-bold">{otherLabel}</span>
                        </span>
                        <span
                          className={`text-[10px] px-1.5 py-0.5 rounded font-bold uppercase ${
                            rel.provenance === 'manual'
                              ? 'bg-purple-100 text-purple-800 dark:bg-purple-950 dark:text-purple-300'
                              : 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300'
                          }`}
                        >
                          {rel.provenance === 'manual' ? 'Manual' : 'Verificado'}
                        </span>
                      </div>
                      <div className="text-slate-500 dark:text-slate-400 mt-1">
                        Tipo: <code className="font-mono font-semibold">{rel.type}</code>
                      </div>
                      {rel.evidence && (
                        <div className="mt-1 text-slate-600 dark:text-slate-300 bg-white dark:bg-slate-900 p-1.5 rounded border border-slate-200 dark:border-slate-800 text-[11px]">
                          <strong>Evidência:</strong> {rel.evidence}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          {/* User-maintained Potential Assessment (Explicitly separated from GitHub metrics) */}
          <div className="border-t border-slate-200 dark:border-slate-800 pt-5">
            <div className="bg-amber-50 dark:bg-amber-950/30 border border-amber-200 dark:border-amber-900/50 p-3 rounded-lg mb-4 text-xs text-amber-800 dark:text-amber-300">
              <p className="font-bold flex items-center gap-1.5 mb-1">
                <AlertCircle className="w-4 h-4 shrink-0 text-amber-600 dark:text-amber-400" />
                Avaliação de Potencial Mantida pelo Usuário
              </p>
              <p className="text-[11px] leading-relaxed">
                Esta camada é mantida exclusivamente por você. Os metadados do GitHub <strong>não</strong> medem objetivamente o potencial comercial ou estratégico de um software.
              </p>
            </div>

            <form onSubmit={handleSaveAssessment} className="space-y-3">
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                    Valor Estratégico
                  </label>
                  <select
                    value={strategicValue}
                    onChange={(e) => setStrategicValue(e.target.value as 'low' | 'medium' | 'high' | 'critical')}
                    className="w-full text-xs p-2 rounded-lg bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-slate-100"
                  >
                    <option value="low">Baixo</option>
                    <option value="medium">Médio</option>
                    <option value="high">Alto</option>
                    <option value="critical">Crítico</option>
                  </select>
                </div>

                <div>
                  <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                    Potencial Comercial
                  </label>
                  <select
                    value={commercialPotential}
                    onChange={(e) => setCommercialPotential(e.target.value as 'none' | 'experimental' | 'viable' | 'production_ready')}
                    className="w-full text-xs p-2 rounded-lg bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-slate-100"
                  >
                    <option value="none">Nenhum / Interno</option>
                    <option value="experimental">Experimental</option>
                    <option value="viable">Viável</option>
                    <option value="production_ready">Pronto para Produção</option>
                  </select>
                </div>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                  Público-Alvo ou Casos de Uso
                </label>
                <input
                  type="text"
                  placeholder="Ex: Engenheiros de dados, equipes de observabilidade..."
                  value={targetAudience}
                  onChange={(e) => setTargetAudience(e.target.value)}
                  className="w-full text-xs p-2 rounded-lg bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-slate-100"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                  Tags Personalizadas (separadas por vírgula)
                </label>
                <input
                  type="text"
                  placeholder="Ex: prioridade, arquitetura-nova, incubadora"
                  value={tagsInput}
                  onChange={(e) => setTagsInput(e.target.value)}
                  className="w-full text-xs p-2 rounded-lg bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-slate-100"
                />
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                  Notas de Avaliação Técnica/Estratégica
                </label>
                <textarea
                  rows={3}
                  placeholder="Anotações subjetivas sobre decisões arquiteturais, dependências de equipe ou planos futuros..."
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                  className="w-full text-xs p-2 rounded-lg bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-slate-100"
                />
              </div>

              <div className="flex items-center justify-between pt-2">
                <button
                  type="submit"
                  className="flex items-center gap-1.5 px-4 py-2 text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition-colors shadow-sm"
                >
                  <Save className="w-3.5 h-3.5" />
                  Salvar Avaliação
                </button>
                {savedSuccess && (
                  <span className="text-xs text-emerald-600 dark:text-emerald-400 font-medium">
                    ✓ Avaliação salva localmente!
                  </span>
                )}
              </div>
            </form>
          </div>
        </div>
      </div>
    </div>
  );
};

interface IndicatorCardProps {
  indicator: IndicatorDetail;
}

const IndicatorCard: React.FC<IndicatorCardProps> = ({ indicator }) => {
  const [expanded, setExpanded] = useState(false);

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'passed':
        return (
          <span className="flex items-center gap-1 text-[11px] font-semibold text-emerald-700 dark:text-emerald-400 bg-emerald-100 dark:bg-emerald-950/50 px-2 py-0.5 rounded">
            <CheckCircle2 className="w-3.5 h-3.5" /> Presente
          </span>
        );
      case 'partial':
        return (
          <span className="flex items-center gap-1 text-[11px] font-semibold text-amber-700 dark:text-amber-400 bg-amber-100 dark:bg-amber-950/50 px-2 py-0.5 rounded">
            <AlertCircle className="w-3.5 h-3.5" /> Parcial
          </span>
        );
      case 'failed':
        return (
          <span className="flex items-center gap-1 text-[11px] font-semibold text-rose-700 dark:text-rose-400 bg-rose-100 dark:bg-rose-950/50 px-2 py-0.5 rounded">
            <XCircle className="w-3.5 h-3.5" /> Ausente
          </span>
        );
      case 'unknown':
      default:
        return (
          <span className="flex items-center gap-1 text-[11px] font-semibold text-slate-700 dark:text-slate-400 bg-slate-200 dark:bg-slate-800 px-2 py-0.5 rounded">
            <HelpCircle className="w-3.5 h-3.5" /> Desconhecido
          </span>
        );
    }
  };

  return (
    <div className="border border-slate-200 dark:border-slate-800 rounded-lg p-3 bg-white dark:bg-slate-900 transition-colors">
      <div className="flex items-start justify-between">
        <div>
          <div className="flex items-center gap-2">
            <h4 className="text-xs font-bold text-slate-800 dark:text-slate-200">{indicator.name}</h4>
            {getStatusBadge(indicator.status)}
            <span className="text-[11px] font-mono text-slate-500">({indicator.score}/100)</span>
          </div>
          <p className="text-xs text-slate-600 dark:text-slate-400 mt-1">{indicator.summary}</p>
        </div>

        <button
          onClick={() => setExpanded(!expanded)}
          className="text-[11px] font-medium text-indigo-600 dark:text-indigo-400 hover:underline shrink-0 ml-2"
        >
          {expanded ? 'Ocultar detalhes' : 'Como é calculado?'}
        </button>
      </div>

      {expanded && (
        <div className="mt-3 pt-3 border-t border-slate-100 dark:border-slate-800 text-[11px] space-y-2 text-slate-600 dark:text-slate-400 bg-slate-50 dark:bg-slate-800/30 p-2.5 rounded">
          <div>
            <strong className="text-slate-700 dark:text-slate-300 block">Fórmula / Método:</strong>
            <span>{indicator.calculation}</span>
          </div>

          <div>
            <strong className="text-slate-700 dark:text-slate-300 block">Dados de Entrada Inspecionados:</strong>
            <pre className="mt-1 p-1.5 bg-slate-100 dark:bg-slate-900 rounded font-mono text-[10px] overflow-x-auto">
              {JSON.stringify(indicator.inputs, null, 2)}
            </pre>
          </div>

          <div>
            <strong className="text-slate-700 dark:text-slate-300 block">Limitações Declaradas:</strong>
            <ul className="list-disc pl-4 space-y-0.5">
              {indicator.limitations.map((lim, idx) => (
                <li key={idx}>{lim}</li>
              ))}
            </ul>
          </div>
        </div>
      )}
    </div>
  );
};
