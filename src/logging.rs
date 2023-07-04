use metrics::{describe_counter, describe_gauge};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use tracing::{info};

pub fn metrics() -> PrometheusHandle {
    let prometheus_metrics = PrometheusBuilder::new()
        .install_recorder()
        .expect("failed to install recorder");
    info!("Prometheus metrics enabled");
    describe_metrics();
    prometheus_metrics
}

fn describe_metrics() {
    describe_counter!(
        "niumside_active_players_lock_failed",
        "Number of times the active_players lock failed"
    );
    describe_counter!(
        "niumside_active_players_cleanups",
        "Number of times the active_players cleanup ran"
    );
    describe_gauge!("niumside_active_players", "Number of active players");
    describe_counter!(
        "niumside_process_loop_iterations",
        "The number of times the active player event process loop has ran"
    );
    describe_counter!(
        "niumside_gain_experience_events",
        "The number of gain experience events inserted into the active players"
    );
}

#[cfg(feature = "standalone")]
pub fn tracing(log_level: tracing::Level) {
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(true)
        .init();
}
