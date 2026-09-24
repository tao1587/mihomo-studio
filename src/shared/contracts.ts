export type StudioMode = "simple" | "full";

export interface RuleSourcePopularity {
  stars: number;
  forks: number;
  snapshotDate: string;
}

export interface RuleSource {
  id: string;
  name: string;
  summary: string;
  repository: string;
  license: string;
  formats: string[];
  categories: string[];
  availableModes: StudioMode[];
  defaultEnabledModes: StudioMode[];
  priority: number;
  policyTargets: string[];
  mirrorEligible: boolean;
  maintenance: "active" | "stable" | "watch";
  selectionKind:
    | "default-family"
    | "geodata"
    | "service-catalog"
    | "replacement-family"
    | "ads-option"
    | "experimental"
    | "upstream-data";
  contentAudit: "text" | "mixed" | "opaque";
  exclusiveGroup: string | null;
  upstreams: string[];
  popularity: RuleSourcePopularity;
}

export interface CatalogRuleSet {
  id: string;
  title: string;
  repositoryId: string | null;
  family: string;
  canonicalUrl: string | null;
  ref: string | null;
  path: string | null;
  behavior: "classical" | "domain" | "ipcidr" | "inline";
  format: "text" | "yaml" | "mrs" | "inline";
  targetPolicy: string;
  order: number;
  availableModes: StudioMode[];
  defaultEnabledModes: StudioMode[];
  exclusiveGroup: string | null;
  mirrorKind: "github-prefix" | "direct-only" | "none";
  contentAudit: "text" | "opaque" | "builtin";
  attributionRequired: boolean;
}

export interface RuleCatalog {
  schemaVersion: number;
  researchedAt: string;
  sources: RuleSource[];
  ruleSets: CatalogRuleSet[];
}

export interface InputSource {
  id: string;
  kind: "subscription" | "nodes" | "file";
  name: string;
  safeLabel: string;
  sourceFormat: string;
  nodeCount: number;
  duplicateCount: number;
  protocols: ProtocolCount[];
  userAgent: string | null;
  warnings: string[];
  requiresMihomoResolver: boolean;
  formatReadyForCompilation: boolean;
}

export interface ProtocolCount {
  protocol: string;
  count: number;
}

export interface SourceInspectionSummary {
  safeLabel: string;
  sourceFormat: string;
  nodeCount: number;
  duplicateCount: number;
  protocols: ProtocolCount[];
  userAgent: string | null;
  warnings: string[];
  requiresMihomoResolver: boolean;
  formatReadyForCompilation: boolean;
}

export interface ConvertNodeTextRequest {
  content: string;
}

export interface ConvertNodeTextResult {
  yaml: string;
  templateYaml: string | null;
  templateFileName: string | null;
  nodeCount: number;
  duplicateNodeCount: number;
  compatibilityNormalizationCount: number;
  warnings: string[];
}

export interface BuildDraftRequest {
  mode: StudioMode;
  selectedRuleSourceIds: string[];
  inputSourceCount: number;
  formatReadySourceCount: number;
  resolvedNodeCount: number;
  githubMirror: string | null;
}

export interface ProfileDraft {
  mode: StudioMode;
  selectedRuleSourceCount: number;
  inputSourceCount: number;
  resolvedNodeCount: number;
  groups: string[];
  ruleOrder: string[];
  finalRule: string;
  privacy: PrivacyPolicyDraft;
  warnings: string[];
  readyForCompilation: boolean;
}

export interface PrivacyPolicyDraft {
  level: "strict";
  dnsMode: "fake-ip";
  dnsEgress: "proxy-only";
  dnsHijackRequired: true;
  tunStrictRouteRequired: true;
  ipv6Enabled: false;
  directEgressAllowed: false;
  protectedNodeBootstrapRequired: true;
  providerUpdatesViaProxy: true;
}

export interface CompileSourceInput {
  kind: "subscription" | "nodes";
  value: string;
  fetchRoute: "system" | "direct";
}

export interface BootstrapMappingInput {
  source: number;
  node: number;
  ipv4: string;
}

export interface CompileProfileRequest {
  mode: StudioMode;
  selectedRuleSourceIds: string[];
  githubMirror: string | null;
  sources: CompileSourceInput[];
  bootstrapMappings: BootstrapMappingInput[];
}

export interface CompileReport {
  status: "generated";
  nodeCount: number;
  duplicateNodeCount: number;
  ruleProviderCount: number;
  ruleCount: number;
  strictPrivacy: boolean;
  contentSha256: string;
  warnings: string[];
}

export interface CompileProfileResult {
  yaml: string;
  report: CompileReport;
}
