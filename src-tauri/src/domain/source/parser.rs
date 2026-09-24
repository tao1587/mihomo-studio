use std::collections::{BTreeMap, HashSet};

use base64::{engine::general_purpose, Engine as _};
use sha2::{Digest, Sha256};
use url::Url;
use yaml_rust2::{Yaml, YamlLoader};

use super::model::{
    ContentFormat, ParsedContent, ParsedNode, ProtocolCount, SourceInspectionSummary,
};

pub(crate) const MAX_SOURCE_BYTES: usize = 8 * 1024 * 1024;
const MAX_PARSE_DEPTH: usize = 1;
const SUPPORTED_PROTOCOLS: &[&str] = &[
    "vless",
    "vmess",
    "trojan",
    "ss",
    "ssr",
    "hysteria",
    "hysteria2",
    "hy2",
    "tuic",
    "wireguard",
    "wg",
    "socks",
    "socks5",
    "http",
    "https",
    "snell",
    "anytls",
    "mieru",
    "ssh",
];

pub(crate) fn inspect_bytes(bytes: &[u8]) -> ParsedContent {
    if bytes.len() > MAX_SOURCE_BYTES {
        return ParsedContent {
            format: ContentFormat::Unknown,
            nodes: Vec::new(),
            warnings: vec!["内容超过 8 MiB 安全上限。".to_string()],
            requires_mihomo_resolver: false,
        };
    }

    let Ok(text) = std::str::from_utf8(bytes) else {
        return ParsedContent {
            format: ContentFormat::Unknown,
            nodes: Vec::new(),
            warnings: vec!["响应不是 UTF-8 文本，可能是二进制 provider。".to_string()],
            requires_mihomo_resolver: true,
        };
    };

    inspect_text_inner(text, 0)
}

pub(crate) fn inspect_node_text(content: &str) -> ParsedContent {
    if content.len() > MAX_SOURCE_BYTES {
        return ParsedContent {
            format: ContentFormat::Unknown,
            nodes: Vec::new(),
            warnings: vec!["节点文本超过 8 MiB 安全上限。".to_string()],
            requires_mihomo_resolver: false,
        };
    }
    inspect_text_inner(content, 0)
}

fn inspect_text_inner(text: &str, depth: usize) -> ParsedContent {
    let trimmed = text.trim().trim_start_matches('\u{feff}');
    if trimmed.is_empty() {
        return unknown("内容为空。", false);
    }

    let html_probe = trimmed
        .chars()
        .take(512)
        .collect::<String>()
        .to_ascii_lowercase();
    if html_probe.contains("<!doctype html") || html_probe.contains("<html") {
        return unknown("响应是 HTML 页面，不是订阅内容。", false);
    }

    if let Some(parsed) = parse_uri_lines(trimmed) {
        return parsed;
    }
    if let Some(parsed) = parse_mihomo_yaml(trimmed) {
        return parsed;
    }
    if depth < MAX_PARSE_DEPTH {
        if let Some(decoded) = decode_base64_text(trimmed) {
            let mut parsed = inspect_text_inner(&decoded, depth + 1);
            match parsed.format {
                ContentFormat::UriList => parsed.format = ContentFormat::Base64UriList,
                ContentFormat::MihomoYaml | ContentFormat::MihomoProviderYaml => {
                    parsed.format = ContentFormat::Base64MihomoYaml;
                }
                _ => {}
            }
            if parsed.format.is_recognized() {
                return parsed;
            }
        }
    }

    unknown("内容格式暂未识别，将交给短生命周期 Mihomo resolver。", true)
}

fn parse_uri_lines(text: &str) -> Option<ParsedContent> {
    let mut nodes = Vec::new();
    let mut unsupported_lines = 0usize;
    let mut candidate_lines = 0usize;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        candidate_lines += 1;
        if let Some(node) = parse_proxy_uri(line) {
            nodes.push(node);
        } else {
            unsupported_lines += 1;
        }
    }

    if nodes.is_empty() {
        return None;
    }

    let mut warnings = Vec::new();
    if unsupported_lines > 0 {
        warnings.push(format!(
            "忽略 {unsupported_lines} 行暂不支持的内容；已识别 {}/{} 行。",
            nodes.len(),
            candidate_lines
        ));
    }

    Some(ParsedContent {
        format: ContentFormat::UriList,
        nodes,
        warnings,
        requires_mihomo_resolver: unsupported_lines > 0,
    })
}

fn parse_proxy_uri(line: &str) -> Option<ParsedNode> {
    let (scheme, _) = line.split_once("://")?;
    let protocol = scheme.to_ascii_lowercase();
    if !SUPPORTED_PROTOCOLS.contains(&protocol.as_str()) {
        return None;
    }

    let fingerprint_input = normalize_uri_for_fingerprint(line);
    Some(ParsedNode {
        protocol: normalize_protocol(&protocol).to_string(),
        fingerprint: sha256_hex(fingerprint_input.as_bytes()),
    })
}

fn normalize_uri_for_fingerprint(line: &str) -> String {
    if let Ok(mut url) = Url::parse(line) {
        url.set_fragment(None);
        if url.query().is_some() {
            let mut pairs = url
                .query_pairs()
                .map(|(key, value)| (key.into_owned(), value.into_owned()))
                .collect::<Vec<_>>();
            pairs.sort();
            url.set_query(None);
            if !pairs.is_empty() {
                url.query_pairs_mut().extend_pairs(pairs);
            }
        }
        return url.to_string();
    }

    line.split('#').next().unwrap_or(line).trim().to_string()
}

fn parse_mihomo_yaml(text: &str) -> Option<ParsedContent> {
    let documents = YamlLoader::load_from_str(text).ok()?;
    for document in &documents {
        if let Some(proxies) = document["proxies"].as_vec() {
            let mut nodes = Vec::new();
            let mut unknown_types = BTreeMap::<String, usize>::new();
            for proxy in proxies {
                let Some(protocol) = proxy["type"].as_str() else {
                    *unknown_types.entry("missing-type".to_string()).or_default() += 1;
                    continue;
                };
                let normalized = normalize_protocol(&protocol.to_ascii_lowercase()).to_string();
                if !SUPPORTED_PROTOCOLS.contains(&protocol.to_ascii_lowercase().as_str()) {
                    *unknown_types.entry(protocol.to_string()).or_default() += 1;
                }
                nodes.push(ParsedNode {
                    protocol: normalized,
                    fingerprint: yaml_proxy_fingerprint(proxy),
                });
            }

            let mut warnings = Vec::new();
            if proxies.is_empty() {
                warnings.push("YAML 中的 proxies 列表为空。".to_string());
            }
            if !unknown_types.is_empty() {
                warnings.push(format!(
                    "存在 {} 个需要 Mihomo resolver 确认的协议条目。",
                    unknown_types.values().sum::<usize>()
                ));
            }
            return Some(ParsedContent {
                format: ContentFormat::MihomoYaml,
                nodes,
                warnings,
                requires_mihomo_resolver: !unknown_types.is_empty(),
            });
        }

        if !document["proxy-providers"].is_badvalue() {
            return Some(ParsedContent {
                format: ContentFormat::MihomoProviderYaml,
                nodes: Vec::new(),
                warnings: vec!["检测到 proxy-providers；节点枚举需要 Mihomo resolver。".to_string()],
                requires_mihomo_resolver: true,
            });
        }
    }
    None
}

fn yaml_proxy_fingerprint(proxy: &Yaml) -> String {
    sha256_hex(canonical_yaml(proxy, true).as_bytes())
}

fn canonical_yaml(value: &Yaml, omit_proxy_name: bool) -> String {
    match value {
        Yaml::Hash(mapping) => {
            let mut entries = mapping
                .iter()
                .filter(|(key, _)| {
                    !(omit_proxy_name && key.as_str().is_some_and(|value| value == "name"))
                })
                .map(|(key, value)| (canonical_yaml(key, false), canonical_yaml(value, false)))
                .collect::<Vec<_>>();
            entries.sort();
            format!("{{{:?}}}", entries)
        }
        Yaml::Array(values) => format!(
            "[{}]",
            values
                .iter()
                .map(|value| canonical_yaml(value, false))
                .collect::<Vec<_>>()
                .join(",")
        ),
        other => format!("{other:?}"),
    }
}

fn decode_base64_text(text: &str) -> Option<String> {
    let compact = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();
    if compact.len() < 8
        || !compact.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'-' | b'_' | b'=')
        })
    {
        return None;
    }

    let engines = [
        &general_purpose::STANDARD,
        &general_purpose::STANDARD_NO_PAD,
        &general_purpose::URL_SAFE,
        &general_purpose::URL_SAFE_NO_PAD,
    ];
    engines.iter().find_map(|engine| {
        engine
            .decode(compact.as_bytes())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    })
}

fn normalize_protocol(protocol: &str) -> &str {
    match protocol {
        "hy2" => "hysteria2",
        "wg" => "wireguard",
        "socks5" => "socks",
        value => value,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn unknown(warning: &str, requires_mihomo_resolver: bool) -> ParsedContent {
    ParsedContent {
        format: ContentFormat::Unknown,
        nodes: Vec::new(),
        warnings: vec![warning.to_string()],
        requires_mihomo_resolver,
    }
}

pub(crate) fn summarize(
    parsed: ParsedContent,
    safe_label: String,
    user_agent: Option<String>,
) -> SourceInspectionSummary {
    let format_ready_for_compilation = parsed.format_ready_for_compilation();
    let mut seen = HashSet::new();
    let mut protocols = BTreeMap::<String, usize>::new();
    let mut duplicate_count = 0usize;

    for node in parsed.nodes {
        if !seen.insert(node.fingerprint) {
            duplicate_count += 1;
            continue;
        }
        *protocols.entry(node.protocol).or_default() += 1;
    }

    SourceInspectionSummary {
        safe_label,
        source_format: parsed.format.as_str().to_string(),
        node_count: seen.len(),
        duplicate_count,
        protocols: protocols
            .into_iter()
            .map(|(protocol, count)| ProtocolCount { protocol, count })
            .collect(),
        user_agent,
        warnings: parsed.warnings,
        requires_mihomo_resolver: parsed.requires_mihomo_resolver,
        format_ready_for_compilation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_deduplicates_mixed_uri_lines() {
        let input = "vless://id@example.com:443?security=tls#One\n\
                     vless://id@example.com:443?security=tls#Renamed\n\
                     trojan://secret@example.net:443#Two";
        let summary = summarize(inspect_node_text(input), "local".into(), None);

        assert_eq!(summary.source_format, "uri-list");
        assert_eq!(summary.node_count, 2);
        assert_eq!(summary.duplicate_count, 1);
        assert_eq!(summary.protocols.len(), 2);
        assert!(!summary.format_ready_for_compilation);
    }

    #[test]
    fn marks_vless_uri_and_base64_vless_as_format_ready() {
        let input = "vless://id@192.0.2.10:443?security=tls#One";
        let plain = summarize(inspect_node_text(input), "local".into(), None);
        let encoded = summarize(
            inspect_node_text(&general_purpose::STANDARD.encode(input)),
            "local".into(),
            None,
        );

        assert!(plain.format_ready_for_compilation);
        assert!(encoded.format_ready_for_compilation);
    }

    #[test]
    fn parses_standard_and_unpadded_base64_subscriptions() {
        let input = "vless://id@example.com:443#One\ntrojan://secret@example.net:443#Two";
        for encoded in [
            general_purpose::STANDARD.encode(input),
            general_purpose::STANDARD_NO_PAD.encode(input),
        ] {
            let parsed = inspect_node_text(&encoded);
            assert_eq!(parsed.format, ContentFormat::Base64UriList);
            assert_eq!(parsed.nodes.len(), 2);
            assert!(!parsed.format_ready_for_compilation());
        }
    }

    #[test]
    fn parses_mihomo_yaml_without_exposing_names() {
        let input = r#"
proxies:
  - name: Alpha
    type: vless
    server: example.com
    port: 443
    uuid: 00000000-0000-0000-0000-000000000001
  - name: Renamed Alpha
    type: vless
    server: example.com
    port: 443
    uuid: 00000000-0000-0000-0000-000000000001
  - name: Beta
    type: hysteria2
    server: example.net
    port: 443
    password: fixture-only
"#;
        let summary = summarize(inspect_node_text(input), "local".into(), None);

        assert_eq!(summary.source_format, "mihomo-yaml");
        assert_eq!(summary.node_count, 2);
        assert_eq!(summary.duplicate_count, 1);
        assert!(summary.format_ready_for_compilation);
    }

    #[test]
    fn yaml_key_order_and_display_name_do_not_change_fingerprint() {
        let input = r#"
proxies:
  - name: First
    type: vless
    server: example.invalid
    port: 443
    uuid: 00000000-0000-0000-0000-000000000001
  - uuid: 00000000-0000-0000-0000-000000000001
    port: 443
    server: example.invalid
    type: vless
    name: Reordered
"#;
        let summary = summarize(inspect_node_text(input), "local".into(), None);
        assert_eq!(summary.node_count, 1);
        assert_eq!(summary.duplicate_count, 1);
    }

    #[test]
    fn detects_provider_yaml_as_resolver_required() {
        let parsed = inspect_node_text(
            "proxy-providers:\n  provider1:\n    type: http\n    url: https://example.invalid/sub",
        );
        assert_eq!(parsed.format, ContentFormat::MihomoProviderYaml);
        assert!(parsed.requires_mihomo_resolver);
    }

    #[test]
    fn rejects_html_and_binary_as_unknown() {
        assert_eq!(
            inspect_node_text("<!doctype html><html></html>").format,
            ContentFormat::Unknown
        );
        assert_eq!(inspect_bytes(&[0xff, 0xfe]).format, ContentFormat::Unknown);
    }

    #[test]
    fn sorts_query_parameters_before_fingerprinting() {
        let first = parse_proxy_uri("vless://id@example.com:443?security=tls&type=ws#One")
            .expect("first URI");
        let second = parse_proxy_uri("vless://id@example.com:443?type=ws&security=tls#Two")
            .expect("second URI");
        assert_eq!(first.fingerprint, second.fingerprint);
    }
}
