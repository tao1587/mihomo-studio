use crate::{
    application::ports::CatalogRepository,
    domain::catalog::{CatalogError, RuleCatalog},
};

const EMBEDDED_CATALOG: &str = include_str!("../../../../src/data/rule-catalog.json");

#[derive(Clone, Copy, Debug, Default)]
pub struct EmbeddedCatalogRepository;

impl CatalogRepository for EmbeddedCatalogRepository {
    fn load(&self) -> Result<RuleCatalog, CatalogError> {
        let catalog: RuleCatalog = serde_json::from_str(EMBEDDED_CATALOG)
            .map_err(|error| CatalogError::InvalidData(error.to_string()))?;
        catalog.validate()?;
        Ok(catalog)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn embedded_catalog_is_valid_and_unique() {
        let catalog = EmbeddedCatalogRepository
            .load()
            .expect("catalog should parse");
        let source_ids = catalog
            .sources
            .iter()
            .map(|source| source.id.as_str())
            .collect::<HashSet<_>>();
        let rule_set_ids = catalog
            .rule_sets
            .iter()
            .map(|rule_set| rule_set.id.as_str())
            .collect::<HashSet<_>>();

        assert_eq!(source_ids.len(), catalog.sources.len());
        assert_eq!(rule_set_ids.len(), catalog.rule_sets.len());
    }

    #[test]
    fn simple_defaults_have_one_final_match_and_one_primary_ad_source() {
        let catalog = EmbeddedCatalogRepository
            .load()
            .expect("catalog should parse");
        let simple_defaults = catalog
            .rule_sets
            .iter()
            .filter(|rule_set| {
                rule_set
                    .default_enabled_modes
                    .iter()
                    .any(|mode| mode == "simple")
            })
            .collect::<Vec<_>>();
        let final_matches = simple_defaults
            .iter()
            .filter(|rule_set| rule_set.id == "builtin.match" && rule_set.order == 1000)
            .count();
        let primary_ads = simple_defaults
            .iter()
            .filter(|rule_set| rule_set.exclusive_group.as_deref() == Some("ads.primary"))
            .count();

        assert_eq!(final_matches, 1);
        assert_eq!(primary_ads, 1);
    }

    #[test]
    fn opaque_rules_are_not_marked_as_text_auditable() {
        let catalog = EmbeddedCatalogRepository
            .load()
            .expect("catalog should parse");
        assert!(catalog
            .rule_sets
            .iter()
            .filter(|rule_set| rule_set.format == "mrs")
            .all(|rule_set| rule_set.content_audit == "opaque"));
    }

    #[test]
    fn removed_legacy_catalog_is_absent() {
        assert!(!EMBEDDED_CATALOG
            .to_ascii_lowercase()
            .contains("divineengine"));
    }
}
