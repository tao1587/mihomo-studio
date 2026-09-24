use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr},
};

use yaml_rust2::{Yaml, YamlLoader};

use super::model::{MihomoConfigIr, StrictCompileError};

const PROXY_POLICY: &str = "节点选择";

pub(crate) fn validate_ir(config: &MihomoConfigIr) -> Result<(), StrictCompileError> {
    let node_names = config
        .nodes
        .iter()
        .map(|node| node.name.as_str())
        .collect::<HashSet<_>>();
    if node_names.len() != config.nodes.len() {
        return Err(StrictCompileError::InvalidGroupGraph);
    }
    validate_bootstrap_hosts(config)?;
    let group_names = config
        .groups
        .iter()
        .map(|group| group.name.as_str())
        .collect::<HashSet<_>>();
    if group_names.len() != config.groups.len()
        || group_names.iter().any(|name| node_names.contains(name))
    {
        return Err(StrictCompileError::InvalidGroupGraph);
    }

    for group in &config.groups {
        if group.proxies.is_empty()
            || group.proxies.iter().any(|member| {
                !matches!(member.as_str(), "REJECT" | "REJECT-DROP")
                    && !node_names.contains(member.as_str())
                    && !group_names.contains(member.as_str())
            })
        {
            return Err(StrictCompileError::InvalidGroupGraph);
        }
    }
    reject_group_cycles(config)?;

    if config
        .providers
        .iter()
        .any(|provider| provider.proxy != PROXY_POLICY)
    {
        return Err(StrictCompileError::PrivacyInvariant(
            "provider 未绑定代理策略".into(),
        ));
    }
    if config.rules.iter().any(|rule| {
        rule.target == "DIRECT"
            || (!matches!(rule.target.as_str(), "REJECT" | "REJECT-DROP")
                && !group_names.contains(rule.target.as_str()))
    }) {
        return Err(StrictCompileError::PrivacyInvariant(
            "规则存在直连或悬空目标".into(),
        ));
    }
    let match_rules = config
        .rules
        .iter()
        .filter(|rule| rule.expression.starts_with("MATCH,"))
        .collect::<Vec<_>>();
    if match_rules.len() != 1
        || config
            .rules
            .last()
            .is_none_or(|rule| !rule.expression.starts_with("MATCH,"))
    {
        return Err(StrictCompileError::PrivacyInvariant(
            "MATCH 不唯一或不在最后".into(),
        ));
    }

    Ok(())
}

fn validate_bootstrap_hosts(config: &MihomoConfigIr) -> Result<(), StrictCompileError> {
    let mut required_domains = HashSet::new();
    for node in &config.nodes {
        let Yaml::Hash(mapping) = &node.yaml else {
            return Err(StrictCompileError::InvalidGroupGraph);
        };
        let server = mapping
            .get(&Yaml::String("server".into()))
            .and_then(Yaml::as_str)
            .ok_or(StrictCompileError::InvalidGroupGraph)?;
        match server.parse::<IpAddr>() {
            Ok(IpAddr::V4(_)) => {}
            Ok(IpAddr::V6(_)) => {
                return Err(StrictCompileError::PrivacyInvariant(
                    "节点包含 IPv6 server".into(),
                ));
            }
            Err(_) => {
                required_domains.insert(server.trim().to_ascii_lowercase());
            }
        }
    }

    if required_domains.len() != config.bootstrap_hosts.len()
        || required_domains.iter().any(|domain| {
            config
                .bootstrap_hosts
                .get(domain)
                .is_none_or(|value| value.parse::<Ipv4Addr>().is_err())
        })
    {
        return Err(StrictCompileError::PrivacyInvariant(
            "域名节点缺少已验证 IPv4 hosts 映射".into(),
        ));
    }
    Ok(())
}

fn reject_group_cycles(config: &MihomoConfigIr) -> Result<(), StrictCompileError> {
    let graph = config
        .groups
        .iter()
        .map(|group| (group.name.as_str(), group.proxies.as_slice()))
        .collect::<HashMap<_, _>>();
    let mut complete = HashSet::new();
    let mut visiting = HashSet::new();
    for group in graph.keys() {
        visit_group(group, &graph, &mut visiting, &mut complete)?;
    }
    Ok(())
}

fn visit_group<'a>(
    group: &'a str,
    graph: &HashMap<&'a str, &'a [String]>,
    visiting: &mut HashSet<&'a str>,
    complete: &mut HashSet<&'a str>,
) -> Result<(), StrictCompileError> {
    if complete.contains(group) {
        return Ok(());
    }
    if !visiting.insert(group) {
        return Err(StrictCompileError::InvalidGroupGraph);
    }
    if let Some(members) = graph.get(group) {
        for member in *members {
            if graph.contains_key(member.as_str()) {
                visit_group(member, graph, visiting, complete)?;
            }
        }
    }
    visiting.remove(group);
    complete.insert(group);
    Ok(())
}

pub(crate) fn validate_round_trip(yaml: &str) -> Result<(), StrictCompileError> {
    let documents = YamlLoader::load_from_str(yaml).map_err(|_| StrictCompileError::RoundTrip)?;
    let root = documents.first().ok_or(StrictCompileError::RoundTrip)?;
    if root["ipv6"].as_bool() != Some(false) {
        return privacy_error("顶层 IPv6 未关闭");
    }
    validate_round_trip_bootstrap(root)?;

    let dns = &root["dns"];
    if dns["enable"].as_bool() != Some(true)
        || dns["ipv6"].as_bool() != Some(false)
        || dns["enhanced-mode"].as_str() != Some("fake-ip")
        || dns["respect-rules"].as_bool() != Some(true)
        || dns["use-system-hosts"].as_bool() != Some(false)
    {
        return privacy_error("DNS 关键字段不完整");
    }
    for key in ["nameserver", "proxy-server-nameserver"] {
        let servers = dns[key]
            .as_vec()
            .ok_or_else(|| StrictCompileError::PrivacyInvariant(format!("DNS {key} 缺失")))?;
        if servers.is_empty()
            || servers.iter().any(|server| {
                server.as_str().is_none_or(|value| {
                    !value.starts_with("https://") || !value.ends_with(&format!("#{PROXY_POLICY}"))
                })
            })
        {
            return privacy_error("DNS 上游未全部绑定代理策略");
        }
    }
    for forbidden in ["fallback", "default-nameserver", "direct-nameserver"] {
        if !dns[forbidden].is_badvalue() {
            return privacy_error("存在可能直连的 DNS fallback");
        }
    }

    let tun = &root["tun"];
    if tun["enable"].as_bool() != Some(true)
        || tun["auto-route"].as_bool() != Some(true)
        || tun["strict-route"].as_bool() != Some(true)
    {
        return privacy_error("TUN strict route 未完整启用");
    }
    let hijack = tun["dns-hijack"]
        .as_vec()
        .ok_or_else(|| StrictCompileError::PrivacyInvariant("DNS hijack 缺失".into()))?;
    let hijack_values = hijack
        .iter()
        .filter_map(Yaml::as_str)
        .collect::<HashSet<_>>();
    if !hijack_values.contains("any:53") || !hijack_values.contains("tcp://any:53") {
        return privacy_error("DNS hijack 未覆盖 UDP/TCP 53");
    }

    let providers = root["rule-providers"]
        .as_hash()
        .ok_or(StrictCompileError::RoundTrip)?;
    if providers.values().any(|provider| {
        provider["proxy"].as_str() != Some(PROXY_POLICY)
            || provider["url"]
                .as_str()
                .is_none_or(|url| !url.starts_with("https://"))
    }) {
        return privacy_error("provider 下载未绑定代理或未使用 HTTPS");
    }

    let rules = root["rules"]
        .as_vec()
        .ok_or(StrictCompileError::RoundTrip)?;
    let rule_values = rules.iter().filter_map(Yaml::as_str).collect::<Vec<_>>();
    if rule_values.iter().any(|rule| {
        rule.split(',')
            .any(|segment| segment.eq_ignore_ascii_case("DIRECT"))
    }) {
        return privacy_error("规则中出现 DIRECT");
    }
    if rule_values
        .iter()
        .filter(|rule| rule.starts_with("MATCH,"))
        .count()
        != 1
        || rule_values.last().copied() != Some("MATCH,其他兜底")
    {
        return privacy_error("最终 MATCH 不唯一或顺序错误");
    }

    Ok(())
}

fn validate_round_trip_bootstrap(root: &Yaml) -> Result<(), StrictCompileError> {
    let proxies = root["proxies"]
        .as_vec()
        .ok_or(StrictCompileError::RoundTrip)?;
    let required_domains = proxies
        .iter()
        .map(|proxy| {
            proxy["server"]
                .as_str()
                .ok_or(StrictCompileError::RoundTrip)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|server| server.parse::<IpAddr>().is_err())
        .map(|server| server.trim().to_ascii_lowercase())
        .collect::<HashSet<_>>();

    if required_domains.is_empty() {
        if !root["hosts"].is_badvalue() {
            return privacy_error("存在未被域名节点使用的 hosts 映射");
        }
        return Ok(());
    }

    let hosts = root["hosts"]
        .as_hash()
        .ok_or_else(|| StrictCompileError::PrivacyInvariant("域名节点 hosts 映射缺失".into()))?;
    if hosts.len() != required_domains.len()
        || required_domains.iter().any(|domain| {
            hosts
                .get(&Yaml::String(domain.clone()))
                .and_then(Yaml::as_str)
                .is_none_or(|value| value.parse::<Ipv4Addr>().is_err())
        })
    {
        return privacy_error("域名节点 hosts 映射不完整");
    }
    Ok(())
}

fn privacy_error<T>(detail: &str) -> Result<T, StrictCompileError> {
    Err(StrictCompileError::PrivacyInvariant(detail.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_a_direct_rule_after_round_trip() {
        let yaml = r#"
ipv6: false
dns:
  enable: true
  ipv6: false
  enhanced-mode: fake-ip
  respect-rules: true
  use-system-hosts: false
  nameserver: [https://1.1.1.1/dns-query#节点选择]
  proxy-server-nameserver: [https://1.1.1.1/dns-query#节点选择]
tun:
  enable: true
  auto-route: true
  strict-route: true
  dns-hijack: [any:53, tcp://any:53]
rule-providers: {}
rules: [DOMAIN-SUFFIX,example.invalid,DIRECT, MATCH,其他兜底]
"#;
        assert!(validate_round_trip(yaml).is_err());
    }
}
