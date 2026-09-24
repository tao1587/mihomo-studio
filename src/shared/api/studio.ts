import { invoke } from "@tauri-apps/api/core";

import type {
  BuildDraftRequest,
  CompileProfileRequest,
  CompileProfileResult,
  ConvertNodeTextRequest,
  ConvertNodeTextResult,
  ProfileDraft,
  RuleCatalog,
  SourceInspectionSummary,
} from "../contracts";

export function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}

export function getRuleCatalog() {
  return invoke<RuleCatalog>("get_rule_catalog");
}

export function inspectSubscription(url: string) {
  return invoke<SourceInspectionSummary>("inspect_subscription", {
    request: { url, fetchRoute: "system" },
  });
}

export function inspectNodeText(content: string) {
  return invoke<SourceInspectionSummary>("inspect_node_text", {
    request: { content },
  });
}

export function convertNodeText(content: string) {
  const request: ConvertNodeTextRequest = { content };

  return invoke<ConvertNodeTextResult>("convert_node_text", {
    request,
  });
}

export function buildProfileDraft(request: BuildDraftRequest) {
  return invoke<ProfileDraft>("build_profile_draft", { request });
}

export function compileStrictProfile(request: CompileProfileRequest) {
  return invoke<CompileProfileResult>("compile_strict_profile", { request });
}
