use std::{future::Future, pin::Pin};

use crate::domain::{
    catalog::{CatalogError, RuleCatalog},
    source::{FetchRoute, SourceError, SubscriptionUrl},
};

pub trait CatalogRepository: Send + Sync {
    fn load(&self) -> Result<RuleCatalog, CatalogError>;
}

#[derive(Debug)]
pub enum SubscriptionFetchResult {
    HttpStatus(u16),
    Body(Vec<u8>),
}

pub trait SubscriptionGateway: Send + Sync {
    fn fetch<'a>(
        &'a self,
        source: &'a SubscriptionUrl,
        user_agent: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<SubscriptionFetchResult, SourceError>> + Send + 'a>>;
}

pub trait SubscriptionGatewayFactory: Send + Sync {
    fn create(&self, route: FetchRoute) -> Result<Box<dyn SubscriptionGateway>, SourceError>;
}
