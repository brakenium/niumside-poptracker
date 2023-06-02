mod api;

use metrics_exporter_prometheus::PrometheusHandle;
use rocket::{get, routes, Build, Rocket, State};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(paths(api::population, prom_metrics,))]
struct ApiDoc;

#[get("/")]
const fn index() -> &'static str {
    "Hello, world!"
}

#[utoipa::path(
    responses(
        (status = 200, description = "Successful response", body = String),
    )
)]
#[get("/metrics")]
fn prom_metrics(prometheus: &State<PrometheusHandle>) -> String {
    prometheus.render()
}

fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/swagger-ui/<_..>").url("/api-docs/openapi.json", ApiDoc::openapi())
}

pub fn init() -> Rocket<Build> {
    let mut rocket: Rocket<Build> = rocket::build()
        .mount("/", routes![index, prom_metrics])
        .mount("/swagger-ui", swagger_ui())
        .mount("/api", routes![api::population]);

    rocket
}
