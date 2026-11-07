import { Snapshot } from './types.js';

export function generateMarkdownReport(snapshot: Snapshot): string {
  const meta = snapshot.metadata;
  const repos = snapshot.repositories;
  const orgs = snapshot.organizations;
  const rels = snapshot.relationships;
  const annotations = snapshot.userAnnotations?.potentialAssessments || {};

  const totalPublic = repos.filter((r) => !r.isPrivate).length;
  const totalPrivate = repos.filter((r) => r.isPrivate).length;

  let md = `# Relatório de Portfólio GitHub — NEXO\n\n`;

  // Aviso explícito sobre dados privados
  if (totalPrivate > 0) {
    md += `> ⚠️ **AVISO DE PRIVACIDADE E SEGURANÇA:**\n`;
    md += `> Este relatório contém dados derivados de **${totalPrivate} repositório(s) privado(s)**.\n`;
    md += `> Verifique atentamente se não há nomes de projetos internos, caminhos confidenciais ou metadados sensíveis antes de compartilhar ou publicar este documento publicamente.\n\n`;
  }

  md += `## Informações Gerais da Execução\n\n`;
  md += `- **Data do Relatório / Referência:** ${new Date(meta.referenceDate).toLocaleDateString('pt-BR')}\n`;
  md += `- **Usuário Autenticado:** @${meta.authenticatedUser}\n`;
  md += `- **Total de Repositórios Analisados:** ${repos.length} (${totalPublic} públicos, ${totalPrivate} privados)\n`;
  md += `- **Total de Organizações Vinculadas:** ${orgs.length}\n`;
  md += `- **Total de Relacionamentos Registrados:** ${rels.length}\n`;
  if (meta.incompleteSync) {
    md += `- **Status de Sincronização:** Parcial / Incompleta\n`;
    md += `  - Avisos registrados: ${meta.syncWarnings.join('; ')}\n`;
  } else {
    md += `- **Status de Sincronização:** Completa\n`;
  }
  md += `\n---\n\n`;

  md += `## Visão Geral das Organizações\n\n`;
  if (orgs.length === 0) {
    md += `*Nenhuma organização vinculada identificada.*\n\n`;
  } else {
    md += `| Organização | Repositórios Vinculados | Descrição |\n`;
    md += `| :--- | :---: | :--- |\n`;
    for (const org of orgs) {
      md += `| **${org.login}** | ${org.repositoryCount} | ${org.description || '—'} |\n`;
    }
    md += `\n`;
  }

  md += `## Sumário Executivo de Repositórios\n\n`;
  md += `| Repositório | Visibilidade | Linguagem | Testes | CI | Escore Qualidade | Último Push |\n`;
  md += `| :--- | :---: | :---: | :---: | :---: | :---: | :---: |\n`;
  for (const repo of repos) {
    const hasTests = repo.treeSignals.hasTests ? '✅ Presente' : '❌ Ausente';
    const hasCI = repo.treeSignals.hasCI ? '✅ Presente' : '❌ Ausente';
    const score = repo.indicators ? `${repo.indicators.overallQualityScore}/100` : 'N/A';
    const pushDate = new Date(repo.pushedAt).toLocaleDateString('pt-BR');
    const vis = repo.isPrivate ? '🔒 Privado' : '🌐 Público';
    md += `| **${repo.fullName}** | ${vis} | ${repo.primaryLanguage || '—'} | ${hasTests} | ${hasCI} | ${score} | ${pushDate} |\n`;
  }
  md += `\n---\n\n`;

  md += `## Detalhamento Técnico e Indicadores Explicáveis\n\n`;
  for (const repo of repos) {
    md += `### ${repo.fullName}\n\n`;
    md += `- **URL:** ${repo.htmlUrl}\n`;
    md += `- **Descrição:** ${repo.description || 'Sem descrição cadastrada'}\n`;
    md += `- **Tamanho:** ${repo.sizeKb} KB | **Estrelas:** ${repo.stargazersCount} | **Issues Abertas:** ${repo.openIssuesCount}\n`;
    md += `- **Branch Padrão:** \`${repo.defaultBranch}\`\n\n`;

    if (repo.indicators) {
      md += `#### Indicadores Determinísticos:\n\n`;
      const ind = repo.indicators;
      md += `1. **${ind.testInfrastructure.name}** (${ind.testInfrastructure.status.toUpperCase()} - ${ind.testInfrastructure.score}/100):\n`;
      md += `   - *Resumo:* ${ind.testInfrastructure.summary}\n`;
      md += `   - *Limitação declarada:* ${ind.testInfrastructure.limitations[0]}\n`;
      md += `2. **${ind.ciConfiguration.name}** (${ind.ciConfiguration.status.toUpperCase()} - ${ind.ciConfiguration.score}/100):\n`;
      md += `   - *Resumo:* ${ind.ciConfiguration.summary}\n`;
      md += `3. **${ind.documentation.name}** (${ind.documentation.status.toUpperCase()} - ${ind.documentation.score}/100):\n`;
      md += `   - *Resumo:* ${ind.documentation.summary}\n`;
      md += `4. **${ind.activityRecency.name}** (${ind.activityRecency.status.toUpperCase()} - ${ind.activityRecency.score}/100):\n`;
      md += `   - *Resumo:* ${ind.activityRecency.summary}\n`;
      md += `5. **${ind.maintenanceSignals.name}** (${ind.maintenanceSignals.status.toUpperCase()} - ${ind.maintenanceSignals.score}/100):\n`;
      md += `   - *Resumo:* ${ind.maintenanceSignals.summary}\n\n`;
    }

    const anno = annotations[String(repo.id)];
    if (anno) {
      md += `#### Avaliação de Potencial (Anotação Manual do Usuário):\n\n`;
      md += `> *Nota de transparência: Esta seção não deriva de métricas do GitHub e foi preenchida pelo avaliador.*\n\n`;
      md += `- **Valor Estratégico:** ${anno.strategicValue.toUpperCase()}\n`;
      md += `- **Potencial Comercial:** ${anno.commercialPotential.toUpperCase()}\n`;
      if (anno.targetAudience) md += `- **Público-Alvo:** ${anno.targetAudience}\n`;
      if (anno.tags && anno.tags.length > 0) md += `- **Tags:** \`${anno.tags.join('`, `')}\`\n`;
      if (anno.notes) md += `- **Notas:** ${anno.notes}\n`;
      md += `\n`;
    }

    md += `---\n\n`;
  }

  md += `## Mapeamento de Relacionamentos do Portfólio\n\n`;
  if (rels.length === 0) {
    md += `*Nenhum relacionamento registrado.*\n\n`;
  } else {
    md += `| Origem (ID) | Destino (ID) | Tipo | Proveniência | Verificado | Evidência / Descrição |\n`;
    md += `| :--- | :--- | :---: | :---: | :---: | :--- |\n`;
    for (const rel of rels) {
      const prov = rel.provenance === 'manual' ? 'Manual' : 'Automática';
      const ver = rel.verified ? 'Sim' : 'Não';
      md += `| ${rel.source} | ${rel.target} | \`${rel.type}\` | ${prov} | ${ver} | ${rel.evidence || rel.description || '—'} |\n`;
    }
    md += `\n`;
  }

  md += `\n*Relatório gerado automaticamente pelo NEXO — Local-First GitHub Observatory.*\n`;

  return md;
}
