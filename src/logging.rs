use metrics::{describe_counter, describe_histogram};
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
            describe_metrics();
        }
        Err(e) => {
            error!("Unable to start Prometheus metrics: {}", e);
        }
    }
}

fn describe_metrics() {
    describe_counter!("niumside_active_players_lock_failed", "Number of times the active_players lock failed");
    describe_counter!("niumside_active_players_cleanups", "Number of times the active_players cleanup ran");
    describe_histogram!("niumside_active_players", "Number of active players");
    describe_counter!("niumside_process_loop_iterations", "The number of times the active player event process loop has ran");
}

fn tracing(log_level: Level) {
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(true)
        .init();
}
