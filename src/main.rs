mod configuration;
mod constants;
use auraxis::client::{RealtimeClient, RealtimeClientConfig};
use auraxis::event::{Event, EventNames, GainExperience};
use auraxis::subscription::{
    CharacterSubscription, EventSubscription, SubscriptionSettings, WorldSubscription,
};
use auraxis::{CharacterID, Loadout, WorldID, ZoneID};
use chrono::{DateTime, Utc};
use config::Source;
use futures::future;
use tracing::info;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio;
use tracing_subscriber;
use rayon::prelude::*;
use tokio::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
struct ActivePlayer {
    zone: ZoneID,
    loadout: Loadout,
    world: WorldID,
    last_change: DateTime<Utc>,
}

type ActivePlayerDb = Arc<Mutex<HashMap<CharacterID, ActivePlayer>>>;

async fn handle_gain_experience(event: GainExperience, active_players: ActivePlayerDb) {
    let mut active_players_lock = active_players.lock().unwrap();
    active_players_lock.insert(
        event.character_id,
        ActivePlayer {
            zone: event.zone_id,
            loadout: event.loadout_id,
            world: event.world_id,
            last_change: event.timestamp,
        },
    );
}

async fn print_active_players(active_players: ActivePlayerDb) -> Option<()> {
    let active_players = active_players.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut zone_breakdown: HashMap<ZoneID, HashMap<Loadout, Vec<ActivePlayer>>> = HashMap::new();
        let mut loadout_breakdown_numbers: HashMap<ZoneID, HashMap<Loadout, u16>> = HashMap::new();

        let active_players_lock = active_players.lock().unwrap();
        active_players_lock.iter().for_each(|(_, player)| {
            zone_breakdown.entry(player.zone)
                .or_insert_with(HashMap::new)
                .entry(player.loadout)
                .or_insert_with(Vec::new)
                .push(player.clone());
            loadout_breakdown_numbers.entry(player.zone)
                .or_insert_with(HashMap::new)
                .entry(player.loadout)
                .and_modify(|v| *v += 1)
                .or_insert(1);
        });

        let zone_breakdown: HashMap<ZoneID, u16> = loadout_breakdown_numbers.par_iter()
            .map(|(zone_id, loadouts)| {
                let all_loadout_pop: Vec<u16> = loadouts.par_iter()
                    .map(|(_, v)| v.clone()).collect();
                (zone_id.clone(), all_loadout_pop.par_iter().sum::<u16>())
            }).collect();
        info!("{:?}", zone_breakdown);
    }
}

async fn clean_active_players(active_players: ActivePlayerDb) -> Option<()> {
    let active_players = active_players.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(30)).await;
        let mut active_players_lock = active_players.lock().unwrap();
        active_players_lock.retain(|_character_id, player| player.last_change + chrono::Duration::minutes(3) > Utc::now() );
        info!("Cleaned active players");
    }
}

async fn receive_events(mut events: Receiver<Event>, active_players: ActivePlayerDb) -> Option<()> {
    while let Some(event) = events.recv().await {
        let active_players = active_players.clone();
        tokio::spawn(async move {
            match event {
                Event::GainExperience(event) => handle_gain_experience(event, active_players).await,
                _ => (),
            }
        });
    }
    Some(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    let app_config = configuration::Settings::new()?;

    let realtime_config = RealtimeClientConfig {
        service_id: app_config.census.service_id,
        realtime_url: Some(app_config.census.realtime_base_url.to_string()),
        ..RealtimeClientConfig::default()
    };

    let worlds = app_config.worlds.iter().map(|value| value.id).collect();

    let subscription = SubscriptionSettings {
        event_names: Some(EventSubscription::Ids(vec![EventNames::GainExperience])),
        characters: Some(CharacterSubscription::All),
        worlds: Some(WorldSubscription::Ids(worlds)),
        logical_and_characters_with_worlds: Some(true),
        ..SubscriptionSettings::default()
    };

    let mut client = RealtimeClient::new(realtime_config);

    client.subscribe(subscription);

    let events = client.connect().await?;

    let active_players: ActivePlayerDb = Arc::new(Mutex::new(HashMap::new()));

    let futures = vec![
        tokio::spawn(print_active_players(active_players.clone())),
        tokio::spawn(receive_events(events, active_players.clone())),
        tokio::spawn(clean_active_players(active_players.clone()))
    ];

    future::join_all(futures).await;
    Ok(())
}
