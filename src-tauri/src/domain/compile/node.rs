use std::{
    collections::{BTreeMap, HashMap, HashSet},
    net::{IpAddr, Ipv4Addr},
};

use base64::{engine::general_purpose, Engine as _};
use percent_encoding::percent_decode_str;
use sha2::{Digest, Sha256};
use url::{Host, Url};
use yaml_rust2::{yaml::Hash, Yaml, YamlLoader};

use crate::domain::source::MAX_SOURCE_BYTES;

use super::model::{
    BootstrapMappingInput, CompiledNodes, ConvertNodeTextResult, NodeCompileIssue, NodeIr,
    ResolvedSourceMaterial, StrictCompileError, MAX_REPORTED_NODE_ISSUES,
};
use super::serializer::serialize_proxy_list;
use super::template::render_single_ip_template;

const RESERVED_NAMES: &[&str] = &[
    "节点选择",
    "自动选择",
    "故障转移",
    "AI / Claude",
    "流媒体",
    "广告拦截",
    "Telegram",
    "Google",
    "Microsoft",
    "Apple",
    "GitHub",
    "其他兜底",
    "DIRECT",
    "REJECT",
    "REJECT-DROP",
    "PASS",
    "COMPATIBLE",
];

struct ParsedProxy {
    yaml: Yaml,
    target_compatibility_normalization: bool,
}

impl ParsedProxy {
    fn exact(yaml: Yaml) -> Self {
        Self {
            yaml,
            target_compatibility_normalization: false,
        }
    }
}

#[derive(Clone, Copy)]
enum WireGuardSection {
    Interface,
    Peer,
}

pub(crate) fn compile_nodes(
    sources: &[ResolvedSourceMaterial],
    bootstrap_mapping_inputs: &[BootstrapMappingInput],
) -> Result<CompiledNodes, StrictCompileError> {
    if sources.is_empty() {
        return Err(StrictCompileError::NoSources);
    }

    let mut nodes = Vec::new();
    let mut seen_fingerprints = HashSet::new();
    let mut allocated_names = HashMap::<String, usize>::new();
    let mut duplicate_count = 0usize;
    let mut target_compatibility_normalization_count = 0usize;
    let mut bootstrap_mappings = parse_bootstrap_mappings(bootstrap_mapping_inputs)?;
    let mut bootstrap_hosts = BTreeMap::<String, String>::new();

    for (source_index, source) in sources.iter().enumerate() {
        let source_number = source_index + 1;
        if source.content.len() > MAX_SOURCE_BYTES {
            return Err(StrictCompileError::SourceTooLarge(source_number));
        }
        let text = std::str::from_utf8(&source.content)
            .map_err(|_| StrictCompileError::InvalidUtf8(source_number))?;
        let proxies = extract_proxies(text, source_number)?;

        for (node_index, proxy) in proxies.into_iter().enumerate() {
            let node_number = node_index + 1;
            let Yaml::Hash(mut mapping) = proxy.yaml else {
                return Err(StrictCompileError::InvalidNodeEntry {
                    source: source_number,
                    node: node_number,
                });
            };

            let bootstrap_position = (source_number, node_number);
            let bootstrap_host = validate_node(
                &mapping,
                source_number,
                node_number,
                bootstrap_mappings.get(&bootstrap_position).copied(),
            )?;
            if let Some((domain, ipv4)) = bootstrap_host {
                bootstrap_mappings.remove(&bootstrap_position);
                if bootstrap_hosts
                    .get(&domain)
                    .is_some_and(|current| current != &ipv4.to_string())
                {
                    return Err(StrictCompileError::InvalidBootstrapMapping {
                        source: source_number,
                        node: node_number,
                    });
                }
                bootstrap_hosts.insert(domain, ipv4.to_string());
            }
            let fingerprint = node_fingerprint(&mapping);
            if !seen_fingerprints.insert(fingerprint) {
                duplicate_count += 1;
                continue;
            }
            if proxy.target_compatibility_normalization {
                target_compatibility_normalization_count += 1;
            }

            let original_name = string_field(&mapping, "name")
                .filter(|value| !value.trim().is_empty())
                .map(str::trim)
                .unwrap_or("节点");
            let base_name = if RESERVED_NAMES.contains(&original_name) {
                format!("节点 · {original_name}")
            } else {
                original_name.to_string()
            };
            let name = allocate_name(&base_name, &mut allocated_names);
            mapping.insert(Yaml::String("name".into()), Yaml::String(name.clone()));
            nodes.push(NodeIr {
                name,
                yaml: Yaml::Hash(mapping),
            });
        }
    }

    if nodes.is_empty() {
        return Err(StrictCompileError::NoNodes);
    }
    if let Some(((source, node), _)) = bootstrap_mappings.first_key_value() {
        return Err(StrictCompileError::InvalidBootstrapMapping {
            source: *source,
            node: *node,
        });
    }

    Ok(CompiledNodes {
        nodes,
        duplicate_count,
        bootstrap_hosts,
        target_compatibility_normalization_count,
    })
}

fn parse_bootstrap_mappings(
    mappings: &[BootstrapMappingInput],
) -> Result<BTreeMap<(usize, usize), Ipv4Addr>, StrictCompileError> {
    let mut parsed = BTreeMap::new();
    for mapping in mappings {
        let position = (mapping.source, mapping.node);
        let ipv4 = mapping.ipv4.parse::<Ipv4Addr>().map_err(|_| {
            StrictCompileError::InvalidBootstrapMapping {
                source: mapping.source,
                node: mapping.node,
            }
        })?;
        if mapping.source == 0 || mapping.node == 0 || parsed.insert(position, ipv4).is_some() {
            return Err(StrictCompileError::InvalidBootstrapMapping {
                source: mapping.source,
                node: mapping.node,
            });
        }
    }
    Ok(parsed)
}

fn extract_proxies(text: &str, source: usize) -> Result<Vec<ParsedProxy>, StrictCompileError> {
    if let Some(proxies) = proxies_from_yaml(text) {
        return Ok(proxies);
    }
    if let Some(proxies) = proxies_from_uri_text(text, source) {
        return proxies;
    }

    let compact = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let engines = [
        &general_purpose::STANDARD,
        &general_purpose::STANDARD_NO_PAD,
        &general_purpose::URL_SAFE,
        &general_purpose::URL_SAFE_NO_PAD,
    ];
    for engine in engines {
        let Some(decoded) = engine
            .decode(compact.as_bytes())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
        else {
            continue;
        };
        if let Some(proxies) = proxies_from_yaml(&decoded) {
            return Ok(proxies);
        }
        if let Some(proxies) = proxies_from_uri_text(&decoded, source) {
            return proxies;
        }
    }

    Err(StrictCompileError::UnsupportedNodeSource(source))
}

fn proxies_from_yaml(text: &str) -> Option<Vec<ParsedProxy>> {
    let documents = YamlLoader::load_from_str(text).ok()?;
    documents
        .iter()
        .find_map(|document| document["proxies"].as_vec().cloned())
        .map(|proxies| proxies.into_iter().map(ParsedProxy::exact).collect())
}

fn proxies_from_uri_text(
    text: &str,
    source: usize,
) -> Option<Result<Vec<ParsedProxy>, StrictCompileError>> {
    let lines = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with(';'))
        .collect::<Vec<_>>();
    if lines.is_empty() || !lines.iter().any(|line| line.contains("://")) {
        return None;
    }

    let results = lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| vless_uri_to_yaml(line, source, index + 1));
    Some(collect_proxy_results(results))
}

fn collect_proxy_results(
    results: impl IntoIterator<Item = Result<ParsedProxy, StrictCompileError>>,
) -> Result<Vec<ParsedProxy>, StrictCompileError> {
    let mut proxies = Vec::new();
    let mut issues = Vec::<NodeCompileIssue>::new();
    let mut issue_total = 0usize;

    for result in results {
        match result {
            Ok(proxy) => proxies.push(proxy),
            Err(error) => {
                let (reported, total) = error.unpack_node_issues()?;
                issue_total += total;
                let remaining = MAX_REPORTED_NODE_ISSUES.saturating_sub(issues.len());
                issues.extend(reported.into_iter().take(remaining));
            }
        }
    }

    if issue_total > 0 {
        return Err(StrictCompileError::from_node_issues(issues, issue_total));
    }
    Ok(proxies)
}

pub(crate) fn preflight_uri_material(content: &[u8]) -> Option<Result<(), StrictCompileError>> {
    let text = std::str::from_utf8(content).ok()?;
    if let Some(result) = proxies_from_uri_text(text, 1) {
        return Some(result.map(|_| ()));
    }

    let compact = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let engines = [
        &general_purpose::STANDARD,
        &general_purpose::STANDARD_NO_PAD,
        &general_purpose::URL_SAFE,
        &general_purpose::URL_SAFE_NO_PAD,
    ];
    for engine in engines {
        let Some(decoded) = engine
            .decode(compact.as_bytes())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
        else {
            continue;
        };
        if let Some(result) = proxies_from_uri_text(&decoded, 1) {
            return Some(result.map(|_| ()));
        }
    }
    None
}

pub(crate) fn convert_node_text(
    content: &[u8],
) -> Result<ConvertNodeTextResult, StrictCompileError> {
    if content.len() > MAX_SOURCE_BYTES {
        return Err(StrictCompileError::SourceTooLarge(1));
    }
    let text = std::str::from_utf8(content).map_err(|_| StrictCompileError::InvalidUtf8(1))?;
    let proxies = convertible_proxies_from_material(text, 1)?;
    let mut converted = Vec::new();
    let mut allocated_names = HashMap::<String, usize>::new();
    let mut seen_fingerprints = HashSet::new();
    let mut duplicate_node_count = 0usize;
    let mut compatibility_normalization_count = 0usize;

    for proxy in proxies {
        let Yaml::Hash(mut mapping) = proxy.yaml else {
            return Err(StrictCompileError::InvalidNodeEntry { source: 1, node: 1 });
        };
        if !seen_fingerprints.insert(node_fingerprint(&mapping)) {
            duplicate_node_count += 1;
            continue;
        }
        if proxy.target_compatibility_normalization {
            compatibility_normalization_count += 1;
        }
        let original_name = string_field(&mapping, "name")
            .filter(|value| !value.trim().is_empty())
            .map(str::trim)
            .unwrap_or("节点");
        let name = allocate_name(original_name, &mut allocated_names);
        mapping.insert(Yaml::String("name".into()), Yaml::String(name));
        converted.push(Yaml::Hash(mapping));
    }

    if converted.is_empty() {
        return Err(StrictCompileError::NoNodes);
    }
    let node_count = converted.len();
    let mut warnings = Vec::new();
    if compatibility_normalization_count > 0 {
        warnings.push(format!(
            "{compatibility_normalization_count} 个节点包含目标内核无等价字段，已按兼容规则归一化。"
        ));
    }
    if duplicate_node_count > 0 {
        warnings.push(format!("已精确去重 {duplicate_node_count} 个重复节点。"));
    }

    let template = if converted.len() == 1 {
        render_single_ip_template(&converted[0])?
    } else {
        None
    };

    Ok(ConvertNodeTextResult {
        yaml: serialize_proxy_list(converted)?,
        template_yaml: template.as_ref().map(|(yaml, _)| yaml.clone()),
        template_file_name: template.map(|(_, filename)| filename),
        node_count,
        duplicate_node_count,
        compatibility_normalization_count,
        warnings,
    })
}

fn convertible_proxies_from_material(
    text: &str,
    source: usize,
) -> Result<Vec<ParsedProxy>, StrictCompileError> {
    if let Some(result) = wireguard_proxy_from_text(text, source, 1) {
        return result.map(|proxy| vec![proxy]);
    }
    if let Some(result) = proxies_from_uri_text(text, source) {
        return result;
    }

    let compact = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    let engines = [
        &general_purpose::STANDARD,
        &general_purpose::STANDARD_NO_PAD,
        &general_purpose::URL_SAFE,
        &general_purpose::URL_SAFE_NO_PAD,
    ];
    for engine in engines {
        let Some(decoded) = engine
            .decode(compact.as_bytes())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
        else {
            continue;
        };
        if let Some(result) = proxies_from_uri_text(&decoded, source) {
            return result;
        }
    }

    Err(StrictCompileError::UnsupportedNodeSource(source))
}

fn wireguard_proxy_from_text(
    text: &str,
    source: usize,
    node: usize,
) -> Option<Result<ParsedProxy, StrictCompileError>> {
    let recognized = text.lines().enumerate().any(|(index, line)| {
        let line = if index == 0 {
            line.trim().trim_start_matches('\u{feff}')
        } else {
            line.trim()
        };
        line.eq_ignore_ascii_case("[Interface]") || line.eq_ignore_ascii_case("[Peer]")
    });
    recognized.then(|| parse_wireguard_proxy(text, source, node))
}

fn parse_wireguard_proxy(
    text: &str,
    source: usize,
    node: usize,
) -> Result<ParsedProxy, StrictCompileError> {
    let mut interface = None::<BTreeMap<String, String>>;
    let mut peer = None::<BTreeMap<String, String>>;
    let mut current_section = None::<WireGuardSection>;

    for (index, raw_line) in text.lines().enumerate() {
        let line = if index == 0 {
            raw_line.trim().trim_start_matches('\u{feff}')
        } else {
            raw_line.trim()
        };
        if line.is_empty() || line == "&#x20;" || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') || line.ends_with(']') {
            if line.eq_ignore_ascii_case("[Interface]") {
                if interface.is_some() {
                    return Err(StrictCompileError::InvalidWireGuardConfig { source, node });
                }
                interface = Some(BTreeMap::new());
                current_section = Some(WireGuardSection::Interface);
                continue;
            }
            if line.eq_ignore_ascii_case("[Peer]") {
                if peer.is_some() {
                    return Err(StrictCompileError::InvalidWireGuardConfig { source, node });
                }
                peer = Some(BTreeMap::new());
                current_section = Some(WireGuardSection::Peer);
                continue;
            }
            return Err(StrictCompileError::UnsupportedWireGuardField { source, node });
        }

        let (raw_key, raw_value) = line
            .split_once('=')
            .ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?;
        let key = raw_key.trim().to_ascii_lowercase();
        let value = raw_value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(StrictCompileError::InvalidWireGuardConfig { source, node });
        }

        let section =
            current_section.ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?;
        let allowed = match section {
            WireGuardSection::Interface => {
                matches!(key.as_str(), "privatekey" | "address" | "dns")
            }
            WireGuardSection::Peer => matches!(
                key.as_str(),
                "publickey" | "endpoint" | "allowedips" | "persistentkeepalive"
            ),
        };
        if !allowed {
            return Err(StrictCompileError::UnsupportedWireGuardField { source, node });
        }

        let fields = match section {
            WireGuardSection::Interface => interface
                .as_mut()
                .ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?,
            WireGuardSection::Peer => peer
                .as_mut()
                .ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?,
        };
        if fields.insert(key, value.to_string()).is_some() {
            return Err(StrictCompileError::DuplicateWireGuardField { source, node });
        }
    }

    let interface = interface.ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?;
    let peer = peer.ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?;
    let private_key = required_wireguard_field(&interface, "privatekey", source, node)?;
    let public_key = required_wireguard_field(&peer, "publickey", source, node)?;
    if !valid_wireguard_key(private_key) || !valid_wireguard_key(public_key) {
        return Err(StrictCompileError::InvalidWireGuardConfig { source, node });
    }

    let address = required_wireguard_field(&interface, "address", source, node)?;
    let (ipv4, ipv6) = parse_wireguard_addresses(address)
        .ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?;
    let endpoint = required_wireguard_field(&peer, "endpoint", source, node)?;
    let (server, port) = parse_wireguard_endpoint(endpoint)
        .ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?;
    let allowed_ips = required_wireguard_field(&peer, "allowedips", source, node)?;
    let allowed_ips = parse_wireguard_cidr_list(allowed_ips)
        .ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?;
    let dns = match interface.get("dns") {
        Some(value) => Some(
            parse_wireguard_dns(value)
                .ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })?,
        ),
        None => None,
    };
    let persistent_keepalive = peer
        .get("persistentkeepalive")
        .map(|value| value.parse::<u16>())
        .transpose()
        .map_err(|_| StrictCompileError::InvalidWireGuardConfig { source, node })?;

    let mut peer_yaml = Hash::new();
    insert_string(&mut peer_yaml, "server", server);
    peer_yaml.insert(yaml_key("port"), Yaml::Integer(i64::from(port)));
    insert_string(&mut peer_yaml, "public-key", public_key);
    peer_yaml.insert(
        yaml_key("allowed-ips"),
        Yaml::Array(allowed_ips.into_iter().map(Yaml::String).collect()),
    );

    let mut mapping = Hash::new();
    insert_string(&mut mapping, "name", "WireGuard 1");
    insert_string(&mut mapping, "type", "wireguard");
    insert_string(&mut mapping, "ip", ipv4);
    if let Some(ipv6) = ipv6 {
        insert_string(&mut mapping, "ipv6", ipv6);
    }
    insert_string(&mut mapping, "private-key", private_key);
    mapping.insert(yaml_key("peers"), Yaml::Array(vec![Yaml::Hash(peer_yaml)]));
    mapping.insert(yaml_key("udp"), Yaml::Boolean(true));
    if let Some(persistent_keepalive) = persistent_keepalive {
        mapping.insert(
            yaml_key("persistent-keepalive"),
            Yaml::Integer(i64::from(persistent_keepalive)),
        );
    }
    if let Some(dns) = dns {
        mapping.insert(yaml_key("remote-dns-resolve"), Yaml::Boolean(true));
        mapping.insert(
            yaml_key("dns"),
            Yaml::Array(dns.into_iter().map(Yaml::String).collect()),
        );
    }

    Ok(ParsedProxy::exact(Yaml::Hash(mapping)))
}

fn required_wireguard_field<'a>(
    fields: &'a BTreeMap<String, String>,
    field: &str,
    source: usize,
    node: usize,
) -> Result<&'a str, StrictCompileError> {
    fields
        .get(field)
        .map(String::as_str)
        .ok_or(StrictCompileError::InvalidWireGuardConfig { source, node })
}

fn valid_wireguard_key(value: &str) -> bool {
    general_purpose::STANDARD
        .decode(value.as_bytes())
        .is_ok_and(|decoded| decoded.len() == 32)
}

fn parse_wireguard_addresses(value: &str) -> Option<(String, Option<String>)> {
    let mut ipv4 = None;
    let mut ipv6 = None;
    for value in comma_separated_values(value)? {
        let (address, prefix) = parse_wireguard_cidr(value)?;
        let normalized = format!("{address}/{prefix}");
        match address {
            IpAddr::V4(_) if ipv4.is_none() => ipv4 = Some(normalized),
            IpAddr::V6(_) if ipv6.is_none() => ipv6 = Some(normalized),
            _ => return None,
        }
    }
    Some((ipv4?, ipv6))
}

fn parse_wireguard_dns(value: &str) -> Option<Vec<String>> {
    comma_separated_values(value)?
        .into_iter()
        .map(|value| value.parse::<IpAddr>().ok().map(|value| value.to_string()))
        .collect()
}

fn parse_wireguard_cidr_list(value: &str) -> Option<Vec<String>> {
    let mut seen = HashSet::new();
    comma_separated_values(value)?
        .into_iter()
        .map(|value| {
            let (address, prefix) = parse_wireguard_cidr(value)?;
            let normalized = format!("{address}/{prefix}");
            seen.insert(normalized.clone()).then_some(normalized)
        })
        .collect()
}

fn comma_separated_values(value: &str) -> Option<Vec<&str>> {
    let values = value.split(',').map(str::trim).collect::<Vec<_>>();
    (!values.is_empty() && values.iter().all(|value| !value.is_empty())).then_some(values)
}

fn parse_wireguard_cidr(value: &str) -> Option<(IpAddr, u8)> {
    let (address, prefix) = value.rsplit_once('/')?;
    let address = address.parse::<IpAddr>().ok()?;
    let prefix = prefix.parse::<u8>().ok()?;
    let maximum = if address.is_ipv4() { 32 } else { 128 };
    (prefix <= maximum).then_some((address, prefix))
}

fn parse_wireguard_endpoint(value: &str) -> Option<(String, u16)> {
    let (host, port) = value.rsplit_once(':')?;
    let port = port.parse::<u16>().ok().filter(|port| *port > 0)?;
    let host = if let Some(host) = host.strip_prefix('[') {
        let host = host.strip_suffix(']')?;
        host.parse::<std::net::Ipv6Addr>().ok()?.to_string()
    } else {
        if host.is_empty() || host.contains(':') || host.chars().any(char::is_whitespace) {
            return None;
        }
        match Host::parse(host).ok()? {
            Host::Domain(host) => host,
            Host::Ipv4(host) => host.to_string(),
            Host::Ipv6(_) => return None,
        }
    };
    Some((host, port))
}

fn vless_uri_to_yaml(
    line: &str,
    source: usize,
    node: usize,
) -> Result<ParsedProxy, StrictCompileError> {
    let url =
        Url::parse(line).map_err(|_| StrictCompileError::InvalidNodeEntry { source, node })?;
    if !url.scheme().eq_ignore_ascii_case("vless") {
        return Err(StrictCompileError::UnsupportedNodeProtocol { source, node });
    }
    if url.password().is_some() {
        return Err(StrictCompileError::InvalidNodeEntry { source, node });
    }

    let uuid = percent_decode_str(url.username())
        .decode_utf8()
        .map_err(|_| StrictCompileError::InvalidNodeEntry { source, node })?
        .into_owned();
    if uuid.trim().is_empty() {
        return Err(StrictCompileError::InvalidNodeEntry { source, node });
    }
    let server = match url.host() {
        Some(Host::Domain(value)) => value.to_string(),
        Some(Host::Ipv4(value)) => value.to_string(),
        Some(Host::Ipv6(value)) => value.to_string(),
        None => return Err(StrictCompileError::InvalidNodeEntry { source, node }),
    };
    let port = url
        .port()
        .filter(|port| *port > 0)
        .ok_or(StrictCompileError::InvalidNodeEntry { source, node })?;

    let mut options = BTreeMap::<String, String>::new();
    for (key, value) in url.query_pairs() {
        if options
            .insert(key.into_owned(), value.into_owned())
            .is_some()
        {
            return Err(StrictCompileError::DuplicateNodeOption { source, node });
        }
    }

    let name = url
        .fragment()
        .and_then(|value| percent_decode_str(value).decode_utf8().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("VLESS {node}"));
    let encryption = options
        .remove("encryption")
        .unwrap_or_else(|| "none".into());
    let security = options.remove("security").unwrap_or_else(|| "none".into());
    let network = options.remove("type").unwrap_or_else(|| "tcp".into());
    let flow = take_non_empty(&mut options, "flow");
    let packet_encoding = take_non_empty(&mut options, "packetEncoding")
        .or_else(|| take_non_empty(&mut options, "packet-encoding"));
    let servername = take_matching_alias(&mut options, "sni", "servername", source, node)?;
    let client_fingerprint = take_non_empty(&mut options, "fp");
    let alpn = take_non_empty(&mut options, "alpn").map(|value| {
        value
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>()
    });
    let path = options.remove("path").filter(|value| !value.is_empty());
    let host = options.remove("host").filter(|value| !value.is_empty());
    let service_name = options
        .remove("serviceName")
        .or_else(|| options.remove("service-name"))
        .filter(|value| !value.is_empty());
    let mode = options.remove("mode").filter(|value| !value.is_empty());
    let reality_spider_path = options.remove("spx").filter(|value| !value.is_empty());
    let reality_mldsa_verify = options.remove("pqv").filter(|value| !value.is_empty());
    let skip_cert_verify = options
        .remove("allowInsecure")
        .or_else(|| options.remove("skip-cert-verify"))
        .map(|value| parse_share_bool(&value))
        .transpose()
        .map_err(|_| StrictCompileError::InvalidNodeOptionValue { source, node })?;

    if let Some(header_type) = options.remove("headerType") {
        if !header_type.is_empty() && !header_type.eq_ignore_ascii_case("none") {
            return Err(StrictCompileError::UnsupportedNodeTransportOption { source, node });
        }
    }
    let mut mapping = Hash::new();
    insert_string(&mut mapping, "name", name);
    insert_string(&mut mapping, "type", "vless");
    insert_string(&mut mapping, "server", server);
    mapping.insert(yaml_key("port"), Yaml::Integer(i64::from(port)));
    insert_string(&mut mapping, "uuid", uuid);
    mapping.insert(yaml_key("udp"), Yaml::Boolean(true));
    insert_string(
        &mut mapping,
        "encryption",
        if encryption.eq_ignore_ascii_case("none") {
            String::new()
        } else {
            encryption
        },
    );

    if let Some(flow) = flow {
        insert_string(&mut mapping, "flow", flow);
    }
    if let Some(packet_encoding) = packet_encoding {
        insert_string(&mut mapping, "packet-encoding", packet_encoding);
    }
    if let Some(servername) = servername {
        insert_string(&mut mapping, "servername", servername);
    }
    if let Some(client_fingerprint) = client_fingerprint {
        insert_string(&mut mapping, "client-fingerprint", client_fingerprint);
    }
    if let Some(values) = alpn.filter(|values| !values.is_empty()) {
        mapping.insert(
            yaml_key("alpn"),
            Yaml::Array(values.into_iter().map(Yaml::String).collect()),
        );
    }
    if let Some(skip_cert_verify) = skip_cert_verify {
        mapping.insert(
            yaml_key("skip-cert-verify"),
            Yaml::Boolean(skip_cert_verify),
        );
    }

    let tcp_mode_normalization =
        matches!(network.to_ascii_lowercase().as_str(), "tcp" | "raw") && mode.is_some();
    let mut target_compatibility_normalization = tcp_mode_normalization;
    match security.to_ascii_lowercase().as_str() {
        "none" => {
            if reality_spider_path.is_some() || reality_mldsa_verify.is_some() {
                return Err(StrictCompileError::UnsupportedNodeSecurityOption { source, node });
            }
        }
        "tls" | "xtls" => {
            if reality_spider_path.is_some() || reality_mldsa_verify.is_some() {
                return Err(StrictCompileError::UnsupportedNodeSecurityOption { source, node });
            }
            mapping.insert(yaml_key("tls"), Yaml::Boolean(true));
        }
        "reality" => {
            mapping.insert(yaml_key("tls"), Yaml::Boolean(true));
            let public_key = take_non_empty(&mut options, "pbk")
                .ok_or(StrictCompileError::InvalidNodeEntry { source, node })?;
            let mut reality = Hash::new();
            insert_string(&mut reality, "public-key", public_key);
            if let Some(short_id) = options.remove("sid") {
                insert_string(&mut reality, "short-id", short_id);
            }
            target_compatibility_normalization |=
                reality_spider_path.is_some() || reality_mldsa_verify.is_some();
            mapping.insert(yaml_key("reality-opts"), Yaml::Hash(reality));
        }
        _ => return Err(StrictCompileError::UnsupportedNodeSecurityOption { source, node }),
    }

    apply_transport(
        &mut mapping,
        &network,
        path,
        host,
        service_name,
        mode,
        source,
        node,
    )?;

    if !options.is_empty() {
        return Err(StrictCompileError::UnknownNodeOption { source, node });
    }

    Ok(ParsedProxy {
        yaml: Yaml::Hash(mapping),
        target_compatibility_normalization,
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_transport(
    mapping: &mut Hash,
    network: &str,
    path: Option<String>,
    host: Option<String>,
    service_name: Option<String>,
    mode: Option<String>,
    source: usize,
    node: usize,
) -> Result<(), StrictCompileError> {
    match network.to_ascii_lowercase().as_str() {
        "tcp" | "raw" => {
            if path.as_deref().is_some_and(|value| value != "/")
                || host.as_deref().is_some_and(|value| !value.is_empty())
                || service_name.is_some()
                || mode
                    .as_deref()
                    .is_some_and(|value| !matches!(value, "gun" | "multi" | "guna"))
            {
                return Err(StrictCompileError::UnsupportedNodeTransportOption { source, node });
            }
            insert_string(mapping, "network", "tcp");
        }
        "ws" => {
            if service_name.is_some() || mode.is_some() {
                return Err(StrictCompileError::UnsupportedNodeTransportOption { source, node });
            }
            insert_string(mapping, "network", "ws");
            let mut opts = Hash::new();
            insert_string(&mut opts, "path", path.unwrap_or_else(|| "/".into()));
            if let Some(host) = host {
                let mut headers = Hash::new();
                insert_string(&mut headers, "Host", host);
                opts.insert(yaml_key("headers"), Yaml::Hash(headers));
            }
            mapping.insert(yaml_key("ws-opts"), Yaml::Hash(opts));
        }
        "grpc" => {
            if path.is_some()
                || host.is_some()
                || mode.as_deref().is_some_and(|value| value != "gun")
            {
                return Err(StrictCompileError::UnsupportedNodeTransportOption { source, node });
            }
            insert_string(mapping, "network", "grpc");
            let mut opts = Hash::new();
            if let Some(service_name) = service_name {
                insert_string(&mut opts, "grpc-service-name", service_name);
            }
            mapping.insert(yaml_key("grpc-opts"), Yaml::Hash(opts));
        }
        "http" | "h2" => {
            if service_name.is_some() || mode.is_some() {
                return Err(StrictCompileError::UnsupportedNodeTransportOption { source, node });
            }
            insert_string(mapping, "network", "h2");
            let mut opts = Hash::new();
            insert_string(&mut opts, "path", path.unwrap_or_else(|| "/".into()));
            if let Some(host) = host {
                opts.insert(yaml_key("host"), Yaml::Array(vec![Yaml::String(host)]));
            }
            mapping.insert(yaml_key("h2-opts"), Yaml::Hash(opts));
        }
        "xhttp" => {
            if service_name.is_some() {
                return Err(StrictCompileError::UnsupportedNodeTransportOption { source, node });
            }
            insert_string(mapping, "network", "xhttp");
            let mut opts = Hash::new();
            insert_string(&mut opts, "path", path.unwrap_or_else(|| "/".into()));
            if let Some(host) = host {
                insert_string(&mut opts, "host", host);
            }
            if let Some(mode) = mode {
                if !matches!(
                    mode.as_str(),
                    "auto" | "stream-one" | "stream-up" | "packet-up"
                ) {
                    return Err(StrictCompileError::UnsupportedNodeTransportOption {
                        source,
                        node,
                    });
                }
                insert_string(&mut opts, "mode", mode);
            }
            mapping.insert(yaml_key("xhttp-opts"), Yaml::Hash(opts));
        }
        _ => return Err(StrictCompileError::UnsupportedNodeTransportOption { source, node }),
    }
    Ok(())
}

fn take_non_empty(options: &mut BTreeMap<String, String>, key: &str) -> Option<String> {
    options.remove(key).filter(|value| !value.is_empty())
}

fn take_matching_alias(
    options: &mut BTreeMap<String, String>,
    primary: &str,
    alias: &str,
    source: usize,
    node: usize,
) -> Result<Option<String>, StrictCompileError> {
    let primary_value = take_non_empty(options, primary);
    let alias_value = take_non_empty(options, alias);
    match (primary_value, alias_value) {
        (Some(primary_value), Some(alias_value)) => {
            if !primary_value.eq_ignore_ascii_case(&alias_value) {
                return Err(StrictCompileError::ConflictingNodeAlias { source, node });
            }
            Ok(Some(primary_value))
        }
        (Some(value), None) | (None, Some(value)) => Ok(Some(value)),
        (None, None) => Ok(None),
    }
}

fn parse_share_bool(value: &str) -> Result<bool, ()> {
    match value.to_ascii_lowercase().as_str() {
        "1" | "true" => Ok(true),
        "0" | "false" => Ok(false),
        _ => Err(()),
    }
}

fn yaml_key(value: &str) -> Yaml {
    Yaml::String(value.into())
}

fn insert_string(mapping: &mut Hash, key: &str, value: impl Into<String>) {
    mapping.insert(yaml_key(key), Yaml::String(value.into()));
}

fn validate_node(
    mapping: &Hash,
    source: usize,
    node: usize,
    bootstrap_ipv4: Option<Ipv4Addr>,
) -> Result<Option<(String, Ipv4Addr)>, StrictCompileError> {
    let protocol = string_field(mapping, "type")
        .ok_or(StrictCompileError::InvalidNodeEntry { source, node })?;
    let server = string_field(mapping, "server")
        .ok_or(StrictCompileError::InvalidNodeEntry { source, node })?;
    if mapping
        .get(&Yaml::String("port".into()))
        .and_then(Yaml::as_i64)
        .is_none_or(|port| !(1..=65535).contains(&port))
    {
        return Err(StrictCompileError::InvalidNodeEntry { source, node });
    }
    if matches!(protocol.to_ascii_lowercase().as_str(), "direct" | "dns") {
        return Err(StrictCompileError::UnsafeNodeOverride { source, node });
    }
    if protocol.eq_ignore_ascii_case("vless")
        && string_field(mapping, "uuid").is_none_or(|uuid| uuid.trim().is_empty())
    {
        return Err(StrictCompileError::InvalidNodeEntry { source, node });
    }

    let bootstrap_host = match server.parse::<IpAddr>() {
        Ok(IpAddr::V4(_)) => {
            if bootstrap_ipv4.is_some() {
                return Err(StrictCompileError::InvalidBootstrapMapping { source, node });
            }
            None
        }
        Ok(IpAddr::V6(_)) => {
            return Err(StrictCompileError::Ipv6NodeEndpoint { source, node });
        }
        Err(_) => {
            let ipv4 = bootstrap_ipv4
                .ok_or(StrictCompileError::UnprotectedNodeBootstrap { source, node })?;
            let domain = server.trim().to_ascii_lowercase();
            if domain.is_empty() || domain.chars().any(char::is_whitespace) || !domain.contains('.')
            {
                return Err(StrictCompileError::InvalidNodeEntry { source, node });
            }
            Some((domain, ipv4))
        }
    };

    for unsafe_field in ["interface-name", "routing-mark", "dialer-proxy"] {
        if mapping.contains_key(&Yaml::String(unsafe_field.into())) {
            return Err(StrictCompileError::UnsafeNodeOverride { source, node });
        }
    }
    if string_field(mapping, "ip-version")
        .is_some_and(|value| matches!(value.to_ascii_lowercase().as_str(), "ipv6" | "ipv6-prefer"))
    {
        return Err(StrictCompileError::Ipv6NodeEndpoint { source, node });
    }

    Ok(bootstrap_host)
}

fn string_field<'a>(mapping: &'a Hash, field: &str) -> Option<&'a str> {
    mapping
        .get(&Yaml::String(field.to_string()))
        .and_then(Yaml::as_str)
}

fn allocate_name(base: &str, allocated: &mut HashMap<String, usize>) -> String {
    let next = allocated.entry(base.to_string()).or_insert(0);
    *next += 1;
    if *next == 1 {
        base.to_string()
    } else {
        format!("{base} [{}]", *next)
    }
}

fn node_fingerprint(mapping: &Hash) -> String {
    let mut entries = mapping
        .iter()
        .filter(|(key, _)| key.as_str().is_none_or(|value| value != "name"))
        .map(|(key, value)| (canonical_yaml(key), canonical_yaml(value)))
        .collect::<Vec<_>>();
    entries.sort();
    format!("{:x}", Sha256::digest(format!("{entries:?}").as_bytes()))
}

fn canonical_yaml(value: &Yaml) -> String {
    match value {
        Yaml::Hash(mapping) => {
            let mut entries = mapping
                .iter()
                .map(|(key, value)| (canonical_yaml(key), canonical_yaml(value)))
                .collect::<Vec<_>>();
            entries.sort();
            format!("{entries:?}")
        }
        Yaml::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(canonical_yaml)
                .collect::<Vec<_>>()
                .join(",")
        ),
        other => format!("{other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn material(content: &str) -> ResolvedSourceMaterial {
        ResolvedSourceMaterial {
            content: content.as_bytes().to_vec(),
        }
    }

    fn fixture_wireguard_key(fill: u8) -> String {
        general_purpose::STANDARD.encode([fill; 32])
    }

    fn wireguard_fixture() -> (String, String, String) {
        let private_key = fixture_wireguard_key(b'P');
        let public_key = fixture_wireguard_key(b'K');
        let content = format!(
            r#"
[Interface]
PrivateKey = {private_key}
Address = 192.0.2.2/32, 2001:db8::2/128
DNS = 192.0.2.53, 2001:db8::53

[Peer]
PublicKey = {public_key}
Endpoint = wg-node.example.invalid:51820
AllowedIPs = 0.0.0.0/0, ::/0
PersistentKeepalive = 25
&#x20;
"#
        );
        (content, private_key, public_key)
    }

    #[test]
    fn converts_reality_vless_to_a_mihomo_proxy_list() {
        let result = convert_node_text(
            concat!(
                "vless://00000000-0000-4000-8000-000000000001@node.example.invalid:443?",
                "encryption=none&fp=chrome&",
                "pbk=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&security=reality&",
                "sid=0123456789abcdef&sni=cover.example.invalid&",
                "spx=%2Ffixture&type=tcp#Fixture"
            )
            .as_bytes(),
        )
        .expect("fixture VLESS should convert");
        let document =
            &YamlLoader::load_from_str(&result.yaml).expect("converted YAML should parse")[0];
        let proxies = document["proxies"].as_vec().expect("proxy list");

        assert_eq!(result.node_count, 1);
        assert_eq!(result.compatibility_normalization_count, 1);
        assert_eq!(proxies[0]["type"].as_str(), Some("vless"));
        assert_eq!(proxies[0]["network"].as_str(), Some("tcp"));
        assert_eq!(
            proxies[0]["servername"].as_str(),
            Some("cover.example.invalid")
        );
        assert_eq!(proxies[0]["client-fingerprint"].as_str(), Some("chrome"));
        assert_eq!(
            proxies[0]["reality-opts"]["short-id"].as_str(),
            Some("0123456789abcdef")
        );
    }

    #[test]
    fn conversion_deduplicates_exact_nodes_and_allocates_duplicate_names() {
        let first = concat!(
            "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
            "encryption=none&security=tls&type=tcp#Fixture"
        );
        let second = concat!(
            "vless://00000000-0000-4000-8000-000000000002@192.0.2.11:443?",
            "encryption=none&security=tls&type=tcp#Fixture"
        );
        let result = convert_node_text(format!("{first}\n{first}\n{second}").as_bytes())
            .expect("fixture nodes should convert");
        let document =
            &YamlLoader::load_from_str(&result.yaml).expect("converted YAML should parse")[0];
        let proxies = document["proxies"].as_vec().expect("proxy list");

        assert_eq!(result.node_count, 2);
        assert_eq!(result.duplicate_node_count, 1);
        assert_eq!(proxies[0]["name"].as_str(), Some("Fixture"));
        assert_eq!(proxies[1]["name"].as_str(), Some("Fixture [2]"));
    }

    #[test]
    fn conversion_accepts_base64_vless_material() {
        let content = concat!(
            "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
            "encryption=none&security=tls&type=tcp#Fixture"
        );
        let encoded = general_purpose::STANDARD.encode(content);
        let result = convert_node_text(encoded.as_bytes()).expect("Base64 fixture should convert");
        assert_eq!(result.node_count, 1);
        assert!(result.yaml.starts_with("---\nproxies:"));
    }

    #[test]
    fn single_ip_node_renders_complete_template_with_matching_group_references() {
        let result = convert_node_text(
            concat!(
                "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
                "encryption=none&security=tls&type=tcp#Original"
            )
            .as_bytes(),
        )
        .expect("fixture should convert");
        let contract = serde_json::to_value(&result).expect("result serializes");
        let template = contract["templateYaml"]
            .as_str()
            .expect("complete template");
        let document = &YamlLoader::load_from_str(template).expect("template parses")[0];
        let proxies = document["proxies"].as_vec().expect("proxy list");
        let groups = document["proxy-groups"].as_vec().expect("groups");

        assert_eq!(contract["templateFileName"], "192.0.2.10.yaml");
        assert_eq!(document["mixed-port"].as_i64(), Some(7890));
        assert_eq!(proxies.len(), 1);
        assert_eq!(proxies[0]["name"].as_str(), Some("192.0.2.10"));
        assert_eq!(proxies[0]["server"].as_str(), Some("192.0.2.10"));
        assert_eq!(
            proxies[0]["uuid"].as_str(),
            Some("00000000-0000-4000-8000-000000000001")
        );
        assert_eq!(document["rules"].as_vec().map(Vec::len), Some(49));
        assert!(template.contains("# 自动测速选择"));
        assert_eq!(
            groups
                .iter()
                .filter(
                    |group| group["proxies"].as_vec().is_some_and(|members| members
                        .iter()
                        .any(|member| member.as_str() == Some("192.0.2.10")))
                )
                .count(),
            5
        );
        assert!(!template.contains("Original"));
        assert!(!serde_json::to_string(&result.warnings)
            .unwrap()
            .contains("192.0.2.10"));
    }

    #[test]
    fn multiple_or_domain_nodes_do_not_offer_single_ip_template() {
        let domain = "vless://00000000-0000-4000-8000-000000000001@node.example.invalid:443?encryption=none&security=tls&type=tcp#192.0.2.10";
        let another = "vless://00000000-0000-4000-8000-000000000002@192.0.2.11:443?encryption=none&security=tls&type=tcp#Node";
        for input in [domain.to_string(), format!("{domain}\n{another}")] {
            let result = convert_node_text(input.as_bytes()).expect("fixture should convert");
            let contract = serde_json::to_value(&result).expect("result serializes");
            assert!(contract["templateYaml"].is_null());
            assert!(contract["templateFileName"].is_null());
        }
    }

    #[test]
    fn conversion_errors_do_not_echo_share_secrets() {
        let result = convert_node_text(
            concat!(
                "vless://PRIVATE_UUID@192.0.2.10:443?security=tls&",
                "PRIVATE_QUERY_KEY=PRIVATE_QUERY_VALUE#PRIVATE_NODE"
            )
            .as_bytes(),
        );
        let error = match result {
            Ok(_) => panic!("unknown option must fail"),
            Err(error) => error,
        };
        let rendered = error.to_string();
        assert!(rendered.contains("未知分享参数"));
        for secret in [
            "PRIVATE_UUID",
            "PRIVATE_QUERY_KEY",
            "PRIVATE_QUERY_VALUE",
            "PRIVATE_NODE",
        ] {
            assert!(!rendered.contains(secret));
        }
    }

    #[test]
    fn converts_single_peer_wireguard_to_full_mihomo_syntax() {
        let (content, private_key, public_key) = wireguard_fixture();
        let result =
            convert_node_text(content.as_bytes()).expect("fixture WireGuard should convert");
        let document =
            &YamlLoader::load_from_str(&result.yaml).expect("converted YAML should parse")[0];
        let proxies = document["proxies"].as_vec().expect("proxy list");
        let proxy = &proxies[0];
        let peers = proxy["peers"].as_vec().expect("WireGuard peers");
        let allowed_ips = peers[0]["allowed-ips"].as_vec().expect("allowed IPs");
        let dns = proxy["dns"].as_vec().expect("WireGuard DNS list");

        assert_eq!(result.node_count, 1);
        assert_eq!(result.duplicate_node_count, 0);
        assert_eq!(result.compatibility_normalization_count, 0);
        assert!(result.warnings.is_empty());
        assert_eq!(proxy["name"].as_str(), Some("WireGuard 1"));
        assert_eq!(proxy["type"].as_str(), Some("wireguard"));
        assert_eq!(proxy["ip"].as_str(), Some("192.0.2.2/32"));
        assert_eq!(proxy["ipv6"].as_str(), Some("2001:db8::2/128"));
        assert_eq!(proxy["private-key"].as_str(), Some(private_key.as_str()));
        assert_eq!(proxy["udp"].as_bool(), Some(true));
        assert_eq!(proxy["persistent-keepalive"].as_i64(), Some(25));
        assert_eq!(proxy["remote-dns-resolve"].as_bool(), Some(true));
        assert_eq!(dns[0].as_str(), Some("192.0.2.53"));
        assert_eq!(dns[1].as_str(), Some("2001:db8::53"));
        assert_eq!(peers[0]["server"].as_str(), Some("wg-node.example.invalid"));
        assert_eq!(peers[0]["port"].as_i64(), Some(51820));
        assert_eq!(peers[0]["public-key"].as_str(), Some(public_key.as_str()));
        assert_eq!(allowed_ips[0].as_str(), Some("0.0.0.0/0"));
        assert_eq!(allowed_ips[1].as_str(), Some("::/0"));
    }

    #[test]
    fn wireguard_failures_are_closed_and_do_not_echo_values() {
        let (valid, private_key, public_key) = wireguard_fixture();
        let duplicate = valid.replacen(
            "Address = 192.0.2.2/32, 2001:db8::2/128",
            "Address = 192.0.2.2/32\nAddress = 192.0.2.3/32",
            1,
        );
        let unknown = valid.replacen(
            "DNS = 192.0.2.53, 2001:db8::53",
            "DNS = 192.0.2.53\nListenPort = 51821",
            1,
        );
        let invalid_value = valid.replacen(
            "Endpoint = wg-node.example.invalid:51820",
            "Endpoint = private-endpoint.example.invalid:0",
            1,
        );
        let invalid_cidr = valid.replacen(
            "Address = 192.0.2.2/32, 2001:db8::2/128",
            "Address = PRIVATE_ADDRESS_VALUE",
            1,
        );
        let invalid_endpoint = valid.replacen(
            "Endpoint = wg-node.example.invalid:51820",
            "Endpoint = PRIVATE_ENDPOINT_WITHOUT_PORT",
            1,
        );
        let invalid_key = valid.replacen(
            &format!("PrivateKey = {private_key}"),
            "PrivateKey = PRIVATE_KEY_PLACEHOLDER",
            1,
        );
        let missing = valid.replacen("AllowedIPs = 0.0.0.0/0, ::/0\n", "", 1);
        let multiple_peers = format!(
            "{valid}\n[Peer]\nPublicKey = {public_key}\nEndpoint = second.example.invalid:51820\nAllowedIPs = 192.0.2.0/24\n"
        );

        let cases = [
            (
                duplicate,
                StrictCompileError::DuplicateWireGuardField { source: 1, node: 1 },
            ),
            (
                unknown,
                StrictCompileError::UnsupportedWireGuardField { source: 1, node: 1 },
            ),
            (
                invalid_value,
                StrictCompileError::InvalidWireGuardConfig { source: 1, node: 1 },
            ),
            (
                invalid_cidr,
                StrictCompileError::InvalidWireGuardConfig { source: 1, node: 1 },
            ),
            (
                invalid_endpoint,
                StrictCompileError::InvalidWireGuardConfig { source: 1, node: 1 },
            ),
            (
                invalid_key,
                StrictCompileError::InvalidWireGuardConfig { source: 1, node: 1 },
            ),
            (
                missing,
                StrictCompileError::InvalidWireGuardConfig { source: 1, node: 1 },
            ),
            (
                multiple_peers,
                StrictCompileError::InvalidWireGuardConfig { source: 1, node: 1 },
            ),
        ];

        for (content, expected) in cases {
            let error = convert_node_text(content.as_bytes())
                .err()
                .expect("incompatible WireGuard must fail");
            let rendered = error.to_string();
            assert_eq!(error, expected);
            for secret in [
                private_key.as_str(),
                public_key.as_str(),
                "wg-node.example.invalid",
                "private-endpoint.example.invalid",
                "PRIVATE_ADDRESS_VALUE",
                "PRIVATE_ENDPOINT_WITHOUT_PORT",
                "PRIVATE_KEY_PLACEHOLDER",
                "second.example.invalid",
                "192.0.2.2",
                "192.0.2.53",
            ] {
                assert!(!rendered.contains(secret));
            }
        }
    }

    #[test]
    fn compiles_ipv4_yaml_nodes_and_deduplicates_renames() {
        let source = material(
            r#"
proxies:
  - name: Fixture A
    type: ss
    server: 192.0.2.10
    port: 443
    cipher: aes-128-gcm
    password: PLACEHOLDER_PASSWORD
  - name: Fixture Renamed
    type: ss
    server: 192.0.2.10
    port: 443
    cipher: aes-128-gcm
    password: PLACEHOLDER_PASSWORD
"#,
        );
        let compiled = compile_nodes(&[source], &[]).expect("fixture nodes compile");
        assert_eq!(compiled.nodes.len(), 1);
        assert_eq!(compiled.duplicate_count, 1);
    }

    #[test]
    fn rejects_domain_bootstrap_without_a_protected_path() {
        let source = material(
            r#"
proxies:
  - name: Fixture
    type: ss
    server: node.example.invalid
    port: 443
    cipher: aes-128-gcm
    password: PLACEHOLDER_PASSWORD
"#,
        );
        assert_eq!(
            compile_nodes(&[source], &[])
                .err()
                .expect("domain bootstrap must fail"),
            StrictCompileError::UnprotectedNodeBootstrap { source: 1, node: 1 }
        );
    }

    #[test]
    fn compiles_domain_bootstrap_with_a_user_verified_ipv4_mapping() {
        let source = material(
            r#"
proxies:
  - name: Fixture
    type: ss
    server: node.example.invalid
    port: 443
    cipher: aes-128-gcm
    password: PLACEHOLDER_PASSWORD
"#,
        );
        let mappings = vec![BootstrapMappingInput {
            source: 1,
            node: 1,
            ipv4: "192.0.2.10".into(),
        }];

        let compiled = compile_nodes(&[source], &mappings).expect("mapped domain node compiles");

        assert_eq!(
            compiled.bootstrap_hosts.get("node.example.invalid"),
            Some(&"192.0.2.10".to_string())
        );
        let Yaml::Hash(mapping) = &compiled.nodes[0].yaml else {
            panic!("compiled node must remain a mapping");
        };
        assert_eq!(
            string_field(mapping, "server"),
            Some("node.example.invalid")
        );
    }

    #[test]
    fn rejects_stale_or_invalid_bootstrap_mappings() {
        let source = material(
            r#"
proxies:
  - name: Fixture
    type: ss
    server: 192.0.2.10
    port: 443
    cipher: aes-128-gcm
    password: PLACEHOLDER_PASSWORD
"#,
        );
        let mappings = vec![BootstrapMappingInput {
            source: 1,
            node: 1,
            ipv4: "192.0.2.11".into(),
        }];

        assert_eq!(
            compile_nodes(&[source], &mappings)
                .err()
                .expect("mapping for literal node must fail"),
            StrictCompileError::InvalidBootstrapMapping { source: 1, node: 1 }
        );

        let unsafe_domain = material(
            r#"
proxies:
  - name: Fixture
    type: ss
    server: node.example.invalid
    port: 443
    cipher: aes-128-gcm
    password: PLACEHOLDER_PASSWORD
    dialer-proxy: SECRET_OUTER_PROXY
"#,
        );
        assert_eq!(
            compile_nodes(&[unsafe_domain], &mappings)
                .err()
                .expect("mapped domain must still reject unsafe overrides"),
            StrictCompileError::UnsafeNodeOverride { source: 1, node: 1 }
        );
    }

    #[test]
    fn compiles_vless_reality_uri_without_exposing_it_to_errors() {
        let source = material(concat!(
            "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
            "encryption=none&security=reality&sni=fixture.example.invalid&fp=chrome&",
            "type=tcp&flow=xtls-rprx-vision&pbk=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&",
            "pqv=PLACEHOLDER_MLDSA_VERIFY&sid=0123456789abcdef&",
            "spx=%2Ffixture-spider#Fixture%20VLESS"
        ));

        let compiled = compile_nodes(&[source], &[]).expect("VLESS URI compiles into Node IR");

        assert_eq!(compiled.nodes.len(), 1);
        assert_eq!(compiled.nodes[0].name, "Fixture VLESS");
        let Yaml::Hash(mapping) = &compiled.nodes[0].yaml else {
            panic!("compiled VLESS node must be a mapping");
        };
        assert_eq!(string_field(mapping, "type"), Some("vless"));
        assert_eq!(string_field(mapping, "network"), Some("tcp"));
        assert!(mapping.contains_key(&yaml_key("reality-opts")));
        assert_eq!(compiled.target_compatibility_normalization_count, 1);
        let rendered = format!("{mapping:?}");
        assert!(!rendered.contains("PLACEHOLDER_MLDSA_VERIFY"));
        assert!(!rendered.contains("fixture-spider"));
    }

    #[test]
    fn normalizes_provider_servername_alias_and_tcp_grpc_mode() {
        let source = material(concat!(
            "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
            "mode=multi&security=reality&encryption=none&type=tcp&",
            "flow=xtls-rprx-vision&pbk=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&",
            "sid=0123456789abcdef&sni=fixture.example.invalid&",
            "servername=fixture.example.invalid&fp=chrome#Fixture"
        ));

        let compiled = compile_nodes(&[source], &[]).expect("provider-shaped URI compiles");
        let Yaml::Hash(mapping) = &compiled.nodes[0].yaml else {
            panic!("compiled VLESS node must be a mapping");
        };

        assert_eq!(string_field(mapping, "network"), Some("tcp"));
        assert_eq!(
            string_field(mapping, "servername"),
            Some("fixture.example.invalid")
        );
        assert_eq!(compiled.target_compatibility_normalization_count, 1);
        assert!(!format!("{mapping:?}").contains("multi"));
    }

    #[test]
    fn rejects_conflicting_sni_and_servername_aliases() {
        let source = material(concat!(
            "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
            "security=tls&type=tcp&sni=one.example.invalid&",
            "servername=two.example.invalid#Fixture"
        ));

        assert_eq!(
            compile_nodes(&[source], &[])
                .err()
                .expect("conflicting aliases must fail"),
            StrictCompileError::ConflictingNodeAlias { source: 1, node: 1 }
        );
    }

    #[test]
    fn rejects_reality_only_compatibility_extensions_outside_reality() {
        let source = material(concat!(
            "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
            "encryption=none&security=tls&type=tcp&pqv=PLACEHOLDER_MLDSA_VERIFY#Fixture"
        ));

        assert_eq!(
            compile_nodes(&[source], &[])
                .err()
                .expect("REALITY-only extension on TLS must fail"),
            StrictCompileError::UnsupportedNodeSecurityOption { source: 1, node: 1 }
        );
    }

    #[test]
    fn compiles_base64_wrapped_vless_uri_list() {
        let uri = concat!(
            "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
            "encryption=none&security=tls&sni=fixture.example.invalid&type=ws&",
            "host=fixture.example.invalid&path=%2Fws#Fixture"
        );
        let encoded = general_purpose::STANDARD.encode(uri);

        let compiled = compile_nodes(&[material(&encoded)], &[]).expect("Base64 VLESS compiles");

        assert_eq!(compiled.nodes.len(), 1);
        let Yaml::Hash(mapping) = &compiled.nodes[0].yaml else {
            panic!("compiled VLESS node must be a mapping");
        };
        assert_eq!(string_field(mapping, "network"), Some("ws"));
        assert!(mapping.contains_key(&yaml_key("ws-opts")));
    }

    #[test]
    fn rejects_recognized_but_unmapped_protocol_without_echoing_the_uri() {
        let source = material("trojan://PLACEHOLDER_PASSWORD@192.0.2.10:443#Fixture");

        assert_eq!(
            compile_nodes(&[source], &[])
                .err()
                .expect("unmapped protocol must fail closed"),
            StrictCompileError::UnsupportedNodeProtocol { source: 1, node: 1 }
        );
    }

    #[test]
    fn rejects_duplicate_or_unmapped_vless_options() {
        let duplicated = material(concat!(
            "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
            "security=tls&security=reality#Fixture"
        ));
        assert_eq!(
            compile_nodes(&[duplicated], &[])
                .err()
                .expect("duplicate query key must fail"),
            StrictCompileError::DuplicateNodeOption { source: 1, node: 1 }
        );

        let unmapped = material(concat!(
            "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
            "security=tls&secretFutureOption=PLACEHOLDER#Fixture"
        ));
        assert_eq!(
            compile_nodes(&[unmapped], &[])
                .err()
                .expect("unmapped query key must fail"),
            StrictCompileError::UnknownNodeOption { source: 1, node: 1 }
        );
    }

    #[test]
    fn aggregates_multiple_uri_node_errors_without_echoing_secrets() {
        let source = material(concat!(
            "vless://PRIVATE_UUID_ONE@192.0.2.10:443?security=tls&",
            "PRIVATE_QUERY_KEY=PRIVATE_QUERY_VALUE#PRIVATE_NODE_ONE\n",
            "vless://PRIVATE_UUID_TWO@192.0.2.11:443?security=tls&",
            "type=ws&mode=PRIVATE_MODE#PRIVATE_NODE_TWO"
        ));

        let error = compile_nodes(&[source], &[])
            .err()
            .expect("both incompatible nodes must be reported");
        let rendered = error.to_string();

        assert!(matches!(
            error,
            StrictCompileError::NodeIssues { total: 2, .. }
        ));
        assert!(rendered.contains("发现 2 个节点问题"));
        assert!(rendered.contains("第 1 个来源的第 1 个节点包含未知分享参数"));
        assert!(rendered.contains("第 1 个来源的第 2 个节点包含与传输类型不兼容"));
        for secret in [
            "PRIVATE_UUID_ONE",
            "PRIVATE_UUID_TWO",
            "PRIVATE_QUERY_KEY",
            "PRIVATE_QUERY_VALUE",
            "PRIVATE_MODE",
            "PRIVATE_NODE_ONE",
            "PRIVATE_NODE_TWO",
        ] {
            assert!(!rendered.contains(secret));
        }
    }

    #[test]
    fn caps_reported_node_details_but_preserves_the_total() {
        let source = material(
            &(1..=12)
                .map(|index| {
                    format!(
                        "vless://PRIVATE_UUID_{index}@192.0.2.10:443?security=tls&PRIVATE_KEY_{index}=PRIVATE_VALUE_{index}#PRIVATE_NODE_{index}"
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        );

        let error = compile_nodes(&[source], &[])
            .err()
            .expect("incompatible nodes must fail");
        let StrictCompileError::NodeIssues { issues, total } = &error else {
            panic!("multiple errors must use the aggregate variant");
        };

        assert_eq!(*total, 12);
        assert_eq!(issues.len(), MAX_REPORTED_NODE_ISSUES);
        let rendered = error.to_string();
        assert!(rendered.contains("另有 4 个"));
        assert!(rendered.chars().count() <= 500);
        assert!(!rendered.contains("PRIVATE_KEY"));
        assert!(!rendered.contains("PRIVATE_VALUE"));
    }
}
