import type {
  BuildDraftRequest,
  ProfileDraft,
  RuleCatalog,
  StudioMode,
} from "../../../shared/contracts";

export function defaultSelectionForMode(
  catalog: RuleCatalog,
  mode: StudioMode,
): string[] {
  return catalog.sources
    .filter((source) => source.defaultEnabledModes.includes(mode))
    .sort((left, right) => left.priority - right.priority)
    .map((source) => source.id);
}

export function visibleSources(catalog: RuleCatalog, mode: StudioMode) {
  if (mode === "full") return [...catalog.sources];
  return catalog.sources.filter((source) => source.availableModes.includes("simple"));
}

export function buildLocalDraft(request: BuildDraftRequest): ProfileDraft {
  const groups = [
    "节点选择",
    "自动选择",
    "故障转移",
    "AI / Claude",
    "流媒体",
    "广告拦截",
  ];

  if (request.mode === "full") {
    groups.push("Telegram", "Google", "Microsoft", "Apple", "GitHub");
  }
  groups.push("其他兜底");

  const warnings: string[] = [];
  if (request.inputSourceCount === 0) warnings.push("还没有节点来源，当前只能预览规则结构。");
  else if (request.resolvedNodeCount === 0) warnings.push("来源尚未解析出节点；需要 Mihomo resolver 的来源暂不能进入编译。");
  else if (request.formatReadySourceCount !== request.inputSourceCount) {
    warnings.push("部分来源已识别出节点，但其格式或协议尚未接通严格编译。");
  }
  if (request.selectedRuleSourceIds.length === 0) warnings.push("至少选择一个规则来源。");

  return {
    mode: request.mode,
    selectedRuleSourceCount: request.selectedRuleSourceIds.length,
    inputSourceCount: request.inputSourceCount,
    resolvedNodeCount: request.resolvedNodeCount,
    groups,
    ruleOrder: [
      "用户置顶与规则修正",
      "广告与精确拦截",
      "AI / Claude",
      "流媒体与应用服务",
      "国内流量（代理出口）",
      "国外代理",
      "最终兜底",
    ],
    finalRule: "MATCH,其他兜底",
    privacy: {
      level: "strict",
      dnsMode: "fake-ip",
      dnsEgress: "proxy-only",
      dnsHijackRequired: true,
      tunStrictRouteRequired: true,
      ipv6Enabled: false,
      directEgressAllowed: false,
      protectedNodeBootstrapRequired: true,
      providerUpdatesViaProxy: true,
    },
    warnings,
    readyForCompilation:
      request.inputSourceCount > 0
      && request.formatReadySourceCount === request.inputSourceCount
      && request.resolvedNodeCount > 0
      && request.selectedRuleSourceIds.length > 0,
  };
}
