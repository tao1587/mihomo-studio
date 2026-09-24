use std::net::IpAddr;

use yaml_rust2::{Yaml, YamlLoader};

use super::model::StrictCompileError;
use super::serializer::serialize_proxy_list;

const SINGLE_NODE_TEMPLATE: &str = include_str!("templates/clash-single-node.yml");
const NODE_PLACEHOLDER: &str = "__MIHOMO_STUDIO_NODE__";

pub(crate) fn render_single_ip_template(
    proxy: &Yaml,
) -> Result<Option<(String, String)>, StrictCompileError> {
    let Some(ip) = proxy_ip(proxy) else {
        return Ok(None);
    };
    let ip_name = ip.to_string();
    let mut replacement = proxy.clone();
    let Yaml::Hash(fields) = &mut replacement else {
        return Err(StrictCompileError::Serialization);
    };
    fields.insert(Yaml::String("name".into()), Yaml::String(ip_name.clone()));
    if SINGLE_NODE_TEMPLATE.matches(NODE_PLACEHOLDER).count() != 5
        || SINGLE_NODE_TEMPLATE.matches("proxies: []").count() != 1
    {
        return Err(StrictCompileError::Serialization);
    }
    let fragment = serialize_proxy_list(vec![replacement])?;
    let fragment = fragment
        .strip_prefix("---\n")
        .ok_or(StrictCompileError::Serialization)?;
    let template = SINGLE_NODE_TEMPLATE.replace(
        &format!("\"{NODE_PLACEHOLDER}\""),
        &format!("\"{ip_name}\""),
    );
    let output = template.replacen("proxies: []", fragment.trim_end(), 1);
    let documents =
        YamlLoader::load_from_str(&output).map_err(|_| StrictCompileError::RoundTrip)?;
    if output.contains(NODE_PLACEHOLDER)
        || documents.len() != 1
        || documents[0]["proxies"].as_vec().map(Vec::len) != Some(1)
    {
        return Err(StrictCompileError::RoundTrip);
    }
    let filename = format!("{}.yaml", ip_name.replace(':', "-"));
    Ok(Some((output, filename)))
}

fn proxy_ip(proxy: &Yaml) -> Option<IpAddr> {
    let server = proxy["server"]
        .as_str()
        .or_else(|| proxy["peers"].as_vec()?.first()?["server"].as_str());
    server.and_then(|value| value.parse().ok())
}
