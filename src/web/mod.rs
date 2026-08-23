#[cfg(feature = "census_api")]
mod census_api;

pub use axum::extract::State;

use axum::routing::get;
use axum::Router;
use metrics_exporter_prometheus::PrometheusHandle;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::logging;
#[cfg(feature = "database")]
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub(crate) prometheus: PrometheusHandle,
    #[cfg(feature = "database")]
    pub(crate) db_pool: PgPool,
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Successful response", body = String)
    )
)]
pub async fn prom_metrics(State(state): State<AppState>) -> String {
    state.prometheus.render()
}

#[cfg(feature = "census_api")]
#[derive(OpenApi)]
#[openapi(paths(prom_metrics, census_api::population))]
pub struct ApiDoc;

#[cfg(not(feature = "census_api"))]
#[derive(OpenApi)]
#[openapi(paths(prom_metrics))]
pub struct ApiDoc;

pub fn init(#[cfg(feature = "database")] db_pool: PgPool) -> (Router<AppState>, AppState) {
    let http_state = AppState {
        prometheus: logging::metrics(),
        #[cfg(feature = "database")]
        db_pool,
    };

    let mut router = Router::new().route("/metrics", get(prom_metrics));
    let swagger = SwaggerUi::new("/api/docs").url("/api/openapi.json", ApiDoc::openapi());

    #[cfg(feature = "census_api")]
    {
        router = router.nest("/api", census_api::router());
    }

    (router.merge(swagger), http_state)
}
