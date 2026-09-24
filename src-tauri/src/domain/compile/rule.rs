use std::collections::{HashMap, HashSet};

use crate::domain::catalog::{CatalogRuleSet, RuleCatalog};

use super::model::{CompiledRules, RuleIr, RuleProviderIr, StrictCompileError};

const PROXY_POLICY: &str = "节点选择";
const FALLBACK_POLICY: &str = "其他兜底";

pub(crate) fn compile_rules(
    catalog: &RuleCatalog,
    mode: &str,
    selected_source_ids: &[String],
    github_mirror: Option<&str>,
    available_groups: &[String],
) -> Result<CompiledRules, StrictCompileError> {
    let selected = selected_source_ids.iter().cloned().collect::<HashSet<_>>();
    let sources_by_id = catalog
        .sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect::<HashMap<_, _>>();

    for source_id in &selected {
        let has_rule_set = catalog.rule_sets.iter().any(|rule_set| {
            rule_set.repository_id.as_deref() == Some(source_id.as_str())
                && rule_set.available_modes.iter().any(|value| value == mode)
        });
        if !has_rule_set {
            return Err(StrictCompileError::NoRuleSetsForSource(source_id.clone()));
        }
    }

    let preferred_exclusive_sources = selected
        .iter()
        .filter_map(|source_id| sources_by_id.get(source_id.as_str()))
        .filter_map(|source| {
            source
                .exclusive_group
                .as_ref()
                .map(|group| (group.as_str(), source.id.as_str()))
        })
        .collect::<HashMap<_, _>>();

    let mut selected_rule_sets = catalog
        .rule_sets
        .iter()
        .filter(|rule_set| rule_set.id != "builtin.geoip-cn")
        .filter(|rule_set| rule_set.available_modes.iter().any(|value| value == mode))
        .filter(|rule_set| match rule_set.repository_id.as_deref() {
            None => rule_set
                .default_enabled_modes
                .iter()
                .any(|value| value == mode),
            Some(repository_id) => selected.contains(repository_id),
        })
        .filter(|rule_set| {
            let Some(exclusive_group) = rule_set.exclusive_group.as_deref() else {
                return true;
            };
            let Some(preferred_source) = preferred_exclusive_sources.get(exclusive_group) else {
                return true;
            };
            rule_set.repository_id.as_deref() == Some(*preferred_source)
        })
        .collect::<Vec<_>>();
    selected_rule_sets.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.id.cmp(&right.id))
    });

    let mut providers = Vec::new();
    let mut ordered_rules = Vec::<(u32, String, RuleIr)>::new();
    for rule_set in selected_rule_sets {
        let target = strict_target(&rule_set.target_policy);
        validate_target(&target, available_groups)?;
        if rule_set.repository_id.is_some() {
            let provider = build_provider(rule_set, github_mirror)?;
            ordered_rules.push((
                rule_set.order,
                rule_set.id.clone(),
                RuleIr {
                    expression: format!("RULE-SET,{},{}", provider.name, target),
                    target,
                },
            ));
            providers.push(provider);
        } else {
            for (sequence, expression) in builtin_rules(&rule_set.id, &target)?
                .into_iter()
                .enumerate()
            {
                ordered_rules.push((
                    rule_set.order,
                    format!("{}-{sequence:03}", rule_set.id),
                    RuleIr {
                        expression,
                        target: target.clone(),
                    },
                ));
            }
        }
    }

    ordered_rules.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    let rules = ordered_rules
        .into_iter()
        .map(|(_, _, rule)| rule)
        .collect::<Vec<_>>();

    let match_count = rules
        .iter()
        .filter(|rule| rule.expression.starts_with("MATCH,"))
        .count();
    if match_count != 1
        || rules
            .last()
            .is_none_or(|rule| rule.expression != format!("MATCH,{FALLBACK_POLICY}"))
    {
        return Err(StrictCompileError::PrivacyInvariant(
            "最终规则必须是唯一 MATCH,其他兜底".into(),
        ));
    }
    if rules.iter().any(|rule| rule.target == "DIRECT") {
        return Err(StrictCompileError::PrivacyInvariant(
            "规则包含 DIRECT 出口".into(),
        ));
    }

    Ok(CompiledRules { providers, rules })
}

fn strict_target(target: &str) -> String {
    if target == "DIRECT" {
        PROXY_POLICY.to_string()
    } else {
        target.to_string()
    }
}

fn validate_target(target: &str, available_groups: &[String]) -> Result<(), StrictCompileError> {
    if matches!(target, "REJECT" | "REJECT-DROP")
        || available_groups.iter().any(|group| group == target)
    {
        return Ok(());
    }
    Err(StrictCompileError::InvalidRuleTarget(target.to_string()))
}

fn build_provider(
    rule_set: &CatalogRuleSet,
    github_mirror: Option<&str>,
) -> Result<RuleProviderIr, StrictCompileError> {
    let canonical_url = rule_set
        .canonical_url
        .as_deref()
        .ok_or_else(|| StrictCompileError::InvalidRuleProvider(rule_set.id.clone()))?;
    let url = if rule_set.mirror_kind == "github-prefix" {
        github_mirror
            .filter(|mirror| !mirror.trim().is_empty())
            .map(|mirror| format!("{}/{}", mirror.trim().trim_end_matches('/'), canonical_url))
            .unwrap_or_else(|| canonical_url.to_string())
    } else {
        canonical_url.to_string()
    };
    if !url.starts_with("https://") {
        return Err(StrictCompileError::InvalidRuleProvider(rule_set.id.clone()));
    }

    let extension = match rule_set.format.as_str() {
        "yaml" => "yaml",
        "text" => "txt",
        "mrs" => "mrs",
        _ => {
            return Err(StrictCompileError::InvalidRuleProvider(rule_set.id.clone()));
        }
    };
    Ok(RuleProviderIr {
        name: rule_set.id.clone(),
        url,
        path: format!("./rulesets/{}.{}", rule_set.id.replace('.', "-"), extension),
        behavior: rule_set.behavior.clone(),
        format: rule_set.format.clone(),
        proxy: PROXY_POLICY.to_string(),
    })
}

fn builtin_rules(id: &str, target: &str) -> Result<Vec<String>, StrictCompileError> {
    match id {
        "builtin.claude.route" => Ok([
            "DOMAIN-SUFFIX,claude.ai",
            "DOMAIN-SUFFIX,anthropic.com",
            "DOMAIN-SUFFIX,claude.com",
            "DOMAIN-SUFFIX,anthropicusercontent.com",
        ]
        .map(|matcher| format!("{matcher},{target}"))
        .to_vec()),
        "builtin.match" => Ok(vec![format!("MATCH,{target}")]),
        _ => Err(StrictCompileError::InvalidRuleProvider(id.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::catalog::{CatalogRuleSet, RuleCatalog, RuleSource, RuleSourcePopularity};

    use super::*;

    fn source(id: &str, exclusive_group: Option<&str>) -> RuleSource {
        RuleSource {
            id: id.into(),
            name: id.into(),
            summary: String::new(),
            repository: format!("https://github.com/fixture/{id}"),
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
            exclusive_group: exclusive_group.map(str::to_string),
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
                .map(|_| format!("https://raw.githubusercontent.com/fixture/repo/main/{id}.list")),
            r#ref: Some("main".into()),
            path: Some(format!("{id}.list")),
            behavior: if repository.is_some() {
                "classical".into()
            } else {
                "inline".into()
            },
            format: if repository.is_some() {
                "text".into()
            } else {
                "inline".into()
            },
            target_policy: target.into(),
            order,
            available_modes: vec!["simple".into()],
            default_enabled_modes: vec!["simple".into()],
            exclusive_group: None,
            mirror_kind: if repository.is_some() {
                "github-prefix".into()
            } else {
                "none".into()
            },
            content_audit: "text".into(),
            attribution_required: repository.is_some(),
        }
    }

    #[test]
    fn maps_catalog_direct_targets_to_the_proxy_policy() {
        let catalog = RuleCatalog {
            schema_version: 1,
            researched_at: "2026-08-14".into(),
            sources: vec![source("fixture", Some("routing.core"))],
            rule_sets: vec![
                rule("fixture.cn", Some("fixture"), "DIRECT", 100),
                rule("builtin.match", None, FALLBACK_POLICY, 1000),
            ],
        };
        let groups = vec![PROXY_POLICY.into(), FALLBACK_POLICY.into()];
        let compiled = compile_rules(
            &catalog,
            "simple",
            &["fixture".into()],
            Some("https://mirror.example.invalid"),
            &groups,
        )
        .expect("rules compile");

        assert_eq!(compiled.providers[0].proxy, PROXY_POLICY);
        assert!(compiled.providers[0]
            .url
            .starts_with("https://mirror.example.invalid/https://"));
        assert!(compiled.rules.iter().all(|rule| rule.target != "DIRECT"));
        assert_eq!(
            compiled.rules.last().map(|rule| rule.expression.as_str()),
            Some("MATCH,其他兜底")
        );
    }
}
