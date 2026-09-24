use crate::{
    application::ports::CatalogRepository,
    domain::profile::{build_draft, BuildDraftRequest, ProfileDraft, ProfileError},
};

pub fn build_profile_draft(
    repository: &dyn CatalogRepository,
    request: BuildDraftRequest,
) -> Result<ProfileDraft, ProfileError> {
    let catalog = repository
        .load()
        .map_err(|error| ProfileError::Catalog(error.to_string()))?;
    build_draft(&catalog, request)
}
