import { Repository, RepositoryIndicators, IndicatorDetail, TreeSignals } from './types.js';

export function computeIndicators(
  repo: Repository,
  referenceDateStr: string = new Date().toISOString()
): RepositoryIndicators {
  const refDate = new Date(referenceDateStr).getTime();
  const tree = repo.treeSignals;

  // 1. Infraestrutura de Testes
  const testInfra = computeTestInfrastructure(tree);

  // 2. Configuração de CI
  const ciConfig = computeCIConfiguration(tree);

  // 3. Documentação
  const doc = computeDocumentation(tree, repo);

  // 4. Recência de Atividade
  const activity = computeActivityRecency(repo, refDate);

  // 5. Sinais Básicos de Manutenção
  const maintenance = computeMaintenanceSignals(repo, tree);

  // Overall Quality Score (ponderada, determinística)
  // Pesos: Testes 25%, CI 20%, Documentação 25%, Atividade 15%, Manutenção 15%
  let totalScore = 0;
  let totalWeight = 0;

  if (testInfra.status !== 'unknown') {
    totalScore += testInfra.score * 0.25;
    totalWeight += 0.25;
  }
  if (ciConfig.status !== 'unknown') {
    totalScore += ciConfig.score * 0.20;
    totalWeight += 0.20;
  }
  if (doc.status !== 'unknown') {
    totalScore += doc.score * 0.25;
    totalWeight += 0.25;
  }
  if (activity.status !== 'unknown') {
    totalScore += activity.score * 0.15;
    totalWeight += 0.15;
  }
  if (maintenance.status !== 'unknown') {
    totalScore += maintenance.score * 0.15;
    totalWeight += 0.15;
  }

  // Se nenhum indicador pôde ser avaliado com certeza (peso == 0), escore é 0 com status unknown
  const overallQualityScore = totalWeight > 0 ? Math.round(totalScore / totalWeight) : 0;

  return {
    testInfrastructure: testInfra,
    ciConfiguration: ciConfig,
    documentation: doc,
    activityRecency: activity,
    maintenanceSignals: maintenance,
    overallQualityScore,
  };
}

function computeTestInfrastructure(tree: TreeSignals): IndicatorDetail {
  if (!tree.treeInspected) {
    return {
      id: 'test_infrastructure',
      name: 'Infraestrutura de Testes',
      status: 'unknown',
      score: 0,
      summary: 'Árvore de arquivos não inspecionada ou inacessível via API.',
      inputs: { treeInspected: false, inspectionError: tree.inspectionError || null },
      calculation: 'Nenhum dado de arquivo disponível para inspeção.',
      limitations: [
        'A ausência de acesso à árvore impede checar arquivos de testes.',
        'Não foi presumido resultado negativo.',
      ],
    };
  }

  const count = tree.testEvidence.length;

  // Se a árvore foi truncada e nenhum teste foi detectado nos arquivos parciais:
  // a ausência NÃO é comprovada, portanto deve produzir status 'unknown'.
  if (tree.treeTruncated && !tree.hasTests) {
    return {
      id: 'test_infrastructure',
      name: 'Infraestrutura de Testes',
      status: 'unknown',
      score: 0,
      summary: 'Inspeção parcial: a árvore de arquivos foi truncada pelo GitHub e arquivos de teste podem ter sido omitidos.',
      inputs: {
        hasTests: false,
        treeTruncated: true,
        evidenceFound: [],
      },
      calculation: 'Informação inconclusiva/desconhecida devido a truncamento de árvore.',
      limitations: [
        'A árvore de arquivos ultrapassou o limite do GitHub e foi truncada.',
        'A ausência de testes não é comprovada; arquivos em diretórios profundos podem existir.',
      ],
    };
  }

  let status: 'passed' | 'failed' | 'partial' = 'failed';
  let score = 0;
  let summary = 'Nenhuma configuração ou diretório de testes detectado na árvore inspecionada.';

  if (tree.hasTests && count >= 2) {
    status = 'passed';
    score = 100;
    summary = `Detectados múltiplos sinais de testes: ${tree.testEvidence.slice(0, 3).join(', ')}.`;
  } else if (tree.hasTests && count === 1) {
    status = 'partial';
    score = 70;
    summary = `Detectado sinal de teste: ${tree.testEvidence[0]}.`;
  }

  const limitations = [
    'A presença de arquivos ou pastas de teste NÃO garante que os testes foram executados ou que estejam passando.',
  ];
  if (tree.treeTruncated) {
    limitations.push('Árvore de arquivos truncada: evidência observada parcialmente.');
  }

  return {
    id: 'test_infrastructure',
    name: 'Infraestrutura de Testes',
    status,
    score,
    summary,
    inputs: {
      hasTests: tree.hasTests,
      evidenceFound: tree.testEvidence,
      treeTruncated: tree.treeTruncated,
    },
    calculation: 'Escore de 100 para >=2 evidências, 70 para 1 evidência, 0 para nenhuma.',
    limitations,
  };
}

function computeCIConfiguration(tree: TreeSignals): IndicatorDetail {
  if (!tree.treeInspected) {
    return {
      id: 'ci_configuration',
      name: 'Configuração de CI/CD',
      status: 'unknown',
      score: 0,
      summary: 'Árvore de arquivos não inspecionada ou inacessível.',
      inputs: { treeInspected: false },
      calculation: 'Nenhum dado de arquivo disponível.',
      limitations: ['Sem acesso aos arquivos do repositório para verificar diretórios de workflows.'],
    };
  }

  // Se a árvore foi truncada e nenhum CI foi detectado:
  // a ausência de CI NÃO é comprovada.
  if (tree.treeTruncated && !tree.hasCI) {
    return {
      id: 'ci_configuration',
      name: 'Configuração de CI/CD',
      status: 'unknown',
      score: 0,
      summary: 'Inspeção parcial: árvore truncada e diretórios de CI podem ter sido omitidos da resposta.',
      inputs: {
        hasCI: false,
        treeTruncated: true,
      },
      calculation: 'Informação inconclusiva devido ao truncamento da árvore.',
      limitations: [
        'A ausência de fluxos não é comprovada em árvores truncadas pelo GitHub.',
        'Sistemas externos de CI também podem não ter sido verificados.',
      ],
    };
  }

  let status: 'passed' | 'failed' | 'partial' = 'failed';
  let score = 0;
  let summary = 'Nenhum arquivo de automação CI/CD (.github/workflows, etc.) encontrado.';

  if (tree.hasCI) {
    status = 'passed';
    score = 100;
    summary = `Fluxos de automação CI configurados: ${tree.ciEvidence.join(', ')}.`;
  }

  const limitations = [
    'A existência de workflows não assegura que as execuções de CI estejam com status verde no GitHub.',
    'Sistemas externos de CI (ex: CircleCI externo ou Jenkins sem manifesto no repo) podem não ser identificados.',
  ];
  if (tree.treeTruncated) {
    limitations.push('Árvore de arquivos truncada: evidência observada nos arquivos parciais.');
  }

  return {
    id: 'ci_configuration',
    name: 'Configuração de CI/CD',
    status,
    score,
    summary,
    inputs: {
      hasCI: tree.hasCI,
      ciEvidence: tree.ciEvidence,
      treeTruncated: tree.treeTruncated,
    },
    calculation: 'Escore de 100 para presença de arquivos de workflow CI detectados, 0 caso contrário.',
    limitations,
  };
}

function computeDocumentation(tree: TreeSignals, repo: Repository): IndicatorDetail {
  const hasDesc = Boolean(repo.description && repo.description.trim().length > 5);
  const inputs = {
    hasDescription: hasDesc,
    readmeFound: tree.readmeFound,
    contributingFound: tree.contributingFound,
    licenseFileFound: tree.licenseFileFound,
    docEvidence: tree.documentationEvidence,
    treeTruncated: tree.treeTruncated,
  };

  if (!tree.treeInspected) {
    return {
      id: 'documentation',
      name: 'Documentação do Projeto',
      status: 'unknown',
      score: hasDesc ? 40 : 0,
      summary: hasDesc
        ? 'Descrição básica presente, mas arquivos de documentação não puderam ser verificados.'
        : 'Metadados de documentação inacessíveis.',
      inputs,
      calculation: 'Baseado apenas na descrição do repositório devido a árvore inacessível.',
      limitations: ['Árvore de arquivos inacessível; checagem de README e guias não pôde ser executada.'],
    };
  }

  let points = 0;
  if (hasDesc) points += 20;
  if (tree.readmeFound) points += 40;
  if (tree.documentationEvidence.length > 0) points += 20;
  if (tree.contributingFound) points += 10;
  if (tree.licenseFileFound) points += 10;

  const score = Math.min(100, points);

  // Se a árvore foi truncada e o escore for baixo (< 40) porque arquivos de documentação
  // podem ter sido omitidos na truncagem (por exemplo, subdiretórios de documentação):
  let status: 'passed' | 'partial' | 'failed' | 'unknown' = 'failed';
  if (score >= 70) status = 'passed';
  else if (score >= 40) status = 'partial';
  else if (tree.treeTruncated) status = 'unknown';

  const limitations = [
    'A presença de README.md não garante completude, clareza ou qualidade da documentação técnica.',
    'Não avalia compreensão semântica do conteúdo textual.',
  ];
  if (tree.treeTruncated) {
    limitations.push('Árvore de arquivos truncada: documentação em subpastas pode ter sido omitida.');
  }

  return {
    id: 'documentation',
    name: 'Documentação do Projeto',
    status,
    score,
    summary: `Escore de documentação: ${score}/100 (README: ${tree.readmeFound ? 'Sim' : 'Não'}, Descrição: ${hasDesc ? 'Sim' : 'Não'}).`,
    inputs,
    calculation: 'README (+40), Descrição (+20), Docs adicionais (+20), CONTRIBUTING (+10), LICENSE (+10).',
    limitations,
  };
}

function computeActivityRecency(repo: Repository, refTimestamp: number): IndicatorDetail {
  const lastPush = repo.pushedAt ? new Date(repo.pushedAt).getTime() : 0;
  if (!lastPush || isNaN(lastPush)) {
    return {
      id: 'activity_recency',
      name: 'Recência de Atividade',
      status: 'unknown',
      score: 0,
      summary: 'Data de push inválida ou não informada.',
      inputs: { pushedAt: repo.pushedAt },
      calculation: 'Sem data de referência disponível.',
      limitations: ['Impossível calcular a recência sem carimbo de data confiável.'],
    };
  }

  const diffMs = Math.max(0, refTimestamp - lastPush);
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

  let score = 0;
  let status: 'passed' | 'partial' | 'failed' = 'failed';
  let summary = '';

  if (diffDays <= 30) {
    score = 100;
    status = 'passed';
    summary = `Atividade muito recente: último push há ${diffDays} dias.`;
  } else if (diffDays <= 90) {
    score = 80;
    status = 'passed';
    summary = `Atividade recente: último push há ${diffDays} dias.`;
  } else if (diffDays <= 180) {
    score = 60;
    status = 'partial';
    summary = `Atividade moderada: último push há ${diffDays} dias (~${Math.round(diffDays / 30)} meses).`;
  } else if (diffDays <= 365) {
    score = 40;
    status = 'partial';
    summary = `Atividade baixa: último push há ${diffDays} dias (~${Math.round(diffDays / 30)} meses).`;
  } else {
    score = 15;
    status = 'failed';
    summary = `Repositório inativo há mais de 1 ano (${diffDays} dias desde o último push).`;
  }

  return {
    id: 'activity_recency',
    name: 'Recência de Atividade',
    status,
    score,
    summary,
    inputs: {
      pushedAt: repo.pushedAt,
      referenceDate: new Date(refTimestamp).toISOString(),
      daysSinceLastPush: diffDays,
    },
    calculation: '<=30d: 100, <=90d: 80, <=180d: 60, <=365d: 40, >365d: 15.',
    limitations: [
      'A recência de push mede frequência temporal, não produtividade, estabilidade ou valor do código.',
      'Projetos maduros e estáveis podem ter baixa atividade recente propositalmente.',
    ],
  };
}

function computeMaintenanceSignals(repo: Repository, tree: TreeSignals): IndicatorDetail {
  const inputs = {
    archived: repo.archived,
    disabled: repo.disabled,
    openIssuesCount: repo.openIssuesCount,
    sizeKb: repo.sizeKb,
    defaultBranch: repo.defaultBranch,
  };

  if (repo.archived) {
    return {
      id: 'maintenance_signals',
      name: 'Sinais de Manutenção',
      status: 'failed',
      score: 10,
      summary: 'Repositório arquivado (somente leitura).',
      inputs,
      calculation: 'Repositórios arquivados recebem pontuação 10.',
      limitations: [
        'O arquivamento sinaliza encerramento formal de manutenção no GitHub.',
      ],
    };
  }

  let score = 80; // Baseline saudável
  if (repo.disabled) score = 0;
  if (repo.sizeKb === 0) score -= 20; // repositório vazio

  // Ajuste por branch padrão moderna (main)
  if (repo.defaultBranch === 'main') score += 10;
  if (tree.licenseFileFound) score += 10;

  score = Math.min(100, Math.max(0, score));
  const status: 'passed' | 'partial' = score >= 70 ? 'passed' : 'partial';

  return {
    id: 'maintenance_signals',
    name: 'Sinais de Manutenção',
    status,
    score,
    summary: `Sinais estruturais e operacionais ativos (Branch: ${repo.defaultBranch}, Issues: ${repo.openIssuesCount}).`,
    inputs,
    calculation: 'Base 80, bônus para branch main (+10) e licença (+10), penalidades para repo vazio ou desativado.',
    limitations: [
      'A quantidade de issues abertas NÃO é prova isolada de baixa qualidade (projetos populares recebem muitas issues).',
      'Tamanho em KB não reflete a complexidade ou elegância da arquitetura.',
    ],
  };
}
