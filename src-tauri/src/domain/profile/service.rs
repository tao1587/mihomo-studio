use std::collections::HashSet;

use crate::domain::catalog::RuleCatalog;

use super::{BuildDraftRequest, PrivacyPolicyDraft, ProfileDraft, ProfileError};

fn strict_privacy_policy() -> PrivacyPolicyDraft {
    PrivacyPolicyDraft {
        level: "strict".to_string(),
        dns_mode: "fake-ip".to_string(),
        dns_egress: "proxy-only".to_string(),
        dns_hijack_required: true,
        tun_strict_route_required: true,
        ipv6_enabled: false,
        direct_egress_allowed: false,
        protected_node_bootstrap_required: true,
        provider_updates_via_proxy: true,
    }
}

fn validate_request(
    catalog: &RuleCatalog,
    request: &BuildDraftRequest,
) -> Result<(), ProfileError> {
    if !matches!(request.mode.as_str(), "simple" | "full") {
        return Err(ProfileError::InvalidMode);
    }

    if let Some(mirror) = request.github_mirror.as_deref() {
        let mirror = mirror.trim();
        if !mirror.starts_with("https://") || mirror.len() <= "https://".len() {
            return Err(ProfileError::InvalidMirror);
        }
    }

    let known_ids = catalog
        .sources
        .iter()
        .map(|source| source.id.as_str())
        .collect::<HashSet<_>>();
    if let Some(unknown) = request
        .selected_rule_source_ids
        .iter()
        .find(|source_id| !known_ids.contains(source_id.as_str()))
    {
        return Err(ProfileError::UnknownRuleSource(unknown.clone()));
    }

    let selected = catalog
        .sources
        .iter()
        .filter(|source| request.selected_rule_source_ids.contains(&source.id))
        .collect::<Vec<_>>();
    if let Some(reference_only) = selected
        .iter()
        .find(|source| source.selection_kind == "upstream-data")
    {
        return Err(ProfileError::ReferenceOnlyRuleSource(
            reference_only.id.clone(),
        ));
    }

    let mut exclusive_groups = HashSet::new();
    for group in selected
        .iter()
        .filter_map(|source| source.exclusive_group.as_deref())
    {
        if !exclusive_groups.insert(group) {
            return Err(ProfileError::ExclusiveGroupConflict(group.to_string()));
        }
    }

    Ok(())
}

pub fn build_draft(
    catalog: &RuleCatalog,
    request: BuildDraftRequest,
) -> Result<ProfileDraft, ProfileError> {
    validate_request(catalog, &request)?;

    let mut groups = vec![
        "节点选择".to_string(),
        "自动选择".to_string(),
        "故障转移".to_string(),
        "AI / Claude".to_string(),
        "流媒体".to_string(),
        "广告拦截".to_string(),
    ];
    if request.mode == "full" {
        groups.extend(["Telegram", "Google", "Microsoft", "Apple", "GitHub"].map(str::to_string));
    }
    groups.push("其他兜底".to_string());

    let mut warnings = Vec::new();
    if request.input_source_count == 0 {
        warnings.push("还没有节点来源，当前只能预览规则结构。".to_string());
    } else if request.resolved_node_count == 0 {
        warnings
            .push("来源尚未解析出节点；需要 Mihomo resolver 的来源暂不能进入编译。".to_string());
    } else if request.format_ready_source_count != request.input_source_count {
        warnings.push("部分来源已识别出节点，但其格式或协议尚未接通严格编译。".to_string());
    }
    if request.selected_rule_source_ids.is_empty() {
        warnings.push("至少选择一个规则来源。".to_string());
    }

    let selected = catalog
        .sources
        .iter()
        .filter(|source| request.selected_rule_source_ids.contains(&source.id))
        .collect::<Vec<_>>();
    if selected
        .iter()
        .any(|source| source.content_audit == "opaque")
    {
        warnings.push("所选来源含不透明 MRS，只执行 provider 级去重。".to_string());
    }
    if request.github_mirror.is_some() && selected.iter().any(|source| !source.mirror_eligible) {
        warnings.push("部分来源不是 GitHub 传输地址，不应用 GitHub 镜像前缀。".to_string());
    }

    Ok(ProfileDraft {
        mode: request.mode,
        selected_rule_source_count: request.selected_rule_source_ids.len(),
        input_source_count: request.input_source_count,
        resolved_node_count: request.resolved_node_count,
        groups,
        rule_order: [
            "用户置顶与规则修正",
            "广告与精确拦截",
            "AI / Claude",
            "流媒体与应用服务",
            "国内流量（代理出口）",
            "国外代理",
            "最终兜底",
        ]
        .map(str::to_string)
        .to_vec(),
        final_rule: "MATCH,其他兜底".to_string(),
        privacy: strict_privacy_policy(),
        warnings,
        ready_for_compilation: request.input_source_count > 0
            && request.format_ready_source_count == request.input_source_count
            && request.resolved_node_count > 0
            && !request.selected_rule_source_ids.is_empty(),
    })
}

#[cfg(test)]
mod tests {
    use crate::domain::catalog::{RuleCatalog, RuleSource, RuleSourcePopularity};

    use super::*;

    fn source(
        id: &str,
        exclusive_group: Option<&str>,
        selection_kind: &str,
        content_audit: &str,
        mirror_eligible: bool,
    ) -> RuleSource {
        RuleSource {
            id: id.into(),
            name: id.into(),
            summary: String::new(),
            repository: format!("https://github.com/fixture/{id}"),
            license: "fixture".into(),
            formats: vec!["text".into()],
            categories: Vec::new(),
            available_modes: vec!["simple".into(), "full".into()],
            default_enabled_modes: Vec::new(),
            priority: 1,
            policy_targets: Vec::new(),
            mirror_eligible,
            maintenance: "active".into(),
            selection_kind: selection_kind.into(),
            content_audit: content_audit.into(),
            exclusive_group: exclusive_group.map(str::to_string),
            upstreams: Vec::new(),
            popularity: RuleSourcePopularity {
                stars: 0,
                forks: 0,
                snapshot_date: "2026-08-14".into(),
            },
        }
    }

    fn catalog() -> RuleCatalog {
        RuleCatalog {
            schema_version: 1,
            researched_at: "2026-08-14".into(),
            sources: vec![
                source(
                    "acl4ssr",
                    Some("routing.core"),
                    "default-family",
                    "text",
                    true,
                ),
                source(
                    "loyalsoldier-clash-rules",
                    Some("routing.core"),
                    "replacement-family",
                    "text",
                    true,
                ),
                source(
                    "v2fly-domain-list-community",
                    None,
                    "upstream-data",
                    "text",
                    true,
                ),
                source("metacubex-meta-rules-dat", None, "geodata", "opaque", true),
                source("sukkaw-ruleset", None, "replacement-family", "text", false),
            ],
            rule_sets: Vec::new(),
        }
    }

    fn request() -> BuildDraftRequest {
        BuildDraftRequest {
            mode: "simple".into(),
            selected_rule_source_ids: vec!["acl4ssr".into()],
            input_source_count: 1,
            format_ready_source_count: 1,
            resolved_node_count: 3,
            github_mirror: Some("https://mirror.example.invalid/".into()),
        }
    }

    #[test]
    fn fallback_group_and_match_are_last() {
        let draft = build_draft(&catalog(), request()).expect("draft should build");
        assert_eq!(draft.groups.last().map(String::as_str), Some("其他兜底"));
        assert_eq!(
            draft.rule_order.last().map(String::as_str),
            Some("最终兜底")
        );
        assert_eq!(draft.final_rule, "MATCH,其他兜底");
        assert_eq!(draft.privacy.level, "strict");
        assert_eq!(draft.privacy.dns_mode, "fake-ip");
        assert_eq!(draft.privacy.dns_egress, "proxy-only");
        assert!(draft.privacy.dns_hijack_required);
        assert!(draft.privacy.tun_strict_route_required);
        assert!(!draft.privacy.ipv6_enabled);
        assert!(!draft.privacy.direct_egress_allowed);
        assert!(draft.privacy.protected_node_bootstrap_required);
        assert!(draft.privacy.provider_updates_via_proxy);
        assert!(!draft.rule_order.iter().any(|rule| rule.contains("直连")));
        assert!(draft.ready_for_compilation);
    }

    #[test]
    fn full_mode_keeps_the_same_strict_privacy_contract() {
        let mut value = request();
        value.mode = "full".into();

        let draft = build_draft(&catalog(), value).expect("full draft should build");

        assert_eq!(draft.privacy.level, "strict");
        assert_eq!(draft.privacy.dns_egress, "proxy-only");
        assert!(draft.privacy.dns_hijack_required);
        assert!(draft.privacy.tun_strict_route_required);
        assert!(!draft.privacy.ipv6_enabled);
        assert!(!draft.privacy.direct_egress_allowed);
        assert!(!draft.rule_order.iter().any(|rule| rule.contains("直连")));
    }

    #[test]
    fn recognized_but_unmapped_sources_are_not_compilation_ready() {
        let mut value = request();
        value.format_ready_source_count = 0;

        let draft = build_draft(&catalog(), value).expect("draft should remain inspectable");

        assert!(!draft.ready_for_compilation);
        assert!(draft
            .warnings
            .iter()
            .any(|warning| warning.contains("格式或协议尚未接通")));
    }

    #[test]
    fn rejects_unknown_rule_source() {
        let mut value = request();
        value.selected_rule_source_ids = vec!["unknown".into()];
        assert_eq!(
            build_draft(&catalog(), value)
                .expect_err("unknown source should fail")
                .to_string(),
            "unknown rule source: unknown"
        );
    }

    #[test]
    fn rejects_two_core_routing_families() {
        let mut value = request();
        value.selected_rule_source_ids = vec!["acl4ssr".into(), "loyalsoldier-clash-rules".into()];
        assert_eq!(
            build_draft(&catalog(), value)
                .expect_err("exclusive routing families should fail")
                .to_string(),
            "multiple rule sources selected from exclusive group: routing.core"
        );
    }

    #[test]
    fn rejects_reference_only_upstream_data() {
        let mut value = request();
        value.selected_rule_source_ids = vec!["v2fly-domain-list-community".into()];
        assert_eq!(
            build_draft(&catalog(), value)
                .expect_err("upstream data should not be selectable")
                .to_string(),
            "rule source is reference-only: v2fly-domain-list-community"
        );
    }

    #[test]
    fn reports_opaque_and_non_github_transport_boundaries() {
        let mut opaque = request();
        opaque.selected_rule_source_ids = vec!["metacubex-meta-rules-dat".into()];
        let opaque_draft = build_draft(&catalog(), opaque).expect("opaque source should build");
        assert!(opaque_draft
            .warnings
            .iter()
            .any(|warning| warning.contains("provider 级去重")));

        let mut direct_transport = request();
        direct_transport.selected_rule_source_ids = vec!["sukkaw-ruleset".into()];
        let direct_draft =
            build_draft(&catalog(), direct_transport).expect("direct source should build");
        assert!(direct_draft
            .warnings
            .iter()
            .any(|warning| warning.contains("不应用 GitHub 镜像")));
    }

    #[test]
    fn rejects_non_https_mirror() {
        let mut value = request();
        value.github_mirror = Some("http://mirror.example.invalid/".into());
        assert_eq!(
            build_draft(&catalog(), value)
                .expect_err("insecure mirror should fail")
                .to_string(),
            "GitHub mirror must be a non-empty HTTPS URL"
        );
    }

    #[test]
    fn unresolved_source_does_not_claim_compile_readiness() {
        let mut value = request();
        value.resolved_node_count = 0;
        let draft = build_draft(&catalog(), value).expect("draft should remain valid");
        assert!(!draft.ready_for_compilation);
        assert!(draft
            .warnings
            .iter()
            .any(|warning| warning.contains("Mihomo resolver")));
    }
}
