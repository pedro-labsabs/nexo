import { Octokit } from '@octokit/rest';
import fs from 'node:fs';
import path from 'node:path';
import {
  Repository,
  Organization,
  Snapshot,
  TreeSignals,
  Affiliation,
} from '../shared/types.js';
import { computeIndicators } from '../shared/indicators.js';
import { extractExplicitRelationships } from '../shared/relations.js';
import { validateSnapshot, validateAnnotations } from '../shared/validation.js';
import { mergeManualRelationships } from '../shared/annotations.js';

export interface SyncOptions {
  token?: string;
  outputPath?: string;
  annotationsPath?: string;
}

export async function runSync(options: SyncOptions = {}): Promise<Snapshot> {
  const token = options.token || process.env.GH_TOKEN || process.env.GITHUB_TOKEN;

  if (!token || token.trim().length === 0) {
    throw new Error(
      'Token de autenticação não encontrado. Defina a variável de ambiente GH_TOKEN ou GITHUB_TOKEN.'
    );
  }

  const octokit = new Octokit({
    auth: token,
    userAgent: 'nexo-observatory/1.0',
  });

  console.log('🔍 Autenticando com a API do GitHub...');
  let authenticatedUser = 'unknown';
  try {
    const { data: user } = await octokit.rest.users.getAuthenticated();
    authenticatedUser = user.login;
    console.log(`👤 Usuário autenticado: ${authenticatedUser}`);
  } catch (err: any) {
    throw new Error(`Falha de autenticação fatal no GitHub: ${err?.message || err}`);
  }

  const warnings: string[] = [];
  let incompleteSync = false;

  console.log('📦 Buscando repositórios acessíveis (owner, collaborator, organization_member)...');

  // Paginação completa de repositórios
  const affiliations: Affiliation[] = ['owner', 'collaborator', 'organization_member'];
  const repoMap = new Map<number, { raw: any; affiliation: Affiliation }>();

  for (const aff of affiliations) {
    let page = 1;
    const per_page = 100;
    while (true) {
      try {
        const { data: pageRepos } = await octokit.rest.repos.listForAuthenticatedUser({
          affiliation: aff,
          per_page,
          page,
          sort: 'pushed',
          direction: 'desc',
        });

        if (pageRepos.length === 0) break;

        for (const r of pageRepos) {
          if (!repoMap.has(r.id)) {
            repoMap.set(r.id, { raw: r, affiliation: aff });
          }
        }

        if (pageRepos.length < per_page) break;
        page++;
      } catch (err: any) {
        incompleteSync = true;
        warnings.push(`Erro de paginação ao listar repositórios (${aff}, página ${page}): ${err?.message || err}`);
        break;
      }
    }
  }

  console.log(`✨ Encontrados ${repoMap.size} repositórios únicos.`);

  // Identificação de organizações associadas
  const orgsMap = new Map<number, Organization>();
  const normalizedRepos: Repository[] = [];
  const readmeMap: Record<number, string> = {};
  const manifestMap: Record<number, string> = {};

  const referenceDate = new Date().toISOString();

  let count = 0;
  for (const { raw: r, affiliation } of repoMap.values()) {
    count++;
    console.log(`[${count}/${repoMap.size}] Inspecionando ${r.full_name}...`);

    if (r.owner && r.owner.type === 'Organization') {
      if (!orgsMap.has(r.owner.id)) {
        orgsMap.set(r.owner.id, {
          id: r.owner.id,
          login: r.owner.login,
          name: r.owner.login,
          avatarUrl: r.owner.avatar_url,
          description: null,
          repositoryCount: 0,
        });
      }
      const org = orgsMap.get(r.owner.id)!;
      org.repositoryCount++;
    }

    // Inspeção de árvore de arquivos e detecção determinística de evidências
    const treeSignals = await inspectRepositoryTree(octokit, r.owner.login, r.name, r.default_branch);

    if (!treeSignals.treeInspected) {
      if (!treeSignals.isEmptyRepo) {
        incompleteSync = true;
        warnings.push(`Inspeção de árvore inacessível em ${r.full_name}: ${treeSignals.inspectionError || 'Acesso negado ou erro de API'}`);
      }
    } else if (treeSignals.treeTruncated) {
      incompleteSync = true;
      warnings.push(`Árvore de arquivos truncada em ${r.full_name}: coleta de testes e CI parcial`);
    }

    // Leitura segura do README se existir para relacionamentos
    if (treeSignals.readmeFound && treeSignals.readmePath) {
      try {
        const { data: readmeContent } = await octokit.rest.repos.getReadme({
          owner: r.owner.login,
          repo: r.name,
          mediaType: { format: 'raw' },
        });
        if (typeof readmeContent === 'string') {
          readmeMap[r.id] = readmeContent;
        } else {
          incompleteSync = true;
          warnings.push(`Falha na leitura do README em ${r.full_name}: resposta em formato inesperado`);
        }
      } catch (err: any) {
        incompleteSync = true;
        warnings.push(`Falha na leitura do README em ${r.full_name}: ${err?.message || err}`);
      }
    }

    // Leitura segura de manifesto package.json na raiz se detectado
    if (treeSignals.packageJsonFound) {
      try {
        const { data: packageJsonContent } = await octokit.rest.repos.getContent({
          owner: r.owner.login,
          repo: r.name,
          path: 'package.json',
          mediaType: { format: 'raw' },
        });
        if (typeof packageJsonContent === 'string') {
          manifestMap[r.id] = packageJsonContent;
        } else if ((packageJsonContent as any)?.content) {
          manifestMap[r.id] = Buffer.from((packageJsonContent as any).content, 'base64').toString('utf-8');
        }
      } catch (err: any) {
        incompleteSync = true;
        warnings.push(`Falha na leitura do manifesto package.json em ${r.full_name}: ${err?.message || err}`);
      }
    }

    const repoObj: Repository = {
      id: r.id,
      nodeId: r.node_id,
      name: r.name,
      fullName: r.full_name,
      owner: {
        login: r.owner.login,
        id: r.owner.id,
        avatarUrl: r.owner.avatar_url,
        type: r.owner.type as 'User' | 'Organization',
      },
      isPrivate: Boolean(r.private),
      htmlUrl: r.html_url,
      description: r.description || null,
      fork: Boolean(r.fork),
      createdAt: r.created_at,
      updatedAt: r.updated_at,
      pushedAt: r.pushed_at || r.updated_at,
      sizeKb: r.size || 0,
      stargazersCount: r.stargazers_count || 0,
      watchersCount: r.watchers_count || 0,
      openIssuesCount: r.open_issues_count || 0,
      forksCount: r.forks_count || 0,
      primaryLanguage: r.language || null,
      languages: r.language ? { [r.language]: 1 } : {},
      archived: Boolean(r.archived),
      disabled: Boolean(r.disabled),
      defaultBranch: r.default_branch || 'main',
      affiliation,
      treeSignals,
    };

    repoObj.indicators = computeIndicators(repoObj, referenceDate);
    normalizedRepos.push(repoObj);
  }

  const organizations = Array.from(orgsMap.values());
  const automaticRelationships = extractExplicitRelationships(
    normalizedRepos,
    organizations,
    readmeMap,
    manifestMap
  );

  const finalRelationships = [...automaticRelationships];

  // Reaplica anotações e relações manuais locais se arquivo de anotações existir
  let userAnnotations = undefined;
  if (options.annotationsPath && fs.existsSync(options.annotationsPath)) {
    try {
      const rawAnno = JSON.parse(fs.readFileSync(options.annotationsPath, 'utf-8'));
      const annoValid = validateAnnotations(rawAnno);
      if (annoValid.success) {
        userAnnotations = annoValid.data;
        // Mescla relações manuais usando algoritmo determinístico com tombstones
        const { mergedRelationships } = mergeManualRelationships(
          [],
          userAnnotations.manualRelationships || [],
          userAnnotations.deletedRelationships || [],
          []
        );
        for (const manualRel of mergedRelationships) {
          finalRelationships.push(manualRel);
        }
        console.log(`📌 Anotações locais aplicadas com sucesso (${mergedRelationships.length} relações manuais ativas).`);
      } else {
        incompleteSync = true;
        warnings.push(`Falha ao validar anotações existentes: ${annoValid.errors.join('; ')}`);
      }
    } catch (err: any) {
      incompleteSync = true;
      warnings.push(`Erro ao carregar anotações locais: ${err?.message || err}`);
    }
  }

  const snapshot: Snapshot = {
    metadata: {
      version: '1.0',
      createdAt: new Date().toISOString(),
      referenceDate,
      authenticatedUser,
      totalRepositories: normalizedRepos.length,
      incompleteSync,
      syncWarnings: warnings,
    },
    organizations,
    repositories: normalizedRepos,
    relationships: finalRelationships,
    userAnnotations,
  };

  const validation = validateSnapshot(snapshot);
  if (!validation.success) {
    throw new Error(`Snapshot gerado falhou na validação de schema: ${validation.errors.join(', ')}`);
  }

  const outPath = options.outputPath || path.resolve(`snapshots/nexo-snapshot-${Date.now()}.json`);
  const dir = path.dirname(outPath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }

  fs.writeFileSync(outPath, JSON.stringify(snapshot, null, 2), 'utf-8');
  console.log(`💾 Snapshot salvo com sucesso em: ${outPath}`);

  return snapshot;
}

export async function inspectRepositoryTree(
  octokit: Octokit,
  owner: string,
  repo: string,
  defaultBranch = 'main'
): Promise<TreeSignals> {
  const result: TreeSignals = {
    hasTests: false,
    testEvidence: [],
    hasCI: false,
    ciEvidence: [],
    hasDocumentation: false,
    documentationEvidence: [],
    readmeFound: false,
    contributingFound: false,
    licenseFileFound: false,
    packageJsonFound: false,
    isEmptyRepo: false,
    treeTruncated: false,
    treeInspected: false,
  };

  try {
    const { data: treeData } = await octokit.rest.git.getTree({
      owner,
      repo,
      tree_sha: defaultBranch,
      recursive: 'true',
    });

    result.treeInspected = true;
    result.treeTruncated = Boolean(treeData.truncated);

    if (!Array.isArray(treeData.tree)) {
      return result;
    }

    if (treeData.tree.length === 0) {
      result.isEmptyRepo = true;
      return result;
    }

    for (const item of treeData.tree) {
      const p = (item.path || '').toLowerCase();

      // Manifest package.json check (na raiz)
      if (p === 'package.json') {
        result.packageJsonFound = true;
      }

      // Test evidence patterns
      if (
        p.startsWith('test/') ||
        p.startsWith('tests/') ||
        p.startsWith('__tests__/') ||
        p.startsWith('spec/') ||
        p.includes('.test.') ||
        p.includes('.spec.') ||
        p.includes('_test.go') ||
        p.includes('_test.rs') ||
        p.includes('vitest.config') ||
        p.includes('jest.config') ||
        p.includes('pytest.ini')
      ) {
        result.hasTests = true;
        if (result.testEvidence.length < 5 && item.path) {
          result.testEvidence.push(item.path);
        }
      }

      // CI evidence patterns
      if (p.startsWith('.github/workflows/') || p.includes('.circleci/') || p.includes('.travis.yml') || p.includes('gitlab-ci.yml')) {
        result.hasCI = true;
        if (result.ciEvidence.length < 5 && item.path) {
          result.ciEvidence.push(item.path);
        }
      }

      // Documentation evidence patterns
      if (p === 'readme.md' || p.startsWith('readme')) {
        result.readmeFound = true;
        result.readmePath = item.path;
      }
      if (p.startsWith('contributing')) {
        result.contributingFound = true;
      }
      if (p.startsWith('license') || p.startsWith('licence')) {
        result.licenseFileFound = true;
      }
      if (p.startsWith('docs/') || p.startsWith('doc/') || p.endsWith('.md')) {
        result.hasDocumentation = true;
        if (result.documentationEvidence.length < 5 && item.path) {
          result.documentationEvidence.push(item.path);
        }
      }
    }

    if (result.readmeFound) {
      result.hasDocumentation = true;
    }

    return result;
  } catch (err: any) {
    result.treeInspected = false;
    const msg = err?.message || String(err);
    result.inspectionError = msg;

    // Repositório vazio no GitHub gera 409 "Git Repository is empty" ou 404 para branch
    if (msg.includes('Git Repository is empty') || err?.status === 409) {
      result.isEmptyRepo = true;
    }

    return result;
  }
}
