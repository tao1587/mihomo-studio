use std::sync::Arc;

use application::services::ApplicationServices;
use infrastructure::{
    catalog::EmbeddedCatalogRepository, source::ReqwestSubscriptionGatewayFactory,
};

mod application;
mod domain;
mod infrastructure;
mod interface;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let services = ApplicationServices::new(
        Arc::new(EmbeddedCatalogRepository),
        Arc::new(ReqwestSubscriptionGatewayFactory),
    );

    tauri::Builder::default()
        .manage(services)
        .invoke_handler(tauri::generate_handler![
            interface::ipc::catalog::get_rule_catalog,
            interface::ipc::compile::compile_strict_profile,
            interface::ipc::profile::build_profile_draft,
            interface::ipc::source::convert_node_text,
            interface::ipc::source::inspect_subscription,
            interface::ipc::source::inspect_node_text,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mihomo Studio");
}
