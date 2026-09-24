use crate::{
    application::profile,
    application::services::ApplicationServices,
    domain::profile::{BuildDraftRequest, ProfileDraft},
};

#[tauri::command]
pub fn build_profile_draft(
    services: tauri::State<'_, ApplicationServices>,
    request: BuildDraftRequest,
) -> Result<ProfileDraft, String> {
    profile::build_profile_draft(services.catalog_repository(), request)
        .map_err(|error| error.to_string())
}
