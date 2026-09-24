mod model;
mod parser;

pub(crate) use model::{
    has_credentials, is_http_url, is_loopback_url, FetchRoute, NodeTextInspectionRequest,
    SourceError, SourceInspectionSummary, SubscriptionInspectionRequest, SubscriptionUrl,
};
pub(crate) use parser::{inspect_bytes, inspect_node_text, summarize, MAX_SOURCE_BYTES};
