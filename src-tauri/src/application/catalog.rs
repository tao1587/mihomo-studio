use crate::{
    application::ports::CatalogRepository,
    domain::catalog::{CatalogError, RuleCatalog},
};

pub fn get_rule_catalog(repository: &dyn CatalogRepository) -> Result<RuleCatalog, CatalogError> {
    repository.load()
}
