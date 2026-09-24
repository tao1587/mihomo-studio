use crate::{
    application::catalog, application::services::ApplicationServices, domain::catalog::RuleCatalog,
};

#[tauri::command]
pub fn get_rule_catalog(
    services: tauri::State<'_, ApplicationServices>,
) -> Result<RuleCatalog, String> {
    catalog::get_rule_catalog(services.catalog_repository()).map_err(|error| error.to_string())
}
