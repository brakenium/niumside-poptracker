use metrics_exporter_prometheus::PrometheusHandle;
use rocket::{get, routes, State};

#[get("/")]
const fn index() -> &'static str {
    "Hello, world!"
}

#[get("/metrics")]
fn prom_metrics(prometheus: &State<PrometheusHandle>) -> String {
    prometheus.render()
}
pub fn get_routes() -> Vec<rocket::Route> {
    routes![index, prom_metrics]
}
