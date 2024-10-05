use crate::active_players;
use crate::census::event::EventNames;
use crate::census::subscription::{
    CharacterSubscription, EventSubscription, SubscriptionSettings, WorldSubscription,
};
use crate::census::Action;
use crate::census::{CensusMessage, REALTIME_URL};
use crate::event_handlers::receive_events;
use async_trait::async_trait;
use ezsockets::{ClientConfig, CloseCode, CloseFrame};
use metrics::counter;
use std::thread;
use tracing::{debug, error, info};
use url::Url;

struct CensusRealtimeClient {
    client: ezsockets::Client<Self>,
    subscription: SubscriptionSettings,
    state: State,
}

#[derive(Clone)]
pub struct State {
    pub active_players: active_players::ActivePlayerDb,
}

#[derive(Debug, Clone)]
pub struct RealtimeClientConfig {
    pub environment: String,
    pub service_id: String,
    pub realtime_url: Option<Url>,
}

#[derive(thiserror::Error, Debug)]
pub enum RealtimeError {
    #[error("EzSockets error occurred: {0}")]
    EzSocketsError(#[from] ezsockets::Error),
    #[error("Serde error occurred: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[async_trait]
impl ezsockets::ClientExt for CensusRealtimeClient {
    type Call = ();

    async fn on_text(&mut self, text: String) -> Result<(), ezsockets::Error> {
        // info!("received message: {text}");
        let parsed_message: Result<CensusMessage, serde_json::Error> = serde_json::from_str(&text);
        match parsed_message {
            Ok(message) => handle_census_msg(&mut self.client, &self.subscription, self.state.clone(), message)?,
            Err(error) => error!("Failed to parse message: {text} - {error}"),
        }

        Ok(())
    }

    async fn on_binary(&mut self, bytes: Vec<u8>) -> Result<(), ezsockets::Error> {
        info!("received bytes: {bytes:?}");
        Ok(())
    }

    async fn on_call(&mut self, call: Self::Call) -> Result<(), ezsockets::Error> {
        let () = call;
        Ok(())
    }

    async fn on_connect(&mut self) -> Result<(), ezsockets::Error> {
        info!("connected");
        Ok(())
    }
}

fn handle_census_msg(
    client: &mut ezsockets::Client<CensusRealtimeClient>,
    subscription: &SubscriptionSettings,
    state: State,
    message: CensusMessage,
) -> Result<(), RealtimeError> {
    match message {
        CensusMessage::ConnectionStateChanged { connected } => {
            handle_connection_state(connected, subscription, client)?;
        }
        CensusMessage::Heartbeat { .. } => {
            counter!("realtime_messages_received_heartbeat").increment(1);
        }
        CensusMessage::ServiceStateChanged { .. } => {}
        CensusMessage::ServiceMessage { payload } => {
            thread::spawn(move || receive_events(payload, &state.active_players));
        }
        CensusMessage::Subscription { subscription } => {
            debug!("Subscribed: {:?}", subscription);
        }
    }

    Ok(())
}

fn handle_connection_state(connected: bool, subscription: &SubscriptionSettings, client: &ezsockets::Client<CensusRealtimeClient>) -> Result<(), RealtimeError> {
    if !connected {
        error!("Disconnected from Census!");
        return Ok(());
    }

    info!("Connected to Census!");

    counter!("realtime_total_connections").increment(1);

    debug!(
        "Subscribing with {:?}",
        serde_json::to_string(&Action::Subscribe(subscription.clone()))?
    );

    let subscript_send = client.text(serde_json::to_string(&Action::Subscribe(
        subscription.clone(),
    ))?);

    match subscript_send {
        Ok(_) => {
            info!("Subscribed to Census!");
            // TODO: Handle subscription response
            Ok(())
        }
        Err(err) => {
            error!("Failed to send subscription: {:?}", err);
            client
                .close(Some(CloseFrame {
                    code: CloseCode::Normal,
                    reason: "adios!".to_string(),
                }))
                .unwrap();
            Ok(())
        }
    }
}

fn get_census_address(config: RealtimeClientConfig) -> String {
    let base_url = match config.realtime_url {
        Some(url) => url,
        None => match Url::parse(REALTIME_URL) {
            Ok(url) => url,
            Err(err) => {
                error!(
                    "Failed to parse compiled constant REALTIME_URL into Url: {:?}",
                    err
                );
                return String::new();
            }
        },
    };
    format!(
        "{}?environment={}&service-id=s:{}",
        base_url, config.environment, config.service_id
    )
}

pub fn get_subscription_settings() -> SubscriptionSettings {
    SubscriptionSettings {
        event_names: Some(EventSubscription::Ids(vec![EventNames::GainExperience])),
        characters: Some(CharacterSubscription::All),
        worlds: Some(WorldSubscription::All),
        logical_and_characters_with_worlds: Some(true),
        ..SubscriptionSettings::default()
    }
}

pub async fn client(realtime_client_config: RealtimeClientConfig, state: State) {
    let url = match Url::parse(&get_census_address(realtime_client_config.clone())) {
        Ok(url) => url,
        Err(err) => {
            error!("Failed to parse URL: {:?}", err);
            error!("Unable to start realtime client");
            return;
        }
    };

    let config = ClientConfig::new(url);

    info!("Setting up Census websocket client");

    let (_handle, future) = ezsockets::connect(move |client| CensusRealtimeClient {
        client,
        subscription: get_subscription_settings(),
        state,
    }, config).await;
    tokio::spawn(async move {
        future.await.unwrap();
    });

    // loop {
    //     let msg = match socket.read() {
    //         Ok(msg) => msg,
    //         Err(err) => {
    //             handle_websocket_error(&err);
    //             continue;
    //         }
    //     };
    //
    //     match handle_ws_msg(&mut socket, state.clone(), get_subscription_settings(), msg) {
    //         Ok(()) => {}
    //         Err(err) => {
    //             increment_counter!("realtime_messages_received_total_errored");
    //             error!("{:?}", err);
    //         }
    //     }
    // }
}
