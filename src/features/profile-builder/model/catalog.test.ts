import { describe, expect, it } from "vitest";
import catalogData from "../../../data/rule-catalog.json";
import type { RuleCatalog } from "../../../shared/contracts";
import {
  buildLocalDraft,
  defaultSelectionForMode,
  visibleSources,
} from "./catalog";

const catalog = catalogData as RuleCatalog;

describe("rule catalog", () => {
  it("keeps source ids unique and canonical repositories secure", () => {
    const ids = catalog.sources.map((source) => source.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(catalog.sources.every((source) => source.repository.startsWith("https://github.com/"))).toBe(true);
  });

  it("does not contain the removed legacy source", () => {
    expect(JSON.stringify(catalog).toLowerCase()).not.toContain("divineengine");
  });


  it("keeps one default ad source and one final match", () => {
    const defaults = catalog.ruleSets.filter((ruleSet) =>
      ruleSet.defaultEnabledModes.includes("simple"),
    );
    expect(defaults.filter((ruleSet) => ruleSet.exclusiveGroup === "ads.primary")).toHaveLength(1);
    expect(defaults.filter((ruleSet) => ruleSet.id === "builtin.match" && ruleSet.order === 1000)).toHaveLength(1);
  });

  it("offers a smaller simple catalog than the full catalog", () => {
    expect(visibleSources(catalog, "simple").length).toBeGreaterThan(0);
    expect(visibleSources(catalog, "simple").length).toBeLessThanOrEqual(
      visibleSources(catalog, "full").length,
    );
    expect(defaultSelectionForMode(catalog, "simple")).toEqual(["acl4ssr"]);
  });
});

describe("profile draft", () => {
  it("always emits one last fallback rule", () => {
    const draft = buildLocalDraft({
      mode: "simple",
      selectedRuleSourceIds: ["acl4ssr"],
      inputSourceCount: 1,
      formatReadySourceCount: 1,
      resolvedNodeCount: 3,
      githubMirror: null,
    });

    expect(draft.finalRule).toBe("MATCH,其他兜底");
    expect(draft.groups[draft.groups.length - 1]).toBe("其他兜底");
    expect(draft.ruleOrder[draft.ruleOrder.length - 1]).toBe("最终兜底");
    expect(draft.privacy).toEqual({
      level: "strict",
      dnsMode: "fake-ip",
      dnsEgress: "proxy-only",
      dnsHijackRequired: true,
      tunStrictRouteRequired: true,
      ipv6Enabled: false,
      directEgressAllowed: false,
      protectedNodeBootstrapRequired: true,
      providerUpdatesViaProxy: true,
    });
    expect(draft.ruleOrder.some((rule) => rule.includes("直连"))).toBe(false);
    expect(draft.readyForCompilation).toBe(true);
  });

  it("does not mark resolver-only sources as compilation-ready", () => {
    const draft = buildLocalDraft({
      mode: "full",
      selectedRuleSourceIds: ["acl4ssr"],
      inputSourceCount: 1,
      formatReadySourceCount: 0,
      resolvedNodeCount: 0,
      githubMirror: null,
    });

    expect(draft.readyForCompilation).toBe(false);
    expect(draft.warnings.join(" ")).toContain("Mihomo resolver");
  });

  it("does not confuse recognized nodes with compiler-ready formats", () => {
    const draft = buildLocalDraft({
      mode: "simple",
      selectedRuleSourceIds: ["acl4ssr"],
      inputSourceCount: 1,
      formatReadySourceCount: 0,
      resolvedNodeCount: 9,
      githubMirror: null,
    });

    expect(draft.readyForCompilation).toBe(false);
    expect(draft.warnings.join(" ")).toContain("格式或协议尚未接通");
  });

  it("keeps strict privacy constraints in full mode", () => {
    const draft = buildLocalDraft({
      mode: "full",
      selectedRuleSourceIds: ["acl4ssr"],
      inputSourceCount: 1,
      formatReadySourceCount: 1,
      resolvedNodeCount: 3,
      githubMirror: null,
    });

    expect(draft.privacy.level).toBe("strict");
    expect(draft.privacy.dnsEgress).toBe("proxy-only");
    expect(draft.privacy.dnsHijackRequired).toBe(true);
    expect(draft.privacy.tunStrictRouteRequired).toBe(true);
    expect(draft.privacy.ipv6Enabled).toBe(false);
    expect(draft.privacy.directEgressAllowed).toBe(false);
    expect(draft.ruleOrder.some((rule) => rule.includes("直连"))).toBe(false);
  });
});
