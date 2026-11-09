import { z } from 'zod';

// Affiliation type for synchronized repos
export const AffiliationSchema = z.enum(['owner', 'collaborator', 'organization_member']);
export type Affiliation = z.infer<typeof AffiliationSchema>;

// Tree inspection signals
export const TreeSignalsSchema = z.object({
  hasTests: z.boolean(),
  testEvidence: z.array(z.string()).default([]),
  hasCI: z.boolean(),
  ciEvidence: z.array(z.string()).default([]),
  hasDocumentation: z.boolean(),
  documentationEvidence: z.array(z.string()).default([]),
  readmeFound: z.boolean(),
  readmePath: z.string().optional(),
  contributingFound: z.boolean(),
  licenseFileFound: z.boolean(),
  packageJsonFound: z.boolean().default(false),
  isEmptyRepo: z.boolean().default(false),
  treeTruncated: z.boolean(),
  treeInspected: z.boolean(),
  inspectionError: z.string().optional(),
});
export type TreeSignals = z.infer<typeof TreeSignalsSchema>;

// Explanatory indicator status
export const IndicatorStatusSchema = z.enum(['passed', 'failed', 'unknown', 'partial']);
export type IndicatorStatus = z.infer<typeof IndicatorStatusSchema>;

export const IndicatorDetailSchema = z.object({
  id: z.string(),
  name: z.string(), // Em português para UI
  status: IndicatorStatusSchema,
  score: z.number().min(0).max(100),
  summary: z.string(),
  inputs: z.record(z.string(), z.any()),
  calculation: z.string(),
  limitations: z.array(z.string()),
});
export type IndicatorDetail = z.infer<typeof IndicatorDetailSchema>;

// Technical indicators report for a repo
export const RepositoryIndicatorsSchema = z.object({
  testInfrastructure: IndicatorDetailSchema,
  ciConfiguration: IndicatorDetailSchema,
  documentation: IndicatorDetailSchema,
  activityRecency: IndicatorDetailSchema,
  maintenanceSignals: IndicatorDetailSchema,
  overallQualityScore: z.number().min(0).max(100),
});
export type RepositoryIndicators = z.infer<typeof RepositoryIndicatorsSchema>;

// Repository Model
export const RepositorySchema = z.object({
  id: z.number(), // Immutable GitHub repository ID
  nodeId: z.string(),
  name: z.string(),
  fullName: z.string(),
  owner: z.object({
    login: z.string(),
    id: z.number(),
    avatarUrl: z.string().optional(),
    type: z.enum(['User', 'Organization']),
  }),
  isPrivate: z.boolean(),
  htmlUrl: z.string(),
  description: z.string().nullable(),
  fork: z.boolean(),
  createdAt: z.string(),
  updatedAt: z.string(),
  pushedAt: z.string(),
  sizeKb: z.number(),
  stargazersCount: z.number(),
  watchersCount: z.number(),
  openIssuesCount: z.number(),
  forksCount: z.number(),
  primaryLanguage: z.string().nullable(),
  languages: z.record(z.string(), z.number()).default({}),
  archived: z.boolean(),
  disabled: z.boolean(),
  defaultBranch: z.string(),
  affiliation: AffiliationSchema,
  treeSignals: TreeSignalsSchema,
  // Computed offline indicators
  indicators: RepositoryIndicatorsSchema.optional(),
});
export type Repository = z.infer<typeof RepositorySchema>;

// Organization summary
export const OrganizationSchema = z.object({
  id: z.number(),
  login: z.string(),
  name: z.string().optional(),
  avatarUrl: z.string().optional(),
  description: z.string().nullable(),
  repositoryCount: z.number(),
});
export type Organization = z.infer<typeof OrganizationSchema>;

// Relationships
export const RelationTypeSchema = z.enum([
  'belongs_to', // Repositório pertence à Organização/Owner
  'references', // Citação explícita em documentação/README
  'depends_on', // Dependência declarada em manifesto (ex: package.json)
  'fork_of',    // Repositório é fork de outro
  'custom',     // Relação manual customizada definida pelo usuário
]);
export type RelationType = z.infer<typeof RelationTypeSchema>;

export const RelationshipProvenanceSchema = z.enum(['automatic', 'manual']);
export type RelationshipProvenance = z.infer<typeof RelationshipProvenanceSchema>;

export const RelationshipSchema = z.object({
  id: z.string(), // ex: "rel_123_456"
  source: z.string(), // ID ou fullName do nó de origem
  target: z.string(), // ID ou fullName do nó de destino
  sourceType: z.enum(['repository', 'organization']),
  targetType: z.enum(['repository', 'organization']),
  type: RelationTypeSchema,
  provenance: RelationshipProvenanceSchema,
  verified: z.boolean(),
  evidence: z.string().optional(), // Detalhes ou trecho que prova o relacionamento
  description: z.string().optional(),
  createdAt: z.string(),
  updatedAt: z.string().optional(), // Para edições e resolução determinística de conflitos
});
export type Relationship = z.infer<typeof RelationshipSchema>;

// Tombstone for deleted manual relationships to prevent resurrection across imports
export const RelationshipTombstoneSchema = z.object({
  id: z.string(),
  source: z.string(),
  target: z.string(),
  type: RelationTypeSchema,
  deletedAt: z.string(),
});
export type RelationshipTombstone = z.infer<typeof RelationshipTombstoneSchema>;

// Optional User-maintained Potential Assessment
export const PotentialAssessmentSchema = z.object({
  repositoryId: z.number(),
  strategicValue: z.enum(['low', 'medium', 'high', 'critical']),
  commercialPotential: z.enum(['none', 'experimental', 'viable', 'production_ready']),
  targetAudience: z.string().optional(),
  notes: z.string().optional(),
  tags: z.array(z.string()).default([]),
  updatedAt: z.string(),
});
export type PotentialAssessment = z.infer<typeof PotentialAssessmentSchema>;

// Local User Annotations (Independent layer to preserve user work across snapshot syncs)
export const UserAnnotationsSchema = z.object({
  schemaVersion: z.literal('1.0'),
  exportedAt: z.string(),
  authenticatedUser: z.string().optional(), // Context isolation (e.g. "alexdev", "demo")
  manualRelationships: z.array(RelationshipSchema).default([]),
  deletedRelationships: z.array(RelationshipTombstoneSchema).default([]),
  potentialAssessments: z.record(z.string(), PotentialAssessmentSchema).default({}), // key is repo id as string
});
export type UserAnnotations = z.infer<typeof UserAnnotationsSchema>;

// Snapshot Metadata
export const SnapshotMetadataSchema = z.object({
  version: z.literal('1.0'),
  createdAt: z.string(),
  referenceDate: z.string(), // Data de corte para cálculo offline determinístico
  authenticatedUser: z.string(),
  totalRepositories: z.number(),
  incompleteSync: z.boolean().default(false),
  syncWarnings: z.array(z.string()).default([]),
});
export type SnapshotMetadata = z.infer<typeof SnapshotMetadataSchema>;

// Complete Portfolio Snapshot Schema
export const SnapshotSchema = z.object({
  metadata: SnapshotMetadataSchema,
  organizations: z.array(OrganizationSchema),
  repositories: z.array(RepositorySchema),
  relationships: z.array(RelationshipSchema),
  userAnnotations: UserAnnotationsSchema.optional(),
});
export type Snapshot = z.infer<typeof SnapshotSchema>;
