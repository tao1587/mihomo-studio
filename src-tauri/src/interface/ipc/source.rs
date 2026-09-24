use crate::{
    application::services::ApplicationServices,
    application::source,
    domain::compile::{ConvertNodeTextRequest, ConvertNodeTextResult},
    domain::source::{
        NodeTextInspectionRequest, SourceInspectionSummary, SubscriptionInspectionRequest,
    },
};

#[tauri::command]
pub async fn inspect_subscription(
    services: tauri::State<'_, ApplicationServices>,
    request: SubscriptionInspectionRequest,
) -> Result<SourceInspectionSummary, String> {
    source::inspect_subscription(services.subscription_gateway_factory(), request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn inspect_node_text(
    request: NodeTextInspectionRequest,
) -> Result<SourceInspectionSummary, String> {
    source::inspect_node_text(request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn convert_node_text(request: ConvertNodeTextRequest) -> Result<ConvertNodeTextResult, String> {
    source::convert_node_text(request).map_err(|error| error.to_string())
}
