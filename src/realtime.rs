use auraxis::realtime::client::{RealtimeClient, RealtimeClientConfig};
use auraxis::realtime::event::{Event, EventNames};
use auraxis::realtime::subscription::{
    CharacterSubscription, EventSubscription, SubscriptionSettings, WorldSubscription,
};
use auraxis::{AuraxisError, WorldID};
use tokio::sync::mpsc::Receiver;

pub async fn init(secrets: shuttle_secrets::SecretStore) -> Result<Receiver<Event>, AuraxisError> {
    let realtime_config = RealtimeClientConfig {
        service_id: secrets.get("SERVICE_ID").unwrap(),
        realtime_url: secrets.get("REALTIME_URL"),
        ..RealtimeClientConfig::default()
    };

    let subscription = SubscriptionSettings {
        event_names: Some(EventSubscription::Ids(vec![EventNames::GainExperience])),
        characters: Some(CharacterSubscription::All),
        worlds: Some(WorldSubscription::All),
        logical_and_characters_with_worlds: Some(true),
        ..SubscriptionSettings::default()
    };

    let mut client = RealtimeClient::new(realtime_config);

    client.subscribe(subscription);

    client.connect().await
}
