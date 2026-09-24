use sha2::{Digest, Sha256};

use crate::domain::{
    catalog::RuleCatalog,
    profile::{build_draft, BuildDraftRequest},
};

use super::{
    model::{
        BootstrapMappingInput, CompileProfileResult, CompileReport, MihomoConfigIr, ProxyGroupIr,
        ResolvedSourceMaterial, StrictCompileError,
    },
    node::compile_nodes,
    rule::compile_rules,
    serializer::serialize_config,
    validation::{validate_ir, validate_round_trip},
};

const HEALTH_CHECK_URL: &str = "https://www.gstatic.com/generate_204";

pub(crate) fn compile_strict_profile(
    catalog: &RuleCatalog,
    mode: String,
    selected_rule_source_ids: Vec<String>,
    github_mirror: Option<String>,
    sources: Vec<ResolvedSourceMaterial>,
    bootstrap_mappings: Vec<BootstrapMappingInput>,
) -> Result<CompileProfileResult, StrictCompileError> {
    let compiled_nodes = compile_nodes(&sources, &bootstrap_mappings)?;
    let node_count = compiled_nodes.nodes.len();
    let duplicate_node_count = compiled_nodes.duplicate_count;
    let target_compatibility_normalization_count =
        compiled_nodes.target_compatibility_normalization_count;
    let draft = build_draft(
        catalog,
        BuildDraftRequest {
            mode: mode.clone(),
            selected_rule_source_ids: selected_rule_source_ids.clone(),
            input_source_count: sources.len() as u32,
            format_ready_source_count: sources.len() as u32,
            resolved_node_count: node_count as u32,
            github_mirror: github_mirror.clone(),
        },
    )
    .map_err(|_| StrictCompileError::Profile)?;
    if !draft.ready_for_compilation {
        return Err(StrictCompileError::NoNodes);
    }

    let groups = compile_groups(&draft.groups, &compiled_nodes.nodes);
    let available_groups = groups
        .iter()
        .map(|group| group.name.clone())
        .collect::<Vec<_>>();
    let compiled_rules = compile_rules(
        catalog,
        &mode,
        &selected_rule_source_ids,
        github_mirror.as_deref(),
        &available_groups,
    )?;
    let rule_provider_count = compiled_rules.providers.len();
    let rule_count = compiled_rules.rules.len();

    let config = MihomoConfigIr {
        nodes: compiled_nodes.nodes,
        bootstrap_hosts: compiled_nodes.bootstrap_hosts,
        groups,
        providers: compiled_rules.providers,
        rules: compiled_rules.rules,
    };
    validate_ir(&config)?;
    let yaml = serialize_config(&config)?;
    validate_round_trip(&yaml)?;
    let content_sha256 = format!("{:x}", Sha256::digest(yaml.as_bytes()));

    let mut warnings = vec![
        "生成 YAML 含节点凭据，属于敏感文件；当前仅完成静态隐私校验。".into(),
        "Clash Verge 加载、DNS 路径、规则命中和最终出口仍需分别验证。".into(),
    ];
    if target_compatibility_normalization_count > 0 {
        warnings.insert(
            0,
            format!(
                "{target_compatibility_normalization_count} 个节点包含订阅导出的 Mihomo 兼容字段（servername 别名、TCP mode 或 REALITY pqv/spx）；已按目标内核边界归一化，DNS 与路由严格隐私检查未放宽。"
            ),
        );
    }

    Ok(CompileProfileResult {
        yaml,
        report: CompileReport {
            status: "generated".into(),
            node_count,
            duplicate_node_count,
            rule_provider_count,
            rule_count,
            strict_privacy: true,
            content_sha256,
            warnings,
        },
    })
}

fn compile_groups(draft_groups: &[String], nodes: &[super::model::NodeIr]) -> Vec<ProxyGroupIr> {
    let node_names = nodes
        .iter()
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    draft_groups
        .iter()
        .map(|name| match name.as_str() {
            "自动选择" => ProxyGroupIr {
                name: name.clone(),
                group_type: "url-test".into(),
                proxies: node_names.clone(),
                url: Some(HEALTH_CHECK_URL.into()),
                interval: Some(600),
                lazy: Some(true),
            },
            "故障转移" => ProxyGroupIr {
                name: name.clone(),
                group_type: "fallback".into(),
                proxies: node_names.clone(),
                url: Some(HEALTH_CHECK_URL.into()),
                interval: Some(600),
                lazy: Some(true),
            },
            "节点选择" => {
                let mut members = vec!["自动选择".into(), "故障转移".into()];
                members.extend(node_names.clone());
                ProxyGroupIr {
                    name: name.clone(),
                    group_type: "select".into(),
                    proxies: members,
                    url: None,
                    interval: None,
                    lazy: None,
                }
            }
            "广告拦截" => ProxyGroupIr {
                name: name.clone(),
                group_type: "select".into(),
                proxies: vec!["REJECT".into(), "节点选择".into()],
                url: None,
                interval: None,
                lazy: None,
            },
            _ => ProxyGroupIr {
                name: name.clone(),
                group_type: "select".into(),
                proxies: vec!["节点选择".into(), "自动选择".into(), "故障转移".into()],
                url: None,
                interval: None,
                lazy: None,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::domain::catalog::{CatalogRuleSet, RuleSource, RuleSourcePopularity};

    use super::*;

    fn source() -> RuleSource {
        RuleSource {
            id: "fixture".into(),
            name: "fixture".into(),
            summary: String::new(),
            repository: "https://github.com/fixture/rules".into(),
            license: "fixture".into(),
            formats: vec!["text".into()],
            categories: Vec::new(),
            available_modes: vec!["simple".into()],
            default_enabled_modes: vec!["simple".into()],
            priority: 1,
            policy_targets: vec!["DIRECT".into()],
            mirror_eligible: true,
            maintenance: "active".into(),
            selection_kind: "default-family".into(),
            content_audit: "text".into(),
            exclusive_group: Some("routing.core".into()),
            upstreams: Vec::new(),
            popularity: RuleSourcePopularity {
                stars: 0,
                forks: 0,
                snapshot_date: "2026-08-14".into(),
            },
        }
    }

    fn rule(id: &str, repository: Option<&str>, target: &str, order: u32) -> CatalogRuleSet {
        CatalogRuleSet {
            id: id.into(),
            title: id.into(),
            repository_id: repository.map(str::to_string),
            family: "fixture".into(),
            canonical_url: repository
                .map(|_| "https://raw.githubusercontent.com/fixture/rules/main/cn.list".into()),
            r#ref: Some("main".into()),
            path: Some("cn.list".into()),
            behavior: if repository.is_some() {
                "classical"
            } else {
                "inline"
            }
            .into(),
            format: if repository.is_some() {
                "text"
            } else {
                "inline"
            }
            .into(),
            target_policy: target.into(),
            order,
            available_modes: vec!["simple".into()],
            default_enabled_modes: vec!["simple".into()],
            exclusive_group: None,
            mirror_kind: if repository.is_some() {
                "github-prefix"
            } else {
                "none"
            }
            .into(),
            content_audit: "text".into(),
            attribution_required: repository.is_some(),
        }
    }

    fn catalog() -> RuleCatalog {
        RuleCatalog {
            schema_version: 1,
            researched_at: "2026-08-14".into(),
            sources: vec![source()],
            rule_sets: vec![
                rule("fixture.cn", Some("fixture"), "DIRECT", 100),
                rule("builtin.match", None, "其他兜底", 1000),
            ],
        }
    }

    fn fixture_source() -> ResolvedSourceMaterial {
        ResolvedSourceMaterial {
            content: br#"
proxies:
  - name: Fixture Node
    type: ss
    server: 192.0.2.10
    port: 443
    cipher: aes-128-gcm
    password: PLACEHOLDER_PASSWORD
"#
            .to_vec(),
        }
    }

    fn domain_fixture_source() -> ResolvedSourceMaterial {
        ResolvedSourceMaterial {
            content: br#"
proxies:
  - name: Fixture Node
    type: ss
    server: node.example.invalid
    port: 443
    cipher: aes-128-gcm
    password: PLACEHOLDER_PASSWORD
"#
            .to_vec(),
        }
    }

    #[test]
    fn generates_and_round_trips_a_strict_profile() {
        let result = compile_strict_profile(
            &catalog(),
            "simple".into(),
            vec!["fixture".into()],
            Some("https://mirror.example.invalid".into()),
            vec![fixture_source()],
            vec![],
        )
        .expect("strict fixture compiles");

        assert_eq!(result.report.status, "generated");
        assert!(result.report.strict_privacy);
        assert_eq!(result.report.node_count, 1);
        assert!(!result.yaml.contains("DIRECT"));
        assert!(result.yaml.contains("strict-route: true"));
        assert!(result.yaml.contains("respect-rules: true"));
        assert!(result.yaml.contains("proxy: 节点选择"));
    }

    #[test]
    fn actual_default_catalog_generates_without_direct_egress() {
        let catalog: RuleCatalog =
            serde_json::from_str(include_str!("../../../../src/data/rule-catalog.json"))
                .expect("embedded catalog fixture parses");
        let result = compile_strict_profile(
            &catalog,
            "simple".into(),
            vec!["acl4ssr".into()],
            None,
            vec![fixture_source()],
            vec![],
        )
        .expect("actual default catalog compiles");

        assert_eq!(result.report.rule_provider_count, 6);
        assert_eq!(result.report.rule_count, 11);
        assert!(!result.yaml.contains("DIRECT"));
        assert_eq!(result.yaml.matches("MATCH,其他兜底").count(), 1);
        assert_eq!(
            result.yaml,
            include_str!("fixtures/strict-profile.golden.yml")
        );
    }

    #[test]
    fn actual_default_catalog_compiles_vless_uri_to_the_golden_profile() {
        let catalog: RuleCatalog =
            serde_json::from_str(include_str!("../../../../src/data/rule-catalog.json"))
                .expect("embedded catalog fixture parses");
        let source = ResolvedSourceMaterial {
            content: concat!(
                "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
                "mode=multi&encryption=none&security=reality&sni=fixture.example.invalid&",
                "servername=fixture.example.invalid&fp=chrome&",
                "type=tcp&flow=xtls-rprx-vision&pbk=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&",
                "sid=0123456789abcdef&spx=%2F#Fixture%20VLESS"
            )
            .as_bytes()
            .to_vec(),
        };
        let result = compile_strict_profile(
            &catalog,
            "simple".into(),
            vec!["acl4ssr".into()],
            None,
            vec![source],
            vec![],
        )
        .expect("VLESS fixture compiles");
        assert_eq!(result.report.node_count, 1);
        assert!(result
            .report
            .warnings
            .iter()
            .any(|warning| warning.contains("1 个节点") && warning.contains("pqv/spx")));
        assert_eq!(
            result.yaml,
            include_str!("fixtures/strict-vless-profile.golden.yml")
        );
    }

    #[test]
    fn actual_default_catalog_compiles_a_mapped_domain_node_to_the_golden_profile() {
        let catalog: RuleCatalog =
            serde_json::from_str(include_str!("../../../../src/data/rule-catalog.json"))
                .expect("embedded catalog fixture parses");
        let result = compile_strict_profile(
            &catalog,
            "simple".into(),
            vec!["acl4ssr".into()],
            None,
            vec![domain_fixture_source()],
            vec![BootstrapMappingInput {
                source: 1,
                node: 1,
                ipv4: "192.0.2.10".into(),
            }],
        )
        .expect("mapped domain fixture compiles");

        assert_eq!(result.report.node_count, 1);
        assert_eq!(
            result.yaml,
            include_str!("fixtures/strict-domain-profile.golden.yml")
        );
    }
}
