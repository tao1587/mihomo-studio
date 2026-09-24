use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildDraftRequest {
    pub mode: String,
    pub selected_rule_source_ids: Vec<String>,
    pub input_source_count: u32,
    pub format_ready_source_count: u32,
    pub resolved_node_count: u32,
    pub github_mirror: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDraft {
    pub mode: String,
    pub selected_rule_source_count: usize,
    pub input_source_count: u32,
    pub resolved_node_count: u32,
    pub groups: Vec<String>,
    pub rule_order: Vec<String>,
    pub final_rule: String,
    pub privacy: PrivacyPolicyDraft,
    pub warnings: Vec<String>,
    pub ready_for_compilation: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyPolicyDraft {
    pub level: String,
    pub dns_mode: String,
    pub dns_egress: String,
    pub dns_hijack_required: bool,
    pub tun_strict_route_required: bool,
    pub ipv6_enabled: bool,
    pub direct_egress_allowed: bool,
    pub protected_node_bootstrap_required: bool,
    pub provider_updates_via_proxy: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProfileError {
    InvalidMode,
    InvalidMirror,
    UnknownRuleSource(String),
    ReferenceOnlyRuleSource(String),
    ExclusiveGroupConflict(String),
    Catalog(String),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMode => formatter.write_str("mode must be simple or full"),
            Self::InvalidMirror => {
                formatter.write_str("GitHub mirror must be a non-empty HTTPS URL")
            }
            Self::UnknownRuleSource(source) => write!(formatter, "unknown rule source: {source}"),
            Self::ReferenceOnlyRuleSource(source) => {
                write!(formatter, "rule source is reference-only: {source}")
            }
            Self::ExclusiveGroupConflict(group) => write!(
                formatter,
                "multiple rule sources selected from exclusive group: {group}"
            ),
            Self::Catalog(detail) => formatter.write_str(detail),
        }
    }
}

impl std::error::Error for ProfileError {}
