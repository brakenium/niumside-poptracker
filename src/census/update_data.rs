use sqlx::PgPool;
use tracing::{error, info};
use crate::controllers::zone::Zone;

const LITHAFALCON_BASE_URL: &str = "https://census.lithafalcon.cc";

#[derive(serde::Deserialize)]
struct ZoneResponse {
    zone_list: Vec<Zone>,
}

pub async fn update_from_lithafalcon(db_pool: &PgPool) {
    let request_url = format!("{LITHAFALCON_BASE_URL}/get/PS2/zone?c:censusJSON=false&c:lang=en&c:show=zone_id,name,description");
    let request = match reqwest::get(request_url)
        .await {
            Ok(response) => response,
            Err(e) => {
                error!("Error while requesting zones from lithafalcon: {e}");
                return;
            }
        }
        .json::<ZoneResponse>()
        .await;

    match request {
        Ok(response) => {
            let zones = response.zone_list;
            info!("Got {} zones from lithafalcon", zones.len());

            let mut transaction = match db_pool.begin().await {
                Ok(transaction) => transaction,
                Err(e) => {
                    error!("Error while starting transaction: {e}");
                    return;
                }
            };

            for zone in zones {
                let zone_name = match zone.name {
                    Some(name) => name.en,
                    None => None,
                };

                let zone_description = match zone.description {
                    Some(description) => description.en,
                    None => None,
                };

                match sqlx::query!(
                "INSERT INTO zone
                    (zone_id, name, description)
                    VALUES ($1, $2, $3)
                    ON CONFLICT (zone_id) DO UPDATE SET name = $2, description = $3",
                zone.zone_id, zone_name, zone_description
                )
                .execute(&mut *transaction)
                .await {
                    Ok(_) => {},
                    Err(e) => {
                        error!("Error while inserting zone into database: {e}");
                    }
                }
            }

            match transaction.commit().await {
                Ok(()) => {},
                Err(e) => {
                    error!("Error while committing transaction: {e}");
                }
            }
        },
        Err(e) => {
            error!("Error while requesting zones from lithafalcon: {e}");
        }
    }
}

pub async fn run(db_pool: &PgPool) {
    loop {
        update_from_lithafalcon(db_pool).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(60 * 60)).await;
    }
}
