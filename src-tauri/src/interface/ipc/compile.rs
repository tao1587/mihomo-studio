use crate::{
    application::{compile, services::ApplicationServices},
    domain::compile::{CompileProfileRequest, CompileProfileResult},
};

#[tauri::command]
pub async fn compile_strict_profile(
    services: tauri::State<'_, ApplicationServices>,
    request: CompileProfileRequest,
) -> Result<CompileProfileResult, String> {
    compile::compile_strict_profile(
        services.catalog_repository(),
        services.subscription_gateway_factory(),
        request,
    )
    .await
    .map_err(|error| error.to_string())
}
