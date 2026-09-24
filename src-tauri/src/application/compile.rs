use crate::{
    application::{
        ports::{CatalogRepository, SubscriptionGatewayFactory},
        source::fetch_subscription_material,
    },
    domain::compile::{
        compile_strict_profile as compile_domain_profile, CompileProfileRequest,
        CompileProfileResult, ResolvedSourceMaterial, StrictCompileError,
    },
};

pub async fn compile_strict_profile(
    catalog_repository: &dyn CatalogRepository,
    gateway_factory: &dyn SubscriptionGatewayFactory,
    request: CompileProfileRequest,
) -> Result<CompileProfileResult, StrictCompileError> {
    if request.sources.is_empty() {
        return Err(StrictCompileError::NoSources);
    }

    let mut resolved_sources = Vec::with_capacity(request.sources.len());
    for source in request.sources {
        let content = match source.kind.as_str() {
            "nodes" => source.value.into_bytes(),
            "subscription" => {
                fetch_subscription_material(gateway_factory, &source.value, &source.fetch_route)
                    .await
                    .map_err(|_| StrictCompileError::SourceUnavailable)?
            }
            _ => return Err(StrictCompileError::InvalidSourceKind),
        };
        resolved_sources.push(ResolvedSourceMaterial { content });
    }

    let catalog = catalog_repository
        .load()
        .map_err(|_| StrictCompileError::Profile)?;
    compile_domain_profile(
        &catalog,
        request.mode,
        request.selected_rule_source_ids,
        request.github_mirror,
        resolved_sources,
        request.bootstrap_mappings,
    )
}

#[cfg(test)]
mod tests {
    use crate::{
        application::ports::{CatalogRepository, SubscriptionGateway, SubscriptionGatewayFactory},
        domain::{
            catalog::{CatalogError, RuleCatalog},
            source::{FetchRoute, SourceError},
        },
    };

    use super::*;

    struct FixtureCatalog;

    impl CatalogRepository for FixtureCatalog {
        fn load(&self) -> Result<RuleCatalog, CatalogError> {
            serde_json::from_str(include_str!("../../../src/data/rule-catalog.json"))
                .map_err(|_| CatalogError::InvalidData("fixture catalog".into()))
        }
    }

    struct UnusedGatewayFactory;

    impl SubscriptionGatewayFactory for UnusedGatewayFactory {
        fn create(&self, _route: FetchRoute) -> Result<Box<dyn SubscriptionGateway>, SourceError> {
            Err(SourceError::ClientInitialization)
        }
    }

    fn request_with_server(server: &str) -> CompileProfileRequest {
        serde_json::from_value(serde_json::json!({
            "mode": "simple",
            "selectedRuleSourceIds": ["acl4ssr"],
            "githubMirror": null,
            "sources": [{
                "kind": "nodes",
                "fetchRoute": "system",
                "value": format!(
                    "proxies:\n  - name: SECRET_NODE_NAME\n    type: ss\n    server: {server}\n    port: 443\n    cipher: aes-128-gcm\n    password: SECRET_PASSWORD\n"
                )
            }]
        }))
        .expect("compile request contract parses")
    }

    fn request_with_vless_uri() -> CompileProfileRequest {
        serde_json::from_value(serde_json::json!({
            "mode": "simple",
            "selectedRuleSourceIds": ["acl4ssr"],
            "githubMirror": null,
            "sources": [{
                "kind": "nodes",
                "fetchRoute": "system",
                "value": concat!(
                    "vless://00000000-0000-4000-8000-000000000001@192.0.2.10:443?",
                    "mode=multi&encryption=none&security=reality&sni=fixture.example.invalid&",
                    "servername=fixture.example.invalid&fp=chrome&",
                    "type=tcp&flow=xtls-rprx-vision&pbk=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA&",
                    "sid=0123456789abcdef&spx=%2F#SECRET_NODE_NAME"
                )
            }]
        }))
        .expect("VLESS compile request contract parses")
    }

    fn request_with_mapped_domain() -> CompileProfileRequest {
        serde_json::from_value(serde_json::json!({
            "mode": "simple",
            "selectedRuleSourceIds": ["acl4ssr"],
            "githubMirror": null,
            "sources": [{
                "kind": "nodes",
                "fetchRoute": "system",
                "value": "proxies:\n  - name: SECRET_NODE_NAME\n    type: ss\n    server: secret-node.example.invalid\n    port: 443\n    cipher: aes-128-gcm\n    password: SECRET_PASSWORD\n"
            }],
            "bootstrapMappings": [{
                "source": 1,
                "node": 1,
                "ipv4": "192.0.2.10"
            }]
        }))
        .expect("mapped domain compile request contract parses")
    }

    #[test]
    fn ipc_contract_compiles_local_yaml_without_echoing_it_to_the_report() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("fixture runtime");
        let result = runtime
            .block_on(compile_strict_profile(
                &FixtureCatalog,
                &UnusedGatewayFactory,
                request_with_server("192.0.2.10"),
            ))
            .expect("fixture profile compiles");
        let report = serde_json::to_string(&result.report).expect("report serializes");

        assert!(!report.contains("SECRET_NODE_NAME"));
        assert!(!report.contains("SECRET_PASSWORD"));
        assert!(result.yaml.contains("SECRET_PASSWORD"));
    }

    #[test]
    fn bootstrap_error_does_not_echo_node_secrets() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("fixture runtime");
        let error = runtime
            .block_on(compile_strict_profile(
                &FixtureCatalog,
                &UnusedGatewayFactory,
                request_with_server("secret-node.example.invalid"),
            ))
            .err()
            .expect("domain bootstrap must fail")
            .to_string();

        assert!(!error.contains("secret-node.example.invalid"));
        assert!(!error.contains("SECRET_NODE_NAME"));
        assert!(!error.contains("SECRET_PASSWORD"));
        assert!(error.contains("1:1=IPv4"));
    }

    #[test]
    fn ipc_contract_compiles_vless_without_echoing_credentials_to_the_report() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("fixture runtime");
        let result = runtime
            .block_on(compile_strict_profile(
                &FixtureCatalog,
                &UnusedGatewayFactory,
                request_with_vless_uri(),
            ))
            .expect("VLESS profile compiles");
        let report = serde_json::to_string(&result.report).expect("report serializes");

        assert!(!report.contains("SECRET_NODE_NAME"));
        assert!(!report.contains("00000000-0000-4000-8000-000000000001"));
        assert!(!report.contains("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
        assert!(result
            .report
            .warnings
            .iter()
            .any(|warning| warning.contains("pqv/spx")));
        assert!(result.yaml.contains("type: vless"));
        assert!(result.yaml.contains("reality-opts"));
    }

    #[test]
    fn mapped_domain_bootstrap_stays_out_of_the_generation_report() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("fixture runtime");
        let result = runtime
            .block_on(compile_strict_profile(
                &FixtureCatalog,
                &UnusedGatewayFactory,
                request_with_mapped_domain(),
            ))
            .expect("mapped domain fixture compiles");
        let report = serde_json::to_string(&result.report).expect("report serializes");

        assert!(!report.contains("secret-node.example.invalid"));
        assert!(!report.contains("192.0.2.10"));
        assert!(!report.contains("SECRET_NODE_NAME"));
        assert!(!report.contains("SECRET_PASSWORD"));
        assert!(result.yaml.contains("hosts:"));
        assert!(result
            .yaml
            .contains("secret-node.example.invalid: 192.0.2.10"));
    }

    #[test]
    fn invalid_catalog_selection_does_not_echo_user_controlled_source_id() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("fixture runtime");
        let mut request = request_with_server("192.0.2.10");
        request.selected_rule_source_ids = vec!["SECRET_RULE_SOURCE_ID".into()];

        let error = runtime
            .block_on(compile_strict_profile(
                &FixtureCatalog,
                &UnusedGatewayFactory,
                request,
            ))
            .err()
            .expect("unknown catalog source must fail")
            .to_string();

        assert_eq!(error, "编译请求未通过规则草案校验。");
        assert!(!error.contains("SECRET_RULE_SOURCE_ID"));
    }
}
