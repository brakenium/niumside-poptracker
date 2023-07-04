use rocket::{
    get,
    response::status::BadRequest,
    routes,
    serde::{json::Json, Serialize}, State,
};

use serde_json::json;
use sqlx::FromRow;
use utoipa::openapi::OpenApi;
use utoipa::ToSchema;
use crate::controllers;
use crate::startup::DbState;

#[derive(Serialize, ToSchema)]
pub struct Error {
    pub error: String,
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
}

#[derive(Serialize, ToSchema, FromRow)]
pub struct PopWorld {
    pub world_id: i32,
    pub world_population: i64,
    pub timestamp: chrono::NaiveDateTime,
}

#[utoipa::path(
    context_path = "/api",
    responses(
        (status = 200, description = "Successful response", body = Response),
        (status = 400, description = "Bad request", body = Error, example = json!(Error { error: "Invalid world ID".to_string() })),
    )
)]
#[get("/population?<world>")]
pub async fn population(
    world: Option<Vec<i32>>,
    db_pool_state: &State<DbState>,
) -> Result<Json<Response>, BadRequest<String>> {
    let world: Vec<i32> = if let Some(world) = world {
        match controllers::world::get_existing(&db_pool_state.pool, &world[..]).await {
            Ok(worlds) => {
                let worlds: Vec<i32> = worlds.into_iter().map(|w| w.0).collect();
                if worlds.len() > 0 {
                    worlds
                } else {
                    return Err(BadRequest(Some(json!({"error": "Invalid world ID" }).to_string())));
                }
            },
            Err(e) => return Err(BadRequest(Some(json!({"error": "Invalid world ID" }).to_string()))),
        }
    } else {
        controllers::world::get_all(&db_pool_state.pool).await
            .map(|worlds| worlds.into_iter().map(|w| w.0).collect())
            .map_err(|e| BadRequest(Some(e.to_string())))?
    };

    let population = sqlx::query!(
        "
        SELECT wp.timestamp, wp.world_id, SUM(lp.amount) AS world_population FROM world_population wp
        JOIN zone_population zp ON wp.population_id = zp.world_population_id
        JOIN faction_population fp ON zp.zone_population_id = fp.zone_population_id
        JOIN loadout_population lp ON fp.faction_population_id = lp.faction_population_id
        AND wp.population_id = (SELECT MAX(wp2.population_id) FROM world_population wp2 WHERE wp2.world_id = wp.world_id)
        WHERE wp.world_id = ANY($1)
        GROUP BY wp.population_id, wp.timestamp, wp.population_id
        ORDER BY wp.timestamp
        ",
        &world[..]
    )
        .fetch_all(&db_pool_state.pool).await
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    let worlds = population
        .into_iter()
        .map(|p| PopWorld {
            world_id: p.world_id,
            world_population: p.world_population.unwrap_or(0),
            timestamp: p.timestamp,
        })
        .collect();

    let response = Response {
        result: PossibleResults::PopResult(worlds),
    };

    Ok(Json(response))
}

#[get("/openapi.json")]
fn serve_api_doc(openapi: &State<OpenApi>) -> Json<OpenApi> {
    Json(openapi.inner().clone())
}

pub fn routes() -> Vec<rocket::Route> {
    routes![population, serve_api_doc]
}
