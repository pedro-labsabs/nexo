import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { App } from '../App.js';

// Mock cytoscape for headless DOM environment
vi.mock('cytoscape', () => {
  return {
    default: vi.fn(() => ({
      on: vi.fn(),
      zoom: vi.fn(() => 1),
      fit: vi.fn(),
      destroy: vi.fn(),
    })),
  };
});

describe('Dashboard Component Interactions', () => {
  it('renders overview metrics, title, and initial repositories table', () => {
    render(<App />);

    expect(screen.getByText('NEXO')).toBeInTheDocument();
    expect(screen.getByText('Observatório GitHub')).toBeInTheDocument();
    expect(screen.getByText('nexo-core')).toBeInTheDocument();
    expect(screen.getByText('telemetria-edge')).toBeInTheDocument();
  });

  it('filters repositories when typing in the search bar', () => {
    render(<App />);

    const searchInput = screen.getByPlaceholderText(/Buscar repositório/i);
    fireEvent.change(searchInput, { target: { value: 'telemetria' } });

    expect(screen.getByText('telemetria-edge')).toBeInTheDocument();
    expect(screen.queryByText('protocolo-aberto')).not.toBeInTheDocument();
  });

  it('opens repository detail drawer when clicking on details', () => {
    render(<App />);

    const detailButtons = screen.getAllByText('Detalhes →');
    fireEvent.click(detailButtons[0]);

    expect(screen.getByText('Indicadores Técnicos Explicáveis')).toBeInTheDocument();
    expect(screen.getByText(/Avaliação de Potencial Mantida pelo Usuário/i)).toBeInTheDocument();
  });

  it('renders unknown indicator pills for uninspected or truncated repos', () => {
    render(<App />);

    const matchingRepos = screen.getAllByText(/pesquisa-inacessivel/i);
    expect(matchingRepos.length).toBeGreaterThan(0);

    const testUnknowns = screen.getAllByText(/TEST\?/i);
    expect(testUnknowns.length).toBeGreaterThan(0);
    const ciUnknowns = screen.getAllByText(/CI\?/i);
    expect(ciUnknowns.length).toBeGreaterThan(0);
  });
});
