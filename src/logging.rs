use std::net::SocketAddr;
use metrics::{describe_counter, describe_gauge};
use metrics_exporter_prometheus::PrometheusBuilder;
use tracing::{error, info, Level};

pub fn init(address: impl Into<SocketAddr>) {
    // tracing(log_level);
    metrics(address);
}

fn metrics(address: impl Into<SocketAddr>) {
    let prometheus_metrics = PrometheusBuilder::new()
        .with_http_listener(address)
        .install();
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
    describe_gauge!("niumside_active_players", "Number of active players");
    describe_counter!("niumside_process_loop_iterations", "The number of times the active player event process loop has ran");
    describe_counter!("niumside_gain_experience_events", "The number of gain experience events inserted into the active players");
}

fn tracing(log_level: Level) {
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(true)
        .init();
}
