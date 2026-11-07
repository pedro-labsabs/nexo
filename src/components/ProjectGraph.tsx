import React, { useEffect, useRef, useState } from 'react';
import cytoscape, { Core } from 'cytoscape';
import { Repository, Organization, Relationship } from '../shared/types.js';
import { ZoomIn, ZoomOut, Maximize2, Plus } from 'lucide-react';

interface ProjectGraphProps {
  repositories: Repository[];
  organizations: Organization[];
  relationships: Relationship[];
  onSelectNode: (repo: Repository) => void;
  onAddManualRelationship: (rel: Relationship) => void;
  theme: 'light' | 'dark';
}

export const ProjectGraph: React.FC<ProjectGraphProps> = ({
  repositories,
  organizations,
  relationships,
  onSelectNode,
  onAddManualRelationship,
  theme,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const cyRef = useRef<Core | null>(null);

  const [selectedElementInfo, setSelectedElementInfo] = useState<{
    type: 'node' | 'edge';
    title: string;
    details: string;
    evidence?: string;
    provenance?: string;
  } | null>(null);

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [modalSource, setModalSource] = useState(repositories[0]?.id ? String(repositories[0].id) : '');
  const [modalTarget, setModalTarget] = useState(repositories[1]?.id ? String(repositories[1].id) : '');
  const [modalType, setModalType] = useState<'references' | 'depends_on' | 'custom'>('custom');
  const [modalEvidence, setModalEvidence] = useState('');

  // Builds Cytoscape elements
  useEffect(() => {
    if (!containerRef.current) return;

    const elements: cytoscape.ElementDefinition[] = [];

    // 1. Organization nodes
    for (const org of organizations) {
      elements.push({
        group: 'nodes',
        data: {
          id: `org_${org.id}`,
          rawId: org.id,
          label: org.name || org.login,
          nodeType: 'organization',
        },
      });
    }

    // 2. Repository nodes
    for (const repo of repositories) {
      elements.push({
        group: 'nodes',
        data: {
          id: String(repo.id),
          rawId: repo.id,
          label: repo.name,
          fullName: repo.fullName,
          nodeType: 'repository',
          isPrivate: repo.isPrivate,
          score: repo.indicators?.overallQualityScore || 0,
        },
      });
    }

    // 3. Relationships / Edges
    for (const rel of relationships) {
      const sourceId = rel.sourceType === 'organization' ? `org_${rel.source}` : rel.source;
      const targetId = rel.targetType === 'organization' ? `org_${rel.target}` : rel.target;

      elements.push({
        group: 'edges',
        data: {
          id: rel.id,
          source: sourceId,
          target: targetId,
          relationType: rel.type,
          provenance: rel.provenance,
          verified: rel.verified,
          evidence: rel.evidence,
          description: rel.description,
        },
      });
    }

    const isDark = theme === 'dark';

    const cy = cytoscape({
      container: containerRef.current,
      elements,
      style: [
        {
          selector: 'node[nodeType = "organization"]',
          style: {
            'shape': 'round-rectangle',
            'background-color': isDark ? '#3b82f6' : '#2563eb',
            'label': 'data(label)',
            'color': '#ffffff',
            'font-size': '11px',
            'font-weight': 'bold',
            'text-valign': 'center',
            'text-halign': 'center',
            'padding': '12px',
            'width': 'label',
            'height': '36px',
            'border-width': 2,
            'border-color': isDark ? '#1d4ed8' : '#1e40af',
          },
        },
        {
          selector: 'node[nodeType = "repository"]',
          style: {
            'shape': 'ellipse',
            'background-color': isDark ? '#1e293b' : '#f8fafc',
            'label': 'data(label)',
            'color': isDark ? '#f1f5f9' : '#0f172a',
            'font-size': '11px',
            'text-valign': 'bottom',
            'text-margin-y': 5,
            'width': 34,
            'height': 34,
            'border-width': 2.5,
            'border-color': isDark ? '#6366f1' : '#4f46e5',
          },
        },
        {
          selector: 'node:selected',
          style: {
            'border-width': 4,
            'border-color': '#f59e0b',
          },
        },
        {
          selector: 'edge',
          style: {
            'width': 2,
            'line-color': isDark ? '#64748b' : '#94a3b8',
            'target-arrow-color': isDark ? '#64748b' : '#94a3b8',
            'target-arrow-shape': 'triangle',
            'curve-style': 'bezier',
            'arrow-scale': 0.8,
          },
        },
        {
          selector: 'edge[relationType = "belongs_to"]',
          style: {
            'line-style': 'dashed',
            'line-color': isDark ? '#475569' : '#cbd5e1',
            'target-arrow-shape': 'triangle',
            'width': 1.5,
          },
        },
        {
          selector: 'edge[relationType = "depends_on"]',
          style: {
            'width': 3,
            'line-color': '#10b981',
            'target-arrow-color': '#10b981',
            'target-arrow-shape': 'triangle',
          },
        },
        {
          selector: 'edge[provenance = "manual"]',
          style: {
            'line-style': 'dotted',
            'line-color': '#a855f7',
            'target-arrow-color': '#a855f7',
            'target-arrow-shape': 'diamond',
            'width': 2.5,
          },
        },
        {
          selector: 'edge:selected',
          style: {
            'width': 4,
            'line-color': '#f59e0b',
            'target-arrow-color': '#f59e0b',
          },
        },
      ] as any,
      layout: {
        name: 'cose',
        animate: false,
        padding: 40,
        nodeRepulsion: () => 450000,
        idealEdgeLength: () => 90,
      } as any,
    });

    cy.on('tap', 'node', (evt) => {
      const node = evt.target;
      const nodeType = node.data('nodeType');
      if (nodeType === 'repository') {
        const repoId = Number(node.data('rawId'));
        const found = repositories.find((r) => r.id === repoId);
        if (found) {
          onSelectNode(found);
          setSelectedElementInfo({
            type: 'node',
            title: found.fullName,
            details: `Repositório • ${found.isPrivate ? 'Privado' : 'Público'} • Score: ${found.indicators?.overallQualityScore || 0}/100`,
          });
        }
      } else {
        setSelectedElementInfo({
          type: 'node',
          title: `Organização: ${node.data('label')}`,
          details: 'Nó institucional',
        });
      }
    });

    cy.on('tap', 'edge', (evt) => {
      const edge = evt.target;
      setSelectedElementInfo({
        type: 'edge',
        title: `Relação: ${edge.data('relationType')}`,
        details: edge.data('description') || 'Conexão entre entidades',
        evidence: edge.data('evidence'),
        provenance: edge.data('provenance'),
      });
    });

    cyRef.current = cy;

    return () => {
      cy.destroy();
    };
  }, [repositories, organizations, relationships, theme, onSelectNode]);

  const handleZoomIn = () => cyRef.current?.zoom(cyRef.current.zoom() * 1.25);
  const handleZoomOut = () => cyRef.current?.zoom(cyRef.current.zoom() * 0.8);
  const handleFit = () => cyRef.current?.fit();

  const handleAddRelationshipSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!modalSource || !modalTarget || modalSource === modalTarget) {
      alert('Selecione dois repositórios distintos para conectar.');
      return;
    }

    const newRel: Relationship = {
      id: `manual_rel_${Date.now()}`,
      source: modalSource,
      target: modalTarget,
      sourceType: 'repository',
      targetType: 'repository',
      type: modalType,
      provenance: 'manual',
      verified: false,
      evidence: modalEvidence || 'Inserido manualmente pelo usuário.',
      description: `Relação manual (${modalType})`,
      createdAt: new Date().toISOString(),
    };

    onAddManualRelationship(newRel);
    setIsModalOpen(false);
    setModalEvidence('');
  };

  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 rounded-xl overflow-hidden shadow-sm mb-6 flex flex-col transition-colors">
      <div className="p-3 border-b border-slate-200 dark:border-slate-800 flex flex-wrap items-center justify-between gap-3 bg-slate-50/75 dark:bg-slate-800/40">
        <div className="flex items-center gap-3">
          <h3 className="text-xs font-bold text-slate-800 dark:text-slate-200 uppercase tracking-wider">
            Mapa Interativo do Portfólio (Cytoscape.js)
          </h3>
          <span className="text-[11px] text-slate-500 dark:text-slate-400 hidden sm:inline">
            Clique nos nós para inspecionar e veja as conexões comprovadas
          </span>
        </div>

        <div className="flex items-center gap-1.5">
          <button
            onClick={() => setIsModalOpen(true)}
            className="flex items-center gap-1 px-2.5 py-1 text-xs font-semibold text-purple-700 dark:text-purple-300 bg-purple-50 dark:bg-purple-950/60 hover:bg-purple-100 dark:hover:bg-purple-900 border border-purple-200 dark:border-purple-800 rounded-lg transition-colors"
          >
            <Plus className="w-3.5 h-3.5" />
            Nova Relação Manual
          </button>
          <button
            onClick={handleZoomIn}
            className="p-1.5 bg-white dark:bg-slate-800 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-200 dark:border-slate-700 rounded text-slate-700 dark:text-slate-200"
            title="Aproximar Zoom"
          >
            <ZoomIn className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={handleZoomOut}
            className="p-1.5 bg-white dark:bg-slate-800 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-200 dark:border-slate-700 rounded text-slate-700 dark:text-slate-200"
            title="Afastar Zoom"
          >
            <ZoomOut className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={handleFit}
            className="p-1.5 bg-white dark:bg-slate-800 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-200 dark:border-slate-700 rounded text-slate-700 dark:text-slate-200"
            title="Enquadrar Mapa"
          >
            <Maximize2 className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      <div className="relative w-full h-[460px] bg-slate-50 dark:bg-slate-950">
        <div ref={containerRef} className="w-full h-full" />

        <div className="absolute bottom-3 left-3 bg-white/90 dark:bg-slate-900/90 backdrop-blur border border-slate-200 dark:border-slate-800 p-2.5 rounded-lg text-[10px] space-y-1 shadow-sm pointer-events-none">
          <div className="font-bold text-slate-700 dark:text-slate-300 mb-1">Legenda:</div>
          <div className="flex items-center gap-1.5">
            <span className="w-3 h-2 rounded-sm bg-blue-600 inline-block" />
            <span className="text-slate-600 dark:text-slate-400">Organização</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-2.5 h-2.5 rounded-full border border-indigo-600 bg-white dark:bg-slate-800 inline-block" />
            <span className="text-slate-600 dark:text-slate-400">Repositório</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-3 h-0.5 bg-emerald-500 inline-block" />
            <span className="text-slate-600 dark:text-slate-400">Verificado (manifesto/doc)</span>
          </div>
          <div className="flex items-center gap-1.5">
            <span className="w-3 h-0.5 border-b-2 border-dotted border-purple-500 inline-block" />
            <span className="text-slate-600 dark:text-slate-400">Relação Manual</span>
          </div>
        </div>

        {selectedElementInfo && (
          <div className="absolute top-3 right-3 max-w-xs bg-white/95 dark:bg-slate-900/95 backdrop-blur border border-slate-200 dark:border-slate-800 p-3 rounded-lg text-xs shadow-md">
            <div className="flex items-center justify-between">
              <span className="font-bold text-slate-900 dark:text-slate-100 truncate">
                {selectedElementInfo.title}
              </span>
              <button
                onClick={() => setSelectedElementInfo(null)}
                className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 ml-2"
              >
                ✕
              </button>
            </div>
            <p className="text-[11px] text-slate-600 dark:text-slate-400 mt-1">
              {selectedElementInfo.details}
            </p>
            {selectedElementInfo.evidence && (
              <div className="mt-1 text-[10px] text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800 p-1.5 rounded">
                <strong>Evidência:</strong> {selectedElementInfo.evidence}
              </div>
            )}
            {selectedElementInfo.provenance && (
              <div className="mt-1 text-[10px] font-semibold text-purple-600 dark:text-purple-400">
                Proveniência: {selectedElementInfo.provenance === 'manual' ? 'Manual do Usuário' : 'Automática'}
              </div>
            )}
          </div>
        )}
      </div>

      {isModalOpen && (
        <div className="fixed inset-0 z-50 overflow-hidden bg-slate-900/50 backdrop-blur-sm flex items-center justify-center p-4">
          <div className="w-full max-w-md bg-white dark:bg-slate-900 rounded-xl shadow-2xl p-5 border border-slate-200 dark:border-slate-800">
            <div className="flex items-center justify-between mb-3">
              <h4 className="text-sm font-bold text-slate-900 dark:text-slate-100">
                Adicionar Relação Manual
              </h4>
              <button
                onClick={() => setIsModalOpen(false)}
                className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
              >
                ✕
              </button>
            </div>

            <p className="text-xs text-slate-500 dark:text-slate-400 mb-4">
              Crie uma conexão explícita entre dois repositórios. Ela será rotulada claramente como manual e preservada na exportação de anotações.
            </p>

            <form onSubmit={handleAddRelationshipSubmit} className="space-y-3">
              <div>
                <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                  Repositório de Origem
                </label>
                <select
                  value={modalSource}
                  onChange={(e) => setModalSource(e.target.value)}
                  className="w-full text-xs p-2 rounded-lg bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-slate-100"
                >
                  {repositories.map((r) => (
                    <option key={r.id} value={String(r.id)}>
                      {r.fullName}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                  Repositório de Destino
                </label>
                <select
                  value={modalTarget}
                  onChange={(e) => setModalTarget(e.target.value)}
                  className="w-full text-xs p-2 rounded-lg bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-slate-100"
                >
                  {repositories.map((r) => (
                    <option key={r.id} value={String(r.id)}>
                      {r.fullName}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                  Tipo de Relação
                </label>
                <select
                  value={modalType}
                  onChange={(e) => setModalType(e.target.value as any)}
                  className="w-full text-xs p-2 rounded-lg bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-slate-100"
                >
                  <option value="custom">Relação Customizada (custom)</option>
                  <option value="references">Referência Documental (references)</option>
                  <option value="depends_on">Dependência Técnica (depends_on)</option>
                </select>
              </div>

              <div>
                <label className="block text-xs font-medium text-slate-700 dark:text-slate-300 mb-1">
                  Justificativa / Evidência Manual
                </label>
                <textarea
                  rows={2}
                  value={modalEvidence}
                  onChange={(e) => setModalEvidence(e.target.value)}
                  placeholder="Por que esses projetos se conectam? (ex: compartilham schema de banco ou fila kafka)"
                  className="w-full text-xs p-2 rounded-lg bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 text-slate-900 dark:text-slate-100"
                />
              </div>

              <div className="flex items-center justify-end gap-2 pt-2">
                <button
                  type="button"
                  onClick={() => setIsModalOpen(false)}
                  className="px-3 py-1.5 text-xs text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-800 rounded-lg"
                >
                  Cancelar
                </button>
                <button
                  type="submit"
                  className="px-4 py-1.5 text-xs font-semibold text-white bg-purple-600 hover:bg-purple-700 rounded-lg shadow-sm"
                >
                  Adicionar Relação
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};
