mod model;

pub(crate) use model::{CatalogError, CatalogRuleSet, RuleCatalog};

#[cfg(test)]
pub(crate) use model::{RuleSource, RuleSourcePopularity};
