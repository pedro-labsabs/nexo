#!/usr/bin/env node
import { runSync } from './sync.js';

async function main() {
  console.log('==============================================');
  console.log('🚀 NEXO — Observatório de Portfólio GitHub');
  console.log('   Sincronização Local-First de Repositórios');
  console.log('==============================================\n');

  const args = process.argv.slice(2);
  let outputPath: string | undefined;
  let annotationsPath: string | undefined;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === '--out' || args[i] === '-o') {
      outputPath = args[i + 1];
      i++;
    } else if (args[i] === '--annotations' || args[i] === '-a') {
      annotationsPath = args[i + 1];
      i++;
    } else if (args[i] === '--help' || args[i] === '-h') {
      console.log('Uso: npx tsx src/cli/index.ts [opções]\n');
      console.log('Opções:');
      console.log('  -o, --out <caminho>          Caminho do arquivo JSON de snapshot gerado');
      console.log('  -a, --annotations <caminho>  Caminho do arquivo de anotações manuais para mesclar');
      console.log('  -h, --help                   Exibe esta ajuda\n');
      console.log('Variáveis de ambiente:');
      console.log('  GH_TOKEN ou GITHUB_TOKEN     Token pessoal de acesso GitHub (obrigatório)\n');
      process.exit(0);
    }
  }

  try {
    const snapshot = await runSync({ outputPath, annotationsPath });
    console.log('\n✅ Sincronização concluída com sucesso!');
    console.log(`📊 Total de repositórios: ${snapshot.repositories.length}`);
    console.log(`🏢 Total de organizações: ${snapshot.organizations.length}`);
    console.log(`🔗 Total de relacionamentos: ${snapshot.relationships.length}`);
    if (snapshot.metadata.incompleteSync) {
      console.warn('\n⚠️ Aviso: A sincronização foi parcial devido a erros ou permissões limitadas:');
      for (const w of snapshot.metadata.syncWarnings) {
        console.warn(`  - ${w}`);
      }
    }
    console.log('\n💡 Para abrir o dashboard: npm run dev');
  } catch (error: any) {
    console.error('\n❌ Erro durante a sincronização:');
    console.error(error?.message || error);
    process.exit(1);
  }
}

main();
