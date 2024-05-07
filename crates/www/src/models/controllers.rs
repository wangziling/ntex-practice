use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use web_core::response::ServerResponse;

/// Todo model
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct HelloWorld {
    pub greeting: &'static str,
}
