use crate::{
    application::ports::{SubscriptionFetchResult, SubscriptionGatewayFactory},
    domain::{
        compile::{
            convert_node_text as convert_node_material, preflight_uri_material,
            ConvertNodeTextRequest, ConvertNodeTextResult, StrictCompileError,
        },
        source::{
            inspect_bytes, inspect_node_text as parse_node_text, summarize, FetchRoute,
            NodeTextInspectionRequest, SourceError, SourceInspectionSummary,
            SubscriptionInspectionRequest, SubscriptionUrl,
        },
    },
};

const USER_AGENTS: [(&str, &str); 4] = [
    ("Mihomo", "mihomo/1.19"),
    ("Clash.Meta", "Clash.Meta"),
    ("Clash Verge", "Clash-Verge"),
    (
        "Browser",
        "Mozilla/5.0 (compatible; Mihomo-Studio/0.1; +https://example.invalid)",
    ),
];

pub async fn inspect_subscription(
    gateway_factory: &dyn SubscriptionGatewayFactory,
    request: SubscriptionInspectionRequest,
) -> Result<SourceInspectionSummary, SourceError> {
    let source = SubscriptionUrl::parse(&request.url)?;
    let route = FetchRoute::try_from(request.fetch_route.as_str())?;
    let gateway = gateway_factory.create(route)?;
    let safe_label = source.safe_label();
    let mut attempts = Vec::new();
    let mut last_unknown = None;

    for (label, user_agent) in USER_AGENTS {
        match gateway.fetch(&source, user_agent).await {
            Ok(SubscriptionFetchResult::HttpStatus(status)) => {
                attempts.push(format!("{label}: HTTP {status}"));
            }
            Ok(SubscriptionFetchResult::Body(bytes)) => {
                let parsed = inspect_bytes(&bytes);
                let recognized = parsed.format.is_recognized();
                let mut summary = summarize(parsed, safe_label.clone(), Some(label.to_string()));
                apply_compile_preflight(&mut summary, &bytes);
                if recognized {
                    if !attempts.is_empty() {
                        summary.warnings.push(format!(
                            "前 {} 个 User-Agent 未返回可用订阅，已自动切换。",
                            attempts.len()
                        ));
                    }
                    return Ok(summary);
                }
                attempts.push(format!("{label}: 格式未识别"));
                last_unknown = Some(summary);
            }
            Err(category) => attempts.push(format!("{label}: {category}")),
        }
    }

    if let Some(mut summary) = last_unknown {
        summary
            .warnings
            .push("所有兼容 User-Agent 均未直接识别；下一阶段交给 Mihomo resolver。".into());
        return Ok(summary);
    }

    Err(SourceError::ProbeFailed(attempts))
}

pub(crate) async fn fetch_subscription_material(
    gateway_factory: &dyn SubscriptionGatewayFactory,
    raw_url: &str,
    fetch_route: &str,
) -> Result<Vec<u8>, SourceError> {
    let source = SubscriptionUrl::parse(raw_url)?;
    let route = FetchRoute::try_from(fetch_route)?;
    let gateway = gateway_factory.create(route)?;
    let mut attempts = Vec::new();

    for (label, user_agent) in USER_AGENTS {
        match gateway.fetch(&source, user_agent).await {
            Ok(SubscriptionFetchResult::HttpStatus(status)) => {
                attempts.push(format!("{label}: HTTP {status}"));
            }
            Ok(SubscriptionFetchResult::Body(bytes)) => {
                if inspect_bytes(&bytes).format.is_recognized() {
                    return Ok(bytes);
                }
                attempts.push(format!("{label}: 格式未识别"));
            }
            Err(category) => attempts.push(format!("{label}: {category}")),
        }
    }

    Err(SourceError::ProbeFailed(attempts))
}

pub fn inspect_node_text(
    request: NodeTextInspectionRequest,
) -> Result<SourceInspectionSummary, SourceError> {
    let parsed = parse_node_text(&request.content);
    let mut summary = summarize(parsed, "本地粘贴".into(), None);
    apply_compile_preflight(&mut summary, request.content.as_bytes());
    Ok(summary)
}

pub fn convert_node_text(
    request: ConvertNodeTextRequest,
) -> Result<ConvertNodeTextResult, StrictCompileError> {
    convert_node_material(request.content.as_bytes())
}

fn apply_compile_preflight(summary: &mut SourceInspectionSummary, content: &[u8]) {
    if !summary.format_ready_for_compilation {
        return;
    }
    if let Some(Err(error)) = preflight_uri_material(content) {
        summary.format_ready_for_compilation = false;
        summary
            .warnings
            .push(format!("严格编译参数预检未通过：{error}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_text_use_case_returns_redacted_summary_only() {
        let request = NodeTextInspectionRequest {
            content: "vless://fixture-id@example.invalid:443#Fixture Name".into(),
        };
        let summary = inspect_node_text(request).expect("fixture should parse");
        let serialized = serde_json::to_string(&summary).expect("summary serializes");

        assert_eq!(summary.safe_label, "本地粘贴");
        assert_eq!(summary.node_count, 1);
        assert!(!serialized.contains("fixture-id"));
        assert!(!serialized.contains("Fixture Name"));
    }

    #[test]
    fn direct_text_preflight_reports_all_node_categories_without_secrets() {
        let request = NodeTextInspectionRequest {
            content: concat!(
                "vless://PRIVATE_UUID_ONE@192.0.2.10:443?security=tls&",
                "PRIVATE_QUERY_KEY=PRIVATE_QUERY_VALUE#PRIVATE_NODE_ONE\n",
                "vless://PRIVATE_UUID_TWO@192.0.2.11:443?security=tls&",
                "type=ws&mode=PRIVATE_MODE#PRIVATE_NODE_TWO"
            )
            .into(),
        };

        let summary = inspect_node_text(request).expect("recognized source returns a summary");
        let serialized = serde_json::to_string(&summary).expect("summary serializes");

        assert_eq!(summary.node_count, 2);
        assert!(!summary.format_ready_for_compilation);
        assert!(summary.warnings.iter().any(|warning| {
            warning.contains("发现 2 个节点问题")
                && warning.contains("未知分享参数")
                && warning.contains("传输类型")
        }));
        for secret in [
            "PRIVATE_UUID_ONE",
            "PRIVATE_UUID_TWO",
            "PRIVATE_QUERY_KEY",
            "PRIVATE_QUERY_VALUE",
            "PRIVATE_MODE",
            "PRIVATE_NODE_ONE",
            "PRIVATE_NODE_TWO",
        ] {
            assert!(!serialized.contains(secret));
        }
    }

    #[test]
    fn base64_vless_uses_the_same_parameter_preflight() {
        use base64::{engine::general_purpose, Engine as _};

        let content = concat!(
            "vless://PRIVATE_UUID@192.0.2.10:443?security=tls&",
            "PRIVATE_QUERY_KEY=PRIVATE_QUERY_VALUE#PRIVATE_NODE"
        );
        let summary = inspect_node_text(NodeTextInspectionRequest {
            content: general_purpose::STANDARD.encode(content),
        })
        .expect("Base64 source returns a summary");

        assert_eq!(summary.source_format, "base64-uri-list");
        assert!(!summary.format_ready_for_compilation);
        assert!(summary
            .warnings
            .iter()
            .any(|warning| warning.contains("未知分享参数")));
        let serialized = serde_json::to_string(&summary).expect("summary serializes");
        assert!(!serialized.contains("PRIVATE_QUERY_KEY"));
        assert!(!serialized.contains("PRIVATE_QUERY_VALUE"));
    }

    #[test]
    fn provider_shaped_reality_uri_passes_parameter_preflight() {
        let summary = inspect_node_text(NodeTextInspectionRequest {
            content: concat!(
                "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
                "mode=multi&encryption=none&security=reality&type=tcp&",
                "sni=fixture.example.invalid&servername=fixture.example.invalid&",
                "fp=chrome&flow=xtls-rprx-vision&",
                "pbk=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&",
                "pqv=PLACEHOLDER_MLDSA_VERIFY&sid=0123456789abcdef&",
                "spx=%2Ffixture-spider#Fixture"
            )
            .into(),
        })
        .expect("provider-shaped source returns a summary");

        assert_eq!(summary.node_count, 1);
        assert!(summary.format_ready_for_compilation);
        assert!(summary
            .warnings
            .iter()
            .all(|warning| !warning.contains("预检未通过")));
    }

    #[test]
    fn conversion_use_case_returns_a_sensitive_proxy_fragment() {
        let result = convert_node_text(ConvertNodeTextRequest {
            content: concat!(
                "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
                "encryption=none&security=tls&type=tcp#Fixture"
            )
            .into(),
        })
        .expect("fixture should convert");

        assert_eq!(result.node_count, 1);
        assert!(result.yaml.contains("proxies:"));
        assert!(result.yaml.contains("type: vless"));
    }

    #[test]
    fn conversion_use_case_keeps_wireguard_keys_only_in_sensitive_yaml() {
        use base64::{engine::general_purpose, Engine as _};

        let private_key = general_purpose::STANDARD.encode([b'P'; 32]);
        let public_key = general_purpose::STANDARD.encode([b'K'; 32]);
        let result = convert_node_text(ConvertNodeTextRequest {
            content: format!(
                "[Interface]\nPrivateKey = {private_key}\nAddress = 192.0.2.2/32\n\
                 DNS = 192.0.2.53\n\n[Peer]\nPublicKey = {public_key}\n\
                 Endpoint = wg-node.example.invalid:51820\nAllowedIPs = 0.0.0.0/0\n\
                 PersistentKeepalive = 25\n"
            ),
        })
        .expect("fixture WireGuard should convert");
        let warnings = serde_json::to_string(&result.warnings).expect("warnings serialize");

        assert_eq!(result.node_count, 1);
        assert!(result.yaml.contains("type: wireguard"));
        assert!(result.yaml.contains(&private_key));
        assert!(result.yaml.contains(&public_key));
        assert!(!warnings.contains(&private_key));
        assert!(!warnings.contains(&public_key));
        assert!(!warnings.contains("wg-node.example.invalid"));
    }
}
