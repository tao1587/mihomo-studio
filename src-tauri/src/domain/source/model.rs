use std::{fmt, net::IpAddr};

use serde::{Deserialize, Serialize};
use url::{Host, Url};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionInspectionRequest {
    pub url: String,
    #[serde(default = "default_fetch_route")]
    pub fetch_route: String,
}

fn default_fetch_route() -> String {
    "system".to_string()
}

#[derive(Deserialize)]
pub struct NodeTextInspectionRequest {
    pub content: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInspectionSummary {
    pub safe_label: String,
    pub source_format: String,
    pub node_count: usize,
    pub duplicate_count: usize,
    pub protocols: Vec<ProtocolCount>,
    pub user_agent: Option<String>,
    pub warnings: Vec<String>,
    pub requires_mihomo_resolver: bool,
    pub format_ready_for_compilation: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolCount {
    pub protocol: String,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ContentFormat {
    MihomoYaml,
    MihomoProviderYaml,
    UriList,
    Base64UriList,
    Base64MihomoYaml,
    Unknown,
}

impl ContentFormat {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::MihomoYaml => "mihomo-yaml",
            Self::MihomoProviderYaml => "mihomo-provider-yaml",
            Self::UriList => "uri-list",
            Self::Base64UriList => "base64-uri-list",
            Self::Base64MihomoYaml => "base64-mihomo-yaml",
            Self::Unknown => "unknown",
        }
    }

    pub(crate) fn is_recognized(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ParsedNode {
    pub protocol: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ParsedContent {
    pub format: ContentFormat,
    pub nodes: Vec<ParsedNode>,
    pub warnings: Vec<String>,
    pub requires_mihomo_resolver: bool,
}

impl ParsedContent {
    pub(crate) fn format_ready_for_compilation(&self) -> bool {
        if self.requires_mihomo_resolver || self.nodes.is_empty() {
            return false;
        }
        match self.format {
            ContentFormat::MihomoYaml | ContentFormat::Base64MihomoYaml => true,
            ContentFormat::UriList | ContentFormat::Base64UriList => {
                self.nodes.iter().all(|node| node.protocol == "vless")
            }
            ContentFormat::MihomoProviderYaml | ContentFormat::Unknown => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchRoute {
    System,
    Direct,
}

impl TryFrom<&str> for FetchRoute {
    type Error = SourceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.trim() {
            "system" => Ok(Self::System),
            "direct" => Ok(Self::Direct),
            _ => Err(SourceError::InvalidFetchRoute),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SubscriptionUrl(Url);

impl SubscriptionUrl {
    pub fn parse(raw: &str) -> Result<Self, SourceError> {
        let mut url = Url::parse(raw.trim()).map_err(|_| SourceError::InvalidUrl)?;
        if !is_http_url(&url) || url.host().is_none() {
            return Err(SourceError::UnsupportedScheme);
        }
        if has_credentials(&url) {
            return Err(SourceError::EmbeddedCredentials);
        }
        if url.scheme() == "http" && !is_loopback_url(&url) {
            return Err(SourceError::RemoteHttpBlocked);
        }
        url.set_fragment(None);
        Ok(Self(url))
    }

    pub fn as_url(&self) -> &Url {
        &self.0
    }

    pub fn safe_label(&self) -> String {
        let host = match self.0.host() {
            Some(Host::Domain(value)) => value.to_string(),
            Some(Host::Ipv4(value)) => IpAddr::V4(value).to_string(),
            Some(Host::Ipv6(value)) => format!("[{}]", IpAddr::V6(value)),
            None => "订阅".to_string(),
        };
        match self.0.port() {
            Some(port) => format!("{host}:{port}/••••••"),
            None => format!("{host}/••••••"),
        }
    }
}

pub(crate) fn is_http_url(url: &Url) -> bool {
    matches!(url.scheme(), "http" | "https")
}

pub(crate) fn has_credentials(url: &Url) -> bool {
    !url.username().is_empty() || url.password().is_some()
}

pub(crate) fn is_loopback_url(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(host)) => host.eq_ignore_ascii_case("localhost"),
        Some(Host::Ipv4(address)) => address.is_loopback(),
        Some(Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceError {
    InvalidUrl,
    UnsupportedScheme,
    EmbeddedCredentials,
    RemoteHttpBlocked,
    InvalidFetchRoute,
    ClientInitialization,
    ResponseTooLarge,
    RequestTimeout,
    ConnectionFailed,
    RedirectRejected,
    ResponseReadFailed,
    RequestFailed,
    ProbeFailed(Vec<String>),
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUrl => formatter.write_str("请输入有效的订阅 URL。"),
            Self::UnsupportedScheme => formatter.write_str("订阅地址只支持 HTTP(S) URL。"),
            Self::EmbeddedCredentials => {
                formatter.write_str("订阅地址不得在 URL authority 中携带用户名或密码。")
            }
            Self::RemoteHttpBlocked => {
                formatter.write_str("远程订阅必须使用 HTTPS；HTTP 仅允许本机回环地址。")
            }
            Self::InvalidFetchRoute => formatter.write_str("抓取路径只支持 system 或 direct。"),
            Self::ClientInitialization => formatter.write_str("无法初始化订阅抓取器。"),
            Self::ResponseTooLarge => formatter.write_str("响应超过 8 MiB 上限"),
            Self::RequestTimeout => formatter.write_str("请求超时"),
            Self::ConnectionFailed => formatter.write_str("连接失败"),
            Self::RedirectRejected => formatter.write_str("重定向被拒绝"),
            Self::ResponseReadFailed => formatter.write_str("响应读取失败"),
            Self::RequestFailed => formatter.write_str("请求失败"),
            Self::ProbeFailed(attempts) => {
                write!(formatter, "订阅探测失败（{}）。", attempts.join("；"))
            }
        }
    }
}

impl std::error::Error for SourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_https_and_loopback_http_only() {
        assert!(SubscriptionUrl::parse("https://example.invalid/sub").is_ok());
        assert!(SubscriptionUrl::parse("http://127.0.0.1:9090/sub").is_ok());
        assert!(SubscriptionUrl::parse("http://[::1]:9090/sub").is_ok());
        assert!(SubscriptionUrl::parse("http://example.invalid/sub").is_err());
    }

    #[test]
    fn rejects_embedded_credentials_and_non_http_schemes() {
        assert!(SubscriptionUrl::parse("https://user:pass@example.invalid/sub").is_err());
        assert!(SubscriptionUrl::parse("file:///tmp/sub").is_err());
    }

    #[test]
    fn safe_label_does_not_expose_path_query_or_fragment() {
        let url = SubscriptionUrl::parse(
            "https://example.invalid/private/path?access_key=fixture-secret#fragment",
        )
        .expect("valid fixture URL");
        let label = url.safe_label();
        assert_eq!(label, "example.invalid/••••••");
        assert!(!label.contains("private"));
        assert!(!label.contains("access_key"));
        assert!(!label.contains("fixture-secret"));
    }

    #[test]
    fn errors_never_include_request_urls() {
        let result = SubscriptionUrl::parse("not a url").expect_err("invalid URL must fail");
        assert_eq!(result.to_string(), "请输入有效的订阅 URL。");
        assert!(!result.to_string().contains("not a url"));
    }
}
