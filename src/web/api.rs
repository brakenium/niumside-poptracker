use rocket::{Build, get, Rocket, routes, State};
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use sqlx::FromRow;
use crate::shuttle::DbState;

#[derive(Serialize)]
pub struct Response {
    result: PossibleResults,
}

#[derive(Serialize)]
pub enum PossibleResults {
    #[serde(rename = "pop")]
    PopResult(Vec<PopWorld>),
}

#[derive(Serialize, FromRow)]
pub struct PopWorld {
    world_id: i32,
    world_population: i64,
    timestamp: chrono::NaiveDateTime,
}

#[get("/population?<world>")]
pub async fn population(world: Option<Vec<i32>>, db_pool_state: &State<DbState>) -> Result<Json<Response>, BadRequest<String>> {
    let world = if let Some(world) = world { world } else {
        let world = sqlx::query!(
            "SELECT world_id FROM world"
        )
            .fetch_all(&db_pool_state.pool).await
            .map_err(|e| BadRequest(Some(e.to_string())))?;

        world.into_iter().map(|w| w.world_id).collect()
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

    let worlds = population.into_iter().map(|p| PopWorld {
        world_id: p.world_id,
        world_population: p.world_population.unwrap_or(0),
        timestamp: p.timestamp,
    }).collect();

    let response = Response {
        result: PossibleResults::PopResult(worlds),
    };

    Ok(Json(response))
}
