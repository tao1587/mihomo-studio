import { useEffect, useMemo, useState } from "react";

import catalogData from "../data/rule-catalog.json";
import {
  buildLocalDraft,
  defaultSelectionForMode,
  visibleSources,
} from "../features/profile-builder/model/catalog";
import { parseBootstrapMappings } from "../features/source-inspection/model/bootstrapMapping";
import {
  buildProfileDraft,
  compileStrictProfile,
  getRuleCatalog,
  inspectNodeText,
  inspectSubscription,
  isTauriRuntime,
} from "../shared/api/studio";
import type {
  BuildDraftRequest,
  CompileProfileResult,
  CompileSourceInput,
  InputSource,
  ProfileDraft,
  RuleCatalog,
  SourceInspectionSummary,
  StudioMode,
} from "../shared/contracts";
import { safeErrorMessage } from "../shared/lib/presentation";

const fallbackCatalog = catalogData as RuleCatalog;
export const DEFAULT_MIRROR = "https://v6.gh-proxy.org/";

export type InspectingKind = "subscription" | "nodes" | null;

interface SourceMaterial extends CompileSourceInput {
  id: string;
}

export function useStudioWorkspace() {
  const desktopRuntime = isTauriRuntime();
  const [catalog, setCatalog] = useState<RuleCatalog>(fallbackCatalog);
  const [mode, setMode] = useState<StudioMode>("simple");
  const [selectedSourceIds, setSelectedSourceIds] = useState<string[]>(() =>
    defaultSelectionForMode(fallbackCatalog, "simple"),
  );
  const [mirrorEnabled, setMirrorEnabled] = useState(true);
  const [mirrorUrl, setMirrorUrl] = useState(DEFAULT_MIRROR);
  const [subscriptionUrl, setSubscriptionUrl] = useState("");
  const [directNodes, setDirectNodes] = useState("");
  const [bootstrapMappings, setBootstrapMappings] = useState("");
  const [inputSources, setInputSources] = useState<InputSource[]>([]);
  const [sourceMaterials, setSourceMaterials] = useState<SourceMaterial[]>([]);
  const [draft, setDraft] = useState<ProfileDraft>(() =>
    buildLocalDraft({
      mode: "simple",
      selectedRuleSourceIds: defaultSelectionForMode(fallbackCatalog, "simple"),
      inputSourceCount: 0,
      formatReadySourceCount: 0,
      resolvedNodeCount: 0,
      githubMirror: DEFAULT_MIRROR,
    }),
  );
  const [isCompiling, setIsCompiling] = useState(false);
  const [compiledProfile, setCompiledProfile] = useState<CompileProfileResult | null>(null);
  const [compileError, setCompileError] = useState("");
  const [inspectingKind, setInspectingKind] = useState<InspectingKind>(null);
  const [sourceError, setSourceError] = useState("");

  useEffect(() => {
    if (!desktopRuntime) return;
    getRuleCatalog()
      .then((value) => {
        setCatalog(value);
        setSelectedSourceIds(defaultSelectionForMode(value, "simple"));
      })
      .catch(() => setCatalog(fallbackCatalog));
  }, [desktopRuntime]);

  useEffect(() => {
    setCompiledProfile(null);
    setCompileError("");
  }, [mode, selectedSourceIds, mirrorEnabled, mirrorUrl, inputSources, bootstrapMappings]);

  const shownSources = useMemo(
    () => visibleSources(catalog, mode),
    [catalog, mode],
  );
  const selectedSources = useMemo(
    () => catalog.sources.filter((source) => selectedSourceIds.includes(source.id)),
    [catalog, selectedSourceIds],
  );
  const resolvedNodeCount = useMemo(
    () => inputSources.reduce((total, source) => total + source.nodeCount, 0),
    [inputSources],
  );
  const formatReadySourceCount = useMemo(
    () => inputSources.filter((source) => source.formatReadyForCompilation).length,
    [inputSources],
  );

  useEffect(() => {
    setDraft(buildLocalDraft({
      mode,
      selectedRuleSourceIds: selectedSourceIds,
      inputSourceCount: inputSources.length,
      formatReadySourceCount,
      resolvedNodeCount,
      githubMirror: mirrorEnabled && mirrorUrl.trim() ? mirrorUrl.trim() : null,
    }));
  }, [
    mode,
    selectedSourceIds,
    inputSources.length,
    formatReadySourceCount,
    resolvedNodeCount,
    mirrorEnabled,
    mirrorUrl,
  ]);

  function changeMode(nextMode: StudioMode) {
    setMode(nextMode);
    setSelectedSourceIds(defaultSelectionForMode(catalog, nextMode));
  }

  function toggleRuleSource(id: string) {
    const candidate = catalog.sources.find((source) => source.id === id);
    if (!candidate || candidate.selectionKind === "upstream-data") return;

    setSelectedSourceIds((current) => {
      if (current.includes(id)) {
        return current.filter((sourceId) => sourceId !== id);
      }
      const withoutExclusivePeer = candidate.exclusiveGroup
        ? current.filter((sourceId) => {
            const selected = catalog.sources.find((source) => source.id === sourceId);
            return selected?.exclusiveGroup !== candidate.exclusiveGroup;
          })
        : current;
      return [...withoutExclusivePeer, id];
    });
  }

  function appendInspectedSource(
    kind: "subscription" | "nodes",
    summary: SourceInspectionSummary,
    value: string,
  ) {
    const id = crypto.randomUUID();
    setInputSources((current) => [
      ...current,
      {
        id,
        kind,
        name: `${kind === "subscription" ? "订阅" : "直接节点"} ${current.filter((item) => item.kind === kind).length + 1}`,
        ...summary,
      },
    ]);
    setSourceMaterials((current) => [
      ...current,
      { id, kind, value, fetchRoute: "system" },
    ]);
  }

  async function addSubscription() {
    const value = subscriptionUrl.trim();
    if (!value) return;
    if (!desktopRuntime) {
      setSourceError("远程订阅检测请在 Tauri 桌面运行时中执行。");
      return;
    }

    setInspectingKind("subscription");
    setSourceError("");
    try {
      appendInspectedSource("subscription", await inspectSubscription(value), value);
      setSubscriptionUrl("");
    } catch (error) {
      setSourceError(safeErrorMessage(error, "订阅检测未完成，请检查地址或网络。"));
    } finally {
      setInspectingKind(null);
    }
  }

  async function addDirectNodes() {
    const value = directNodes.trim();
    if (!value) return;
    if (!desktopRuntime) {
      setSourceError("节点识别请在 Tauri 桌面运行时中执行。");
      return;
    }

    setInspectingKind("nodes");
    setSourceError("");
    try {
      appendInspectedSource("nodes", await inspectNodeText(value), value);
      setDirectNodes("");
    } catch (error) {
      setSourceError(safeErrorMessage(error, "节点内容识别失败。"));
    } finally {
      setInspectingKind(null);
    }
  }

  function removeInputSource(id: string) {
    setInputSources((current) => current.filter((item) => item.id !== id));
    setSourceMaterials((current) => current.filter((item) => item.id !== id));
    setBootstrapMappings("");
  }

  async function compileDraft() {
    const request: BuildDraftRequest = {
      mode,
      selectedRuleSourceIds: selectedSourceIds,
      inputSourceCount: inputSources.length,
      formatReadySourceCount,
      resolvedNodeCount,
      githubMirror: mirrorEnabled && mirrorUrl.trim() ? mirrorUrl.trim() : null,
    };

    const parsedBootstrapMappings = parseBootstrapMappings(bootstrapMappings);
    if (parsedBootstrapMappings.error) {
      setCompiledProfile(null);
      setCompileError(parsedBootstrapMappings.error);
      return;
    }

    setIsCompiling(true);
    setCompileError("");
    try {
      const nextDraft = desktopRuntime
        ? await buildProfileDraft(request)
        : buildLocalDraft(request);
      setDraft(nextDraft);
      if (!desktopRuntime || !nextDraft.readyForCompilation) {
        setCompiledProfile(null);
        if (desktopRuntime) {
          setCompileError(
            nextDraft.warnings[0] ?? "当前来源尚未满足严格编译条件。",
          );
        }
        return;
      }
      setCompiledProfile(
        await compileStrictProfile({
          mode,
          selectedRuleSourceIds: selectedSourceIds,
          githubMirror: request.githubMirror,
          sources: sourceMaterials.map(({ id: _id, ...source }) => source),
          bootstrapMappings: parsedBootstrapMappings.mappings,
        }),
      );
    } catch (error) {
      setCompiledProfile(null);
      setCompileError(safeErrorMessage(error, "严格隐私 YAML 生成失败。"));
    } finally {
      setIsCompiling(false);
    }
  }

  return {
    desktopRuntime,
    catalog,
    mode,
    selectedSourceIds,
    selectedSources,
    shownSources,
    mirrorEnabled,
    mirrorUrl,
    subscriptionUrl,
    directNodes,
    bootstrapMappings,
    inputSources,
    draft,
    isCompiling,
    compiledProfile,
    compileError,
    inspectingKind,
    sourceError,
    setMirrorEnabled,
    setMirrorUrl,
    setSubscriptionUrl,
    setDirectNodes,
    setBootstrapMappings,
    changeMode,
    toggleRuleSource,
    addSubscription,
    addDirectNodes,
    removeInputSource,
    compileDraft,
  };
}
