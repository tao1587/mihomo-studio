use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use yaml_rust2::Yaml;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertNodeTextRequest {
    pub content: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertNodeTextResult {
    pub yaml: String,
    pub template_yaml: Option<String>,
    pub template_file_name: Option<String>,
    pub node_count: usize,
    pub duplicate_node_count: usize,
    pub compatibility_normalization_count: usize,
    pub warnings: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileProfileRequest {
    pub mode: String,
    pub selected_rule_source_ids: Vec<String>,
    pub github_mirror: Option<String>,
    pub sources: Vec<CompileSourceInput>,
    #[serde(default)]
    pub bootstrap_mappings: Vec<BootstrapMappingInput>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileSourceInput {
    pub kind: String,
    pub value: String,
    #[serde(default = "default_fetch_route")]
    pub fetch_route: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BootstrapMappingInput {
    pub source: usize,
    pub node: usize,
    pub ipv4: String,
}

fn default_fetch_route() -> String {
    "system".to_string()
}

#[derive(Clone)]
pub struct ResolvedSourceMaterial {
    pub content: Vec<u8>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileProfileResult {
    pub yaml: String,
    pub report: CompileReport,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileReport {
    pub status: String,
    pub node_count: usize,
    pub duplicate_node_count: usize,
    pub rule_provider_count: usize,
    pub rule_count: usize,
    pub strict_privacy: bool,
    pub content_sha256: String,
    pub warnings: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct NodeIr {
    pub name: String,
    pub yaml: Yaml,
}

pub(crate) struct CompiledNodes {
    pub nodes: Vec<NodeIr>,
    pub duplicate_count: usize,
    pub bootstrap_hosts: BTreeMap<String, String>,
    pub target_compatibility_normalization_count: usize,
}

#[derive(Clone)]
pub(crate) struct RuleProviderIr {
    pub name: String,
    pub url: String,
    pub path: String,
    pub behavior: String,
    pub format: String,
    pub proxy: String,
}

#[derive(Clone)]
pub(crate) struct RuleIr {
    pub expression: String,
    pub target: String,
}

pub(crate) struct CompiledRules {
    pub providers: Vec<RuleProviderIr>,
    pub rules: Vec<RuleIr>,
}

#[derive(Clone)]
pub(crate) struct ProxyGroupIr {
    pub name: String,
    pub group_type: String,
    pub proxies: Vec<String>,
    pub url: Option<String>,
    pub interval: Option<i64>,
    pub lazy: Option<bool>,
}

pub(crate) struct MihomoConfigIr {
    pub nodes: Vec<NodeIr>,
    pub bootstrap_hosts: BTreeMap<String, String>,
    pub groups: Vec<ProxyGroupIr>,
    pub providers: Vec<RuleProviderIr>,
    pub rules: Vec<RuleIr>,
}

pub(crate) const MAX_REPORTED_NODE_ISSUES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeIssueKind {
    UnsupportedProtocol,
    UnknownShareParameter,
    DuplicateShareParameter,
    ConflictingAlias,
    InvalidOptionValue,
    IncompatibleSecurityOption,
    IncompatibleTransportOption,
    InvalidEntry,
    UnprotectedBootstrap,
    InvalidBootstrapMapping,
    Ipv6Endpoint,
    UnsafeOverride,
}

impl NodeIssueKind {
    fn safe_description(self) -> &'static str {
        match self {
            Self::UnsupportedProtocol => "协议尚未接通严格编译",
            Self::UnknownShareParameter => "包含未知分享参数",
            Self::DuplicateShareParameter => "包含重复分享参数",
            Self::ConflictingAlias => "连接主机别名值冲突",
            Self::InvalidOptionValue => "分享参数值格式无效",
            Self::IncompatibleSecurityOption => "包含与 TLS/REALITY 不兼容的分享参数",
            Self::IncompatibleTransportOption => "包含与传输类型不兼容的分享参数",
            Self::InvalidEntry => "缺少必要字段",
            Self::UnprotectedBootstrap => "域名 server 缺少受保护 bootstrap 映射",
            Self::InvalidBootstrapMapping => "bootstrap 映射无效或未匹配域名节点",
            Self::Ipv6Endpoint => "使用尚未完成严格隐私闭环的 IPv6 server",
            Self::UnsafeOverride => "包含可能绕过严格出口策略的字段",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NodeCompileIssue {
    pub source: usize,
    pub node: usize,
    pub kind: NodeIssueKind,
}

impl NodeCompileIssue {
    fn into_error(self) -> StrictCompileError {
        let Self { source, node, kind } = self;
        match kind {
            NodeIssueKind::UnsupportedProtocol => {
                StrictCompileError::UnsupportedNodeProtocol { source, node }
            }
            NodeIssueKind::UnknownShareParameter => {
                StrictCompileError::UnknownNodeOption { source, node }
            }
            NodeIssueKind::DuplicateShareParameter => {
                StrictCompileError::DuplicateNodeOption { source, node }
            }
            NodeIssueKind::ConflictingAlias => {
                StrictCompileError::ConflictingNodeAlias { source, node }
            }
            NodeIssueKind::InvalidOptionValue => {
                StrictCompileError::InvalidNodeOptionValue { source, node }
            }
            NodeIssueKind::IncompatibleSecurityOption => {
                StrictCompileError::UnsupportedNodeSecurityOption { source, node }
            }
            NodeIssueKind::IncompatibleTransportOption => {
                StrictCompileError::UnsupportedNodeTransportOption { source, node }
            }
            NodeIssueKind::InvalidEntry => StrictCompileError::InvalidNodeEntry { source, node },
            NodeIssueKind::UnprotectedBootstrap => {
                StrictCompileError::UnprotectedNodeBootstrap { source, node }
            }
            NodeIssueKind::InvalidBootstrapMapping => {
                StrictCompileError::InvalidBootstrapMapping { source, node }
            }
            NodeIssueKind::Ipv6Endpoint => StrictCompileError::Ipv6NodeEndpoint { source, node },
            NodeIssueKind::UnsafeOverride => {
                StrictCompileError::UnsafeNodeOverride { source, node }
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StrictCompileError {
    NoSources,
    InvalidSourceKind,
    SourceUnavailable,
    SourceTooLarge(usize),
    InvalidUtf8(usize),
    UnsupportedNodeSource(usize),
    UnsupportedNodeProtocol {
        source: usize,
        node: usize,
    },
    UnknownNodeOption {
        source: usize,
        node: usize,
    },
    DuplicateNodeOption {
        source: usize,
        node: usize,
    },
    ConflictingNodeAlias {
        source: usize,
        node: usize,
    },
    InvalidNodeOptionValue {
        source: usize,
        node: usize,
    },
    UnsupportedNodeSecurityOption {
        source: usize,
        node: usize,
    },
    UnsupportedNodeTransportOption {
        source: usize,
        node: usize,
    },
    InvalidNodeEntry {
        source: usize,
        node: usize,
    },
    InvalidWireGuardConfig {
        source: usize,
        node: usize,
    },
    DuplicateWireGuardField {
        source: usize,
        node: usize,
    },
    UnsupportedWireGuardField {
        source: usize,
        node: usize,
    },
    UnprotectedNodeBootstrap {
        source: usize,
        node: usize,
    },
    InvalidBootstrapMapping {
        source: usize,
        node: usize,
    },
    Ipv6NodeEndpoint {
        source: usize,
        node: usize,
    },
    UnsafeNodeOverride {
        source: usize,
        node: usize,
    },
    NodeIssues {
        issues: Vec<NodeCompileIssue>,
        total: usize,
    },
    NoNodes,
    Profile,
    NoRuleSetsForSource(String),
    InvalidRuleProvider(String),
    InvalidRuleTarget(String),
    InvalidGroupGraph,
    PrivacyInvariant(String),
    Serialization,
    RoundTrip,
}

impl fmt::Display for StrictCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSources => formatter.write_str("至少添加一个节点来源。"),
            Self::InvalidSourceKind => formatter.write_str("节点来源类型无效。"),
            Self::SourceUnavailable => formatter.write_str("订阅来源获取失败。"),
            Self::SourceTooLarge(source) => {
                write!(formatter, "第 {source} 个来源超过 8 MiB 安全上限。")
            }
            Self::InvalidUtf8(source) => {
                write!(formatter, "第 {source} 个来源不是可编译的 UTF-8 文本。")
            }
            Self::UnsupportedNodeSource(source) => write!(
                formatter,
                "第 {source} 个来源尚不能直接编译；当前支持 Mihomo YAML 和 VLESS URI。"
            ),
            Self::UnsupportedNodeProtocol { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点协议尚未接通严格编译。"
            ),
            Self::UnknownNodeOption { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点包含未知分享参数。"
            ),
            Self::DuplicateNodeOption { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点包含重复分享参数。"
            ),
            Self::ConflictingNodeAlias { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点连接主机别名值冲突。"
            ),
            Self::InvalidNodeOptionValue { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点分享参数值格式无效。"
            ),
            Self::UnsupportedNodeSecurityOption { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点包含与 TLS/REALITY 不兼容的分享参数。"
            ),
            Self::UnsupportedNodeTransportOption { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点包含与传输类型不兼容的分享参数。"
            ),
            Self::InvalidNodeEntry { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点缺少必要字段。"
            ),
            Self::InvalidWireGuardConfig { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个 WireGuard 配置结构或字段值无效。"
            ),
            Self::DuplicateWireGuardField { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个 WireGuard 配置包含重复字段。"
            ),
            Self::UnsupportedWireGuardField { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个 WireGuard 配置包含未映射字段。"
            ),
            Self::UnprotectedNodeBootstrap { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点使用域名 server；请在域名节点 bootstrap 中添加 {source}:{node}=IPv4 的已验证映射。"
            ),
            Self::InvalidBootstrapMapping { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点 bootstrap 映射无效或未匹配域名节点。"
            ),
            Self::Ipv6NodeEndpoint { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点使用 IPv6 server；当前严格隐私配置尚未启用 IPv6 闭环。"
            ),
            Self::UnsafeNodeOverride { source, node } => write!(
                formatter,
                "第 {source} 个来源的第 {node} 个节点包含可能绕过严格出口策略的字段。"
            ),
            Self::NodeIssues { issues, total } => {
                write!(formatter, "发现 {total} 个节点问题：")?;
                for (index, issue) in issues.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str("；")?;
                    }
                    write!(
                        formatter,
                        "第 {} 个来源的第 {} 个节点{}",
                        issue.source,
                        issue.node,
                        issue.kind.safe_description()
                    )?;
                }
                if *total > issues.len() {
                    write!(formatter, "；另有 {} 个同类或其他节点问题", total - issues.len())?;
                }
                formatter.write_str("。请一次修正上述节点后重新生成。")
            }
            Self::NoNodes => formatter.write_str("来源中没有可编译节点。"),
            Self::Profile => formatter.write_str("编译请求未通过规则草案校验。"),
            Self::NoRuleSetsForSource(source) => {
                write!(formatter, "规则来源 {source} 尚无可编译 rule set。")
            }
            Self::InvalidRuleProvider(provider) => {
                write!(formatter, "规则 provider {provider} 缺少安全传输信息。")
            }
            Self::InvalidRuleTarget(target) => {
                write!(formatter, "规则目标 {target} 不存在或不符合严格隐私策略。")
            }
            Self::InvalidGroupGraph => formatter.write_str("策略组引用不完整或存在循环。"),
            Self::PrivacyInvariant(detail) => {
                write!(formatter, "严格隐私静态检查失败：{detail}")
            }
            Self::Serialization => formatter.write_str("Mihomo YAML 序列化失败。"),
            Self::RoundTrip => formatter.write_str("生成 YAML 回读失败。"),
        }
    }
}

impl std::error::Error for StrictCompileError {}

impl StrictCompileError {
    pub(crate) fn unpack_node_issues(
        self,
    ) -> Result<(Vec<NodeCompileIssue>, usize), StrictCompileError> {
        let issue = match self {
            Self::NodeIssues { issues, total } => return Ok((issues, total)),
            Self::UnsupportedNodeProtocol { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::UnsupportedProtocol,
            },
            Self::UnknownNodeOption { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::UnknownShareParameter,
            },
            Self::DuplicateNodeOption { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::DuplicateShareParameter,
            },
            Self::ConflictingNodeAlias { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::ConflictingAlias,
            },
            Self::InvalidNodeOptionValue { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::InvalidOptionValue,
            },
            Self::UnsupportedNodeSecurityOption { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::IncompatibleSecurityOption,
            },
            Self::UnsupportedNodeTransportOption { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::IncompatibleTransportOption,
            },
            Self::InvalidNodeEntry { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::InvalidEntry,
            },
            Self::UnprotectedNodeBootstrap { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::UnprotectedBootstrap,
            },
            Self::InvalidBootstrapMapping { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::InvalidBootstrapMapping,
            },
            Self::Ipv6NodeEndpoint { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::Ipv6Endpoint,
            },
            Self::UnsafeNodeOverride { source, node } => NodeCompileIssue {
                source,
                node,
                kind: NodeIssueKind::UnsafeOverride,
            },
            other => return Err(other),
        };
        Ok((vec![issue], 1))
    }

    pub(crate) fn from_node_issues(
        mut issues: Vec<NodeCompileIssue>,
        total: usize,
    ) -> StrictCompileError {
        if total == 1 && issues.len() == 1 {
            return issues.remove(0).into_error();
        }
        StrictCompileError::NodeIssues { issues, total }
    }
}
