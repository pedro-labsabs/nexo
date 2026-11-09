import { Repository, Relationship, Organization } from './types.js';

export function extractExplicitRelationships(
  repos: Repository[],
  orgs: Organization[],
  readmeContentsMap: Record<number, string> = {},
  manifestContentsMap: Record<number, string> = {}
): Relationship[] {
  const relationships: Relationship[] = [];
  const knownRepoFullNames = new Map<string, Repository>();

  for (const repo of repos) {
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

  // 2. Relações Verificadas entre Repositórios a partir de documentação (README)
  for (const repo of repos) {
    const readme = readmeContentsMap[repo.id];
    if (!readme) continue;

    for (const [targetFullName, targetRepo] of knownRepoFullNames.entries()) {
      if (targetRepo.id === repo.id) continue;

      const urlPattern = new RegExp(`github\\.com/${escapeRegex(targetFullName)}(\\b|/|#)`, 'i');
      const markdownLinkPattern = new RegExp(`\\[[^\\]]+\\]\\([^)]*${escapeRegex(targetFullName)}[^)]*\\)`, 'i');

      if (urlPattern.test(readme) || markdownLinkPattern.test(readme)) {
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
          createdAt: repo.createdAt || new Date().toISOString(),
        });
      }
    }
  }

  // 3. Relações Verificadas entre Repositórios a partir de manifestos (ex: package.json)
  // REGRA ESTRITA: apenas referências explícitas verificáveis (URLs github/git), sem inferir por nome do pacote
  for (const repo of repos) {
    const manifestStr = manifestContentsMap[repo.id];
    if (!manifestStr) continue;

    try {
      const manifest = JSON.parse(manifestStr);
      const depSections = [
        manifest.dependencies,
        manifest.devDependencies,
        manifest.peerDependencies,
        manifest.optionalDependencies,
      ];

      for (const deps of depSections) {
        if (!deps || typeof deps !== 'object') continue;

        for (const [depPkgName, depTarget] of Object.entries(deps)) {
          if (typeof depTarget !== 'string') continue;

          // Procura URL GitHub ou specifier git explícito apontando para repositórios conhecidos do portfólio
          for (const [targetFullName, targetRepo] of knownRepoFullNames.entries()) {
            if (targetRepo.id === repo.id) continue;

            const targetOwnerRepoRegex = new RegExp(
              `(github\\.com[/:#]|git\\+https?://github\\.com/|github:)${escapeRegex(targetFullName)}(\\.git)?(#|/|$)`,
              'i'
            );

            if (targetOwnerRepoRegex.test(depTarget)) {
              const relId = `rel_dep_${repo.id}_${targetRepo.id}`;
              // Evita duplicar se declarado em mais de uma seção de deps
              if (!relationships.some((r) => r.id === relId)) {
                relationships.push({
                  id: relId,
                  source: String(repo.id),
                  target: String(targetRepo.id),
                  sourceType: 'repository',
                  targetType: 'repository',
                  type: 'depends_on',
                  provenance: 'automatic',
                  verified: true,
                  evidence: `package.json: dependência "${depPkgName}": "${depTarget}" aponta inequivocamente para ${targetRepo.fullName}`,
                  description: `Dependência declarada em manifesto (${depPkgName})`,
                  createdAt: repo.createdAt || new Date().toISOString(),
                });
              }
            }
          }
        }
      }
    } catch {
      // Manifest parsing errors are handled and warned during sync
    }
  }

  return relationships;
}

function escapeRegex(string: string) {
  return string.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}
