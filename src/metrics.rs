use metrics_exporter_prometheus::PrometheusBuilder;

pub fn install() {
    let builder = PrometheusBuilder::new();
    builder.install().unwrap();
}
