import React from 'react';
import { Search, Filter, ArrowUpDown } from 'lucide-react';
import { Organization } from '../shared/types.js';

export interface FilterState {
  search: string;
  organization: string;
  visibility: 'all' | 'public' | 'private';
  language: string;
  affiliation: 'all' | 'owner' | 'collaborator' | 'organization_member';
  sortBy: 'pushed' | 'stars' | 'size' | 'score' | 'name';
  sortOrder: 'asc' | 'desc';
}

interface FiltersBarProps {
  filters: FilterState;
  onChange: (filters: FilterState) => void;
  organizations: Organization[];
  availableLanguages: string[];
}

export const FiltersBar: React.FC<FiltersBarProps> = ({
  filters,
  onChange,
  organizations,
  availableLanguages,
}) => {
  return (
    <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 p-4 rounded-xl shadow-sm mb-6 transition-colors">
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-3">
        {/* Search */}
        <div className="relative">
          <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input
            type="text"
            placeholder="Buscar repositório por nome ou descrição..."
            value={filters.search}
            onChange={(e) => onChange({ ...filters, search: e.target.value })}
            className="w-full pl-9 pr-3 py-2 text-xs bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 text-slate-900 dark:text-slate-100 placeholder-slate-400"
          />
        </div>

        {/* Organization */}
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-slate-400 shrink-0" />
          <select
            value={filters.organization}
            onChange={(e) => onChange({ ...filters, organization: e.target.value })}
            className="w-full py-2 px-2.5 text-xs bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 text-slate-900 dark:text-slate-100"
          >
            <option value="all">Todas as organizações</option>
            {organizations.map((org) => (
              <option key={org.id} value={org.login}>
                {org.name || org.login}
              </option>
            ))}
          </select>
        </div>

        {/* Language & Visibility */}
        <div className="grid grid-cols-2 gap-2">
          <select
            value={filters.language}
            onChange={(e) => onChange({ ...filters, language: e.target.value })}
            className="w-full py-2 px-2 text-xs bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 text-slate-900 dark:text-slate-100"
          >
            <option value="all">Todas as linguagens</option>
            {availableLanguages.map((lang) => (
              <option key={lang} value={lang}>
                {lang}
              </option>
            ))}
          </select>

          <select
            value={filters.visibility}
            onChange={(e) => onChange({ ...filters, visibility: e.target.value as any })}
            className="w-full py-2 px-2 text-xs bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 text-slate-900 dark:text-slate-100"
          >
            <option value="all">Visibilidade (Todas)</option>
            <option value="public">Públicos</option>
            <option value="private">Privados</option>
          </select>
        </div>

        {/* Sorting */}
        <div className="flex items-center gap-2">
          <ArrowUpDown className="w-4 h-4 text-slate-400 shrink-0" />
          <select
            value={filters.sortBy}
            onChange={(e) => onChange({ ...filters, sortBy: e.target.value as any })}
            className="w-full py-2 px-2.5 text-xs bg-slate-50 dark:bg-slate-800 border border-slate-200 dark:border-slate-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-indigo-500 text-slate-900 dark:text-slate-100"
          >
            <option value="pushed">Ordenar por Última Atividade</option>
            <option value="score">Ordenar por Escore de Qualidade</option>
            <option value="stars">Ordenar por Estrelas</option>
            <option value="size">Ordenar por Tamanho (KB)</option>
            <option value="name">Ordenar por Nome</option>
          </select>

          <button
            onClick={() => onChange({ ...filters, sortOrder: filters.sortOrder === 'asc' ? 'desc' : 'asc' })}
            className="px-2.5 py-2 text-xs bg-slate-50 dark:bg-slate-800 hover:bg-slate-100 dark:hover:bg-slate-700 border border-slate-200 dark:border-slate-700 rounded-lg text-slate-700 dark:text-slate-200 font-semibold"
            title="Inverter ordem"
          >
            {filters.sortOrder === 'asc' ? '↑ Cresc.' : '↓ Decresc.'}
          </button>
        </div>
      </div>
    </div>
  );
};
