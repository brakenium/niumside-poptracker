use crate::constants;
use auraxis::{WorldID, ZoneID};
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::env;
use url::Url;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct CensusConfig {
    pub realtime_base_url: Url,
    pub service_id: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct WorldConfig {
    pub id: WorldID,
    pub zones: Option<Vec<ZoneID>>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub census: CensusConfig,
    pub worlds: Vec<WorldConfig>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "production".into());

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("config/default"))
            // Add in the current environment file
            // Default to 'production' env
            // Note that this file is _optional_
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("config/local").required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix(constants::APPLICATION_NAME))
            .build()?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}
