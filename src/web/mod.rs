mod api;

use metrics_exporter_prometheus::PrometheusHandle;
use rocket::{Build, get, Rocket, routes, State};

#[get("/")]
const fn index() -> &'static str {
    "Hello, world!"
}

#[get("/metrics")]
fn prom_metrics(prometheus: &State<PrometheusHandle>) -> String {
    prometheus.render()
}

pub fn init() -> Rocket<Build> {
    let mut rocket: Rocket<Build> = rocket::build()
        .mount("/", routes![index, prom_metrics])
        .mount("/api", routes![api::population]);

    rocket
}
