use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod faction;
pub mod population;
pub mod world;
pub mod zone;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct Languages {
    pub en: Option<String>,
}
