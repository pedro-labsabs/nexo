import { Repository, Relationship, Organization } from './types.js';

export function extractExplicitRelationships(
  repos: Repository[],
  orgs: Organization[],
  readmeContentsMap: Record<number, string> = {}
): Relationship[] {
  const relationships: Relationship[] = [];
  const knownRepoNames = new Map<string, Repository>();
  const knownRepoFullNames = new Map<string, Repository>();

  for (const repo of repos) {
    knownRepoNames.set(repo.name.toLowerCase(), repo);
    knownRepoFullNames.set(repo.fullName.toLowerCase(), repo);
  }

  // 1. Relações de Pertencimento (belongs_to): Repo -> Org/Owner
  for (const repo of repos) {
    const ownerLogin = repo.owner.login;
    const orgMatch = orgs.find((o) => o.login.toLowerCase() === ownerLogin.toLowerCase());
    if (orgMatch) {
      relationships.push({
        id: `rel_belongs_${repo.id}_${orgMatch.id}`,
        source: String(repo.id),
        target: String(orgMatch.id),
        sourceType: 'repository',
        targetType: 'organization',
        type: 'belongs_to',
        provenance: 'automatic',
        verified: true,
        evidence: `Repositório ${repo.fullName} pertence à organização ${orgMatch.login}`,
        description: `Membro da organização ${orgMatch.login}`,
        createdAt: repo.createdAt,
      });
    }
  }

  // 2. Relações Verificadas entre Repositórios a partir de documentação (README) e manifestos
  for (const repo of repos) {
    const readme = readmeContentsMap[repo.id];
    if (!readme) continue;

    // Busca referências explícitas no formato github.com/owner/repo ou [link](https://github.com/...)
    // ou menções explícitas de URLs completas para repos conhecidos do portfólio
    for (const [targetFullName, targetRepo] of knownRepoFullNames.entries()) {
      if (targetRepo.id === repo.id) continue;

      // Regex para encontrar menção direta e inequívoca da URL ou fullName
      const urlPattern = new RegExp(`github\\.com/${escapeRegex(targetFullName)}(\\b|/|#)`, 'i');
      const markdownLinkPattern = new RegExp(`\\[[^\\]]+\\]\\([^)]*${escapeRegex(targetFullName)}[^)]*\\)`, 'i');

      if (urlPattern.test(readme) || markdownLinkPattern.test(readme)) {
        // Encontrou link documental inequívoco
        relationships.push({
          id: `rel_ref_${repo.id}_${targetRepo.id}`,
          source: String(repo.id),
          target: String(targetRepo.id),
          sourceType: 'repository',
          targetType: 'repository',
          type: 'references',
          provenance: 'automatic',
          verified: true,
          evidence: `Citação explícita encontrada no README de ${repo.fullName} apontando para ${targetRepo.fullName}`,
          description: `Referenciado em documentação de ${repo.name}`,
          createdAt: new Date().toISOString(),
        });
      }
    }
  }

  return relationships;
}

function escapeRegex(string: string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
