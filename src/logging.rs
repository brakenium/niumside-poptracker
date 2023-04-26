use metrics_exporter_prometheus::PrometheusBuilder;

use crate::configuration::Settings;

pub fn init(app_config: &Settings) {
    tracing(app_config);
    metrics();
}

fn metrics() {
    let builder = PrometheusBuilder::new();
    builder.install().unwrap();

}

fn tracing(app_config: &Settings) {
    tracing_subscriber::fmt()
        .with_max_level(app_config.app.log_level)
        .with_target(true)
        .init();
}
