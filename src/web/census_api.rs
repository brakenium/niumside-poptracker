use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

use crate::controllers::population::{get_current_tree, PopulationApiResponse, ZoneBreakdown};
use crate::web::AppState;

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
    PopResult(PopulationApiResponse),
    #[serde(rename = "zone")]
    ZoneResult(ZoneBreakdown),
    #[serde(rename = "error")]
    Error(Error),
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PopulationQuery {
    pub world: Option<Vec<i32>>,
    pub zone: Option<Vec<i32>>,
    pub team: Option<Vec<i16>>,
    pub loadout: Option<Vec<i16>>,
}

#[utoipa::path(
    get,
    path = "/api/population",
    responses(
        (status = 200, description = "Successful response", body = Response),
        (status = 400, description = "Bad request", body = Error)
    )
)]
pub async fn population(
    State(state): State<AppState>,
    Query(query): Query<PopulationQuery>,
) -> Result<Json<Response>, (StatusCode, Json<Response>)> {
    let Some(result) = get_current_tree(
        &state.db_pool,
        query.world.as_deref(),
        query.zone.as_deref(),
        query.team.as_deref(),
        query.loadout.as_deref(),
    )
    .await
    else {
        let response = Response {
            result: PossibleResults::Error(Error::NoDataAvailable),
        };

        return Err((StatusCode::BAD_REQUEST, Json(response)));
    };

    let response = Response {
        result: PossibleResults::PopResult(result),
    };

    Ok(Json(response))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/population", get(population))
}
