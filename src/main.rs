mod configuration;
mod constants;
use auraxis::event::{Event, EventNames};
use auraxis::subscription::{
    CharacterSubscription, EventSubscription, SubscriptionSettings, WorldSubscription,
};
use auraxis::{
    client::{RealtimeClient, RealtimeClientConfig},
    WorldID,
};
use std::error::Error;
use tokio;
use tracing_subscriber;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let app_config = configuration::Settings::new()?;
    println!("{:?}", app_config);
    println!("{}", app_config.census.realtime_base_url.to_string());

    let realtime_config = RealtimeClientConfig {
        service_id: app_config.census.service_id,
        realtime_url: Some(app_config.census.realtime_base_url.to_string()),
        ..RealtimeClientConfig::default()
    };

    let subscription = SubscriptionSettings {
        event_names: Some(EventSubscription::Ids(vec![EventNames::PlayerLogin])),
        characters: Some(CharacterSubscription::All),
        worlds: Some(WorldSubscription::Ids(vec![WorldID::Jaeger])),
        logical_and_characters_with_worlds: Some(true),
        ..SubscriptionSettings::default()
    };

    let mut client = RealtimeClient::new(realtime_config);

    client.subscribe(subscription);

    let mut events = client.connect().await?;

    while let Some(event) = events.recv().await {
        tokio::spawn(async move {
            match event {
                Event::GainExperience(event) => println!("{:?}", event),
                _ => (),
            }
        });
    }
    Ok(())
}
