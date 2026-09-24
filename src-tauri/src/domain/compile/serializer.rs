use yaml_rust2::{yaml::Hash, Yaml, YamlEmitter};

use super::model::{MihomoConfigIr, ProxyGroupIr, RuleProviderIr, StrictCompileError};

pub(crate) fn serialize_proxy_list(proxies: Vec<Yaml>) -> Result<String, StrictCompileError> {
    let mut root = Hash::new();
    insert(&mut root, "proxies", Yaml::Array(proxies));

    let mut output = String::new();
    YamlEmitter::new(&mut output)
        .dump(&Yaml::Hash(root))
        .map_err(|_| StrictCompileError::Serialization)?;
    output.push('\n');
    Ok(output)
}

pub(crate) fn serialize_config(config: &MihomoConfigIr) -> Result<String, StrictCompileError> {
    let mut root = Hash::new();
    insert(&mut root, "mixed-port", Yaml::Integer(7890));
    insert(&mut root, "allow-lan", Yaml::Boolean(false));
    insert(&mut root, "bind-address", string("127.0.0.1"));
    insert(&mut root, "mode", string("rule"));
    insert(&mut root, "log-level", string("warning"));
    insert(&mut root, "ipv6", Yaml::Boolean(false));
    insert(&mut root, "find-process-mode", string("strict"));
    insert(&mut root, "unified-delay", Yaml::Boolean(true));
    insert(&mut root, "tcp-concurrent", Yaml::Boolean(true));
    if !config.bootstrap_hosts.is_empty() {
        insert(&mut root, "hosts", hosts_yaml(&config.bootstrap_hosts));
    }
    insert(&mut root, "dns", dns_section());
    insert(&mut root, "tun", tun_section());
    insert(
        &mut root,
        "proxies",
        Yaml::Array(config.nodes.iter().map(|node| node.yaml.clone()).collect()),
    );
    insert(
        &mut root,
        "proxy-groups",
        Yaml::Array(config.groups.iter().map(group_yaml).collect()),
    );
    insert(
        &mut root,
        "rule-providers",
        providers_yaml(&config.providers),
    );
    insert(
        &mut root,
        "rules",
        Yaml::Array(
            config
                .rules
                .iter()
                .map(|rule| string(&rule.expression))
                .collect(),
        ),
    );

    let mut output = String::new();
    YamlEmitter::new(&mut output)
        .dump(&Yaml::Hash(root))
        .map_err(|_| StrictCompileError::Serialization)?;
    output.push('\n');
    Ok(output)
}

fn hosts_yaml(hosts: &std::collections::BTreeMap<String, String>) -> Yaml {
    let mut values = Hash::new();
    for (domain, ipv4) in hosts {
        values.insert(string(domain), string(ipv4));
    }
    Yaml::Hash(values)
}

fn dns_section() -> Yaml {
    let mut dns = Hash::new();
    insert(&mut dns, "enable", Yaml::Boolean(true));
    insert(&mut dns, "listen", string("127.0.0.1:1053"));
    insert(&mut dns, "ipv6", Yaml::Boolean(false));
    insert(&mut dns, "cache-algorithm", string("arc"));
    insert(&mut dns, "prefer-h3", Yaml::Boolean(false));
    insert(&mut dns, "use-hosts", Yaml::Boolean(true));
    insert(&mut dns, "use-system-hosts", Yaml::Boolean(false));
    insert(&mut dns, "respect-rules", Yaml::Boolean(true));
    insert(&mut dns, "enhanced-mode", string("fake-ip"));
    insert(&mut dns, "fake-ip-range", string("198.18.0.1/16"));
    insert(
        &mut dns,
        "fake-ip-filter",
        strings(&["*.lan", "localhost", "+.local"]),
    );
    let protected_nameservers = strings(&[
        "https://1.1.1.1/dns-query#节点选择",
        "https://8.8.8.8/dns-query#节点选择",
    ]);
    insert(&mut dns, "nameserver", protected_nameservers.clone());
    insert(&mut dns, "proxy-server-nameserver", protected_nameservers);
    Yaml::Hash(dns)
}

fn tun_section() -> Yaml {
    let mut tun = Hash::new();
    insert(&mut tun, "enable", Yaml::Boolean(true));
    insert(&mut tun, "stack", string("mixed"));
    insert(&mut tun, "auto-route", Yaml::Boolean(true));
    insert(&mut tun, "auto-detect-interface", Yaml::Boolean(true));
    insert(&mut tun, "dns-hijack", strings(&["any:53", "tcp://any:53"]));
    insert(&mut tun, "strict-route", Yaml::Boolean(true));
    insert(&mut tun, "endpoint-independent-nat", Yaml::Boolean(false));
    Yaml::Hash(tun)
}

fn group_yaml(group: &ProxyGroupIr) -> Yaml {
    let mut value = Hash::new();
    insert(&mut value, "name", string(&group.name));
    insert(&mut value, "type", string(&group.group_type));
    insert(
        &mut value,
        "proxies",
        Yaml::Array(group.proxies.iter().map(|proxy| string(proxy)).collect()),
    );
    if let Some(url) = &group.url {
        insert(&mut value, "url", string(url));
    }
    if let Some(interval) = group.interval {
        insert(&mut value, "interval", Yaml::Integer(interval));
    }
    if let Some(lazy) = group.lazy {
        insert(&mut value, "lazy", Yaml::Boolean(lazy));
    }
    Yaml::Hash(value)
}

fn providers_yaml(providers: &[RuleProviderIr]) -> Yaml {
    let mut values = Hash::new();
    for provider in providers {
        let mut value = Hash::new();
        insert(&mut value, "type", string("http"));
        insert(&mut value, "url", string(&provider.url));
        insert(&mut value, "path", string(&provider.path));
        insert(&mut value, "interval", Yaml::Integer(86400));
        insert(&mut value, "proxy", string(&provider.proxy));
        insert(&mut value, "behavior", string(&provider.behavior));
        insert(&mut value, "format", string(&provider.format));
        insert(&mut value, "size-limit", Yaml::Integer(10 * 1024 * 1024));
        values.insert(string(&provider.name), Yaml::Hash(value));
    }
    Yaml::Hash(values)
}

fn insert(mapping: &mut Hash, key: &str, value: Yaml) {
    mapping.insert(string(key), value);
}

fn string(value: &str) -> Yaml {
    Yaml::String(value.to_string())
}

fn strings(values: &[&str]) -> Yaml {
    Yaml::Array(values.iter().map(|value| string(value)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROXY_POLICY: &str = "节点选择";

    #[test]
    fn dns_upstreams_are_bound_to_the_proxy_policy() {
        let dns = dns_section();
        let nameservers = dns["nameserver"].as_vec().expect("nameserver array");
        assert!(nameservers.iter().all(|server| server
            .as_str()
            .is_some_and(|value| value.ends_with(&format!("#{PROXY_POLICY}")))));
        assert!(dns["respect-rules"].as_bool().unwrap_or(false));
        assert!(!dns["ipv6"].as_bool().unwrap_or(true));
    }
}
