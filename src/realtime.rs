use auraxis::{
    realtime::{
        client::{RealtimeClient, RealtimeClientConfig},
        event::{Event, EventNames},
        subscription::{
            CharacterSubscription, EventSubscription, SubscriptionSettings, WorldSubscription,
        },
    },
    AuraxisError,
};
use tokio::sync::mpsc::Receiver;
use crate::storage::configuration;

pub async fn init(census_config: configuration::CensusConfig, worlds: Vec<configuration::WorldConfig>) -> Result<Receiver<Event>, AuraxisError> {
    let realtime_config = RealtimeClientConfig {
        service_id: census_config.service_id,
        realtime_url: Some(census_config.realtime_base_url.to_string()),
        ..RealtimeClientConfig::default()
    };

    let worlds = worlds.iter().map(|value| value.id).collect();

    let subscription = SubscriptionSettings {
        event_names: Some(EventSubscription::Ids(vec![EventNames::GainExperience])),
        characters: Some(CharacterSubscription::All),
        worlds: Some(WorldSubscription::Ids(worlds)),
        logical_and_characters_with_worlds: Some(true),
        ..SubscriptionSettings::default()
    };

    let mut client = RealtimeClient::new(realtime_config);

    client.subscribe(subscription);

    client.connect().await
}
