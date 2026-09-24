use std::sync::Arc;

use super::ports::{CatalogRepository, SubscriptionGatewayFactory};

pub struct ApplicationServices {
    catalog_repository: Arc<dyn CatalogRepository>,
    subscription_gateway_factory: Arc<dyn SubscriptionGatewayFactory>,
}

impl ApplicationServices {
    pub fn new(
        catalog_repository: Arc<dyn CatalogRepository>,
        subscription_gateway_factory: Arc<dyn SubscriptionGatewayFactory>,
    ) -> Self {
        Self {
            catalog_repository,
            subscription_gateway_factory,
        }
    }

    pub fn catalog_repository(&self) -> &dyn CatalogRepository {
        self.catalog_repository.as_ref()
    }

    pub fn subscription_gateway_factory(&self) -> &dyn SubscriptionGatewayFactory {
        self.subscription_gateway_factory.as_ref()
    }
}
