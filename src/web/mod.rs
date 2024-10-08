#[cfg(feature = "census_api")]
mod census_api;

#[cfg(feature = "census_api")]
use crate::controllers::population;
use metrics_exporter_prometheus::PrometheusHandle;
use rocket::{get, routes, Build, Rocket, State};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        // #[cfg(feature = "census_api")]
        // api::population,
        prom_metrics),
    // #[cfg(feature = "census_api")]
    // components(schemas(
    //     api::Response,
    //     api::PossibleResults,
    //     api::Error,
    //     // #[cfg(feature = "census")]
    //     // population::PopulationApiResponse,
    //     // #[cfg(feature = "census")]
    //     // population::PopWorld,
    //     // #[cfg(feature = "census")]
    //     // population::PopZone,
    //     // #[cfg(feature = "census")]
    //     // population::PopTeam,
    //     // #[cfg(feature = "census")]
    //     // population::PopLoadout,
    // ))
)]
pub struct ApiDoc;

#[utoipa::path(
    context_path = "/metrics",
    responses(
        (status = 200, description = "Successful response", body = String, example = json!(
"# HELP realtime_messages_total_sent Total number of messages sent to Census stream
# TYPE realtime_messages_total_sent counter
realtime_messages_total_sent 42

# HELP realtime_total_pings Total number of ping messages sent to Census stream, may include errors
# TYPE realtime_total_pings counter
realtime_total_pings 40

# HELP realtime_messages_received_total_errored Total number of messages received from Census stream that errored
# TYPE realtime_messages_received_total_errored counter
realtime_messages_received_total_errored 3

# HELP niumside_process_loop_iterations The number of times the active player event process loop has ran
# TYPE niumside_process_loop_iterations counter
niumside_process_loop_iterations 1

# HELP realtime_total_connections Total number of connections to Census stream
# TYPE realtime_total_connections counter
realtime_total_connections 1

# HELP realtime_messages_received_heartbeat Total number of heartbeat messages received from Census stream
# TYPE realtime_messages_received_heartbeat counter
realtime_messages_received_heartbeat 1

# HELP realtime_total_resubscriptions Total number of resubscriptions to Census stream
# TYPE realtime_total_resubscriptions counter
realtime_total_resubscriptions 1

# HELP realtime_messages_received_total Total number of messages received from Census stream
# TYPE realtime_messages_received_total counter
realtime_messages_received_total 6438

# HELP niumside_active_players_cleanups Number of times the active_players cleanup ran
# TYPE niumside_active_players_cleanups counter
niumside_active_players_cleanups 1

# HELP niumside_gain_experience_events The number of gain experience events inserted into the active players
# TYPE niumside_gain_experience_events counter
niumside_gain_experience_events 6387

# HELP niumside_active_players Number of active players
# TYPE niumside_active_players gauge
niumside_active_players 1064"
            )
        ),
    )
)]
#[get("/")]
pub fn prom_metrics(prometheus: &State<PrometheusHandle>) -> String {
    prometheus.render()
}

pub fn init() -> Rocket<Build> {
    #[allow(clippy::no_effect_underscore_binding)]
    let rocket: Rocket<Build> = rocket::build()
        .mount("/metrics", routes![prom_metrics])
        .mount(
            "/",
            SwaggerUi::new("/api/<_..>").url("/api/openapi.json", ApiDoc::openapi()),
        );

    #[cfg(feature = "census_api")]
    let rocket = rocket.mount("/api", census_api::routes());

    rocket
}
