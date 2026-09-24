use std::{collections::HashSet, fmt};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleCatalog {
    pub schema_version: u32,
    pub researched_at: String,
    pub sources: Vec<RuleSource>,
    pub rule_sets: Vec<CatalogRuleSet>,
}

impl RuleCatalog {
    pub fn validate(&self) -> Result<(), CatalogError> {
        let source_ids = self
            .sources
            .iter()
            .map(|source| source.id.as_str())
            .collect::<HashSet<_>>();
        if source_ids.len() != self.sources.len() {
            return Err(CatalogError::InvalidData(
                "rule source ids must be unique".into(),
            ));
        }

        let rule_set_ids = self
            .rule_sets
            .iter()
            .map(|rule_set| rule_set.id.as_str())
            .collect::<HashSet<_>>();
        if rule_set_ids.len() != self.rule_sets.len() {
            return Err(CatalogError::InvalidData(
                "rule set ids must be unique".into(),
            ));
        }

        if self
            .sources
            .iter()
            .any(|source| !source.repository.starts_with("https://github.com/"))
        {
            return Err(CatalogError::InvalidData(
                "rule source repositories must use canonical GitHub HTTPS URLs".into(),
            ));
        }

        if self.rule_sets.iter().any(|rule_set| {
            rule_set
                .repository_id
                .as_ref()
                .is_some_and(|repository_id| !source_ids.contains(repository_id.as_str()))
        }) {
            return Err(CatalogError::InvalidData(
                "rule set references an unknown repository".into(),
            ));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSource {
    pub id: String,
    pub name: String,
    pub summary: String,
    pub repository: String,
    pub license: String,
    pub formats: Vec<String>,
    pub categories: Vec<String>,
    pub available_modes: Vec<String>,
    pub default_enabled_modes: Vec<String>,
    pub priority: u32,
    pub policy_targets: Vec<String>,
    pub mirror_eligible: bool,
    pub maintenance: String,
    pub selection_kind: String,
    pub content_audit: String,
    pub exclusive_group: Option<String>,
    pub upstreams: Vec<String>,
    pub popularity: RuleSourcePopularity,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSourcePopularity {
    pub stars: u32,
    pub forks: u32,
    pub snapshot_date: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogRuleSet {
    pub id: String,
    pub title: String,
    pub repository_id: Option<String>,
    pub family: String,
    pub canonical_url: Option<String>,
    pub r#ref: Option<String>,
    pub path: Option<String>,
    pub behavior: String,
    pub format: String,
    pub target_policy: String,
    pub order: u32,
    pub available_modes: Vec<String>,
    pub default_enabled_modes: Vec<String>,
    pub exclusive_group: Option<String>,
    pub mirror_kind: String,
    pub content_audit: String,
    pub attribution_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CatalogError {
    InvalidData(String),
}

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidData(detail) => write!(formatter, "rule catalog is invalid: {detail}"),
        }
    }
}

impl std::error::Error for CatalogError {}
