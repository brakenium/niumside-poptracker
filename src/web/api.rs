use rocket::{Build, get, Rocket, routes, State};
use rocket::response::status::BadRequest;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use sqlx::FromRow;
use crate::shuttle::DbState;

#[derive(Serialize, FromRow)]
pub struct PopWorld {
    world_id: i32,
    world_population: Option<i64>,
}

#[get("/population")]
pub async fn population(db_pool_state: &State<DbState>) -> Result<Json<Vec<PopWorld>>, BadRequest<String>> {
    let population = sqlx::query_as!(
        PopWorld,
        "SELECT wp.world_id, SUM(lp.amount) AS world_population FROM world_population wp
        JOIN zone_population zp ON wp.population_id = zp.world_population_id
        JOIN faction_population fp ON zp.zone_population_id = fp.zone_population_id
        JOIN loadout_population lp ON fp.faction_population_id = lp.faction_population_id
        AND wp.population_id = (SELECT MAX(wp2.population_id) FROM world_population wp2 WHERE wp2.world_id = wp.world_id)
        GROUP BY wp.population_id, wp.timestamp, wp.population_id
        ORDER BY wp.timestamp"
    )
        .fetch_all(&db_pool_state.pool).await
        .map_err(|e| BadRequest(Some(e.to_string())))?;
    Ok(Json(population))
}
