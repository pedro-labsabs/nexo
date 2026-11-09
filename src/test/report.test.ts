import { describe, it, expect } from 'vitest';
import { generateMarkdownReport } from '../shared/report.js';
import { createDemoSnapshot } from '../demo/generateDemoData.js';

describe('Markdown Report Generation', () => {
  it('generates a complete Markdown report in Portuguese with explicit privacy warning', () => {
    const snapshot = createDemoSnapshot();
    const md = generateMarkdownReport(snapshot);

    expect(md).toContain('# Relatório de Portfólio GitHub — NEXO');
    expect(md).toContain('AVISO DE PRIVACIDADE E SEGURANÇA');
    expect(md).toContain('repositório(s) privado(s)');
    expect(md).toContain('Visão Geral das Organizações');
    expect(md).toContain('Detalhamento Técnico e Indicadores Explicáveis');
    expect(md).toContain('Indicadores Determinísticos');
    expect(md).toContain('Mapeamento de Relacionamentos do Portfólio');
    expect(md).toContain('nexo-core');
  });
});

  it('renders ❓ Desconhecido instead of ❌ Ausente when repository tree is truncated or unknown', () => {
    const snapshot = createDemoSnapshot();
    // Repo 3003 is uninspected tree
    const uninspectedRepo = snapshot.repositories.find((r) => r.id === 3003);
    expect(uninspectedRepo).toBeDefined();

    const md = generateMarkdownReport(snapshot);
    // alexdev/pesquisa-inacessivel should show ❓ Desconhecido for Testes and CI
    const uninspectedLine = md
      .split('\n')
      .find((line) => line.includes('alexdev/pesquisa-inacessivel'));

    expect(uninspectedLine).toBeDefined();
    expect(uninspectedLine).toContain('❓ Desconhecido');
    expect(uninspectedLine).not.toContain('❌ Ausente');
  });
