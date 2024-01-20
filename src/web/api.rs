use rocket::{
    get,
    response::status::BadRequest,
    routes,
    serde::{json::Json, Serialize}, State,
};

use thiserror::Error;

use utoipa::openapi::OpenApi;
use utoipa::ToSchema;
use crate::controllers::population::{get_current_tree, PopWorld};
use crate::startup::DbState;

#[derive(Error, Debug, Serialize, ToSchema)]
pub enum Error {
    #[error("No data available")]
    NoDataAvailable,
}

#[derive(Serialize, ToSchema)]
pub struct Response {
    #[serde(flatten)]
    pub result: PossibleResults,
}

#[derive(Serialize, ToSchema)]
pub enum PossibleResults {
    #[serde(rename = "pop")]
    PopResult(Vec<PopWorld>),
    #[serde(rename = "error")]
    Error(Error),
}

#[utoipa::path(
context_path = "/api",
responses(
(status = 200, description = "Successful response", body = Response),
(status = 400, description = "Bad request", body = Error, example = json ! (Error::NoDataAvailable)),
)
)]
#[get("/population?<world>&<zone>&<faction>&<team>&<loadout>")]
pub async fn population(
    world: Option<Vec<i32>>,
    zone: Option<Vec<i32>>,
    faction: Option<Vec<i16>>,
    team: Option<Vec<i16>>,
    loadout: Option<Vec<i16>>,
    db_pool_state: &State<DbState>,
) -> Result<Json<Response>, BadRequest<Json<Response>>> {
    let Some(result) = get_current_tree(
        &db_pool_state.pool,
        world.as_deref(),
        zone.as_deref(),
        faction.as_deref(),
        team.as_deref(),
        loadout.as_deref(),
    ).await else {
        let response = Response {
            result: PossibleResults::Error(Error::NoDataAvailable),
        };

        return Err(BadRequest(Some(Json(response))));
    };

    let response = Response {
        result: PossibleResults::PopResult(result),
    };

    Ok(Json(response))
}

#[get("/openapi.json")]
fn serve_api_doc(openapi: &State<OpenApi>) -> Json<OpenApi> {
    Json(openapi.inner().clone())
}

#[allow(clippy::no_effect_underscore_binding)]
pub fn routes() -> Vec<rocket::Route> {
    routes![population, serve_api_doc]
}
