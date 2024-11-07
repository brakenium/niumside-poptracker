#[cfg(feature = "census_api")]
use rocket::response::status::BadRequest;
#[cfg(feature = "census_api")]
use rocket::serde::json::Json;
#[cfg(feature = "census_api")]
use rocket::serde::Serialize;
#[cfg(feature = "census_api")]
use thiserror::Error;
#[cfg(feature = "census_api")]
use utoipa::ToSchema;
#[cfg(feature = "census_api")]
use crate::startup::DbState;
#[cfg(feature = "census_api")]
use crate::web::State;
#[cfg(feature = "census_api")]
use rocket::routes;
#[cfg(feature = "census_api")]
use rocket::get;

#[cfg(feature = "census_api")]
use crate::controllers::population::{get_current_tree, PopulationApiResponse, ZoneBreakdown};

#[derive(Error, Debug, Serialize, ToSchema)]
#[cfg(feature = "census_api")]
pub enum Error {
    #[error("No data available")]
    NoDataAvailable,
}

#[derive(Serialize, ToSchema)]
#[cfg(feature = "census_api")]
pub struct Response {
    #[serde(flatten)]
    pub result: PossibleResults,
}

#[derive(Serialize, ToSchema)]
#[cfg(feature = "census_api")]
pub enum PossibleResults {
    #[serde(rename = "pop")]
    PopResult(PopulationApiResponse),
    #[serde(rename = "zone")]
    ZoneResult(ZoneBreakdown),
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
#[get("/population?<world>&<zone>&<team>&<loadout>")]
#[cfg(feature = "census_api")]
pub async fn population(
    world: Option<Vec<i32>>,
    zone: Option<Vec<i32>>,
    team: Option<Vec<i16>>,
    loadout: Option<Vec<i16>>,
    db_pool_state: &State<DbState>,
) -> Result<Json<Response>, BadRequest<Json<Response>>> {
    let Some(result) = get_current_tree(
        &db_pool_state.pool,
        world.as_deref(),
        zone.as_deref(),
        team.as_deref(),
        loadout.as_deref(),
    )
        .await
    else {
        let response = Response {
            result: PossibleResults::Error(Error::NoDataAvailable),
        };

        return Err(BadRequest(Json(response)));
    };

    let response = Response {
        result: PossibleResults::PopResult(result),
    };

    Ok(Json(response))
}

#[allow(clippy::no_effect_underscore_binding)]
#[cfg(feature = "census_api")]
pub fn routes() -> Vec<rocket::Route> {
    routes![
        population
    ]
}
