use metrics_exporter_prometheus::PrometheusBuilder;
use tracing::{error, info, Level};

pub fn init(log_level: Level) {
    tracing(log_level);
    metrics();
}

fn metrics() {
    let prometheus_metrics = PrometheusBuilder::new().install();
    match prometheus_metrics {
        Ok(_m) => {
            info!("Prometheus metrics enabled");
        }
        Err(e) => {
            error!("Unable to start Prometheus metrics: {}", e);
        }
    }
}

fn tracing(log_level: Level) {
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(true)
        .init();
}
