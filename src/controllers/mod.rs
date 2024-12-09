use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod character;
pub mod faction;
pub mod population;
pub mod user;
pub mod world;
pub mod zone;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Languages {
    pub en: Option<String>,
}
