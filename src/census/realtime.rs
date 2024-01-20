use crate::active_players;
use crate::census::event::EventNames;
use crate::census::subscription::{
    CharacterSubscription, EventSubscription, SubscriptionSettings, WorldSubscription,
};
use crate::census::CensusMessage;
use crate::census::{Action, AuraxisError};
use crate::event_handlers::receive_events;
use metrics::increment_counter;
use std::net::TcpStream;
use std::thread;
use tracing::{debug, error, info};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Error, Message, WebSocket};
use url::Url;

pub const REALTIME_URL: &str = "wss://push.planetside2.com/streaming";

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_realtime_url_parsing() {
        let parsed_url = Url::parse(REALTIME_URL);
        assert!(parsed_url.is_ok(), "Failed to parse REALTIME_URL");
    }
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

fn handle_ws_msg(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    state: State,
    subscription_config: SubscriptionSettings,
    msg: Message,
) -> Result<(), AuraxisError> {
    match msg {
        Message::Text(text) => {
            // info!("Received: {}", text);
            let message: CensusMessage = serde_json::from_str(&text)?;

            match message {
                CensusMessage::ConnectionStateChanged { connected } => {
                    if connected {
                        info!("Connected to Census!");

                        increment_counter!("realtime_total_connections");

                        debug!(
                            "Subscribing with {:?}",
                            serde_json::to_string(&Action::Subscribe(
                                (subscription_config).clone()
                            ))?
                        );

                        socket.send(Message::Text(serde_json::to_string(&Action::Subscribe(
                            subscription_config,
                        ))?))?;
                    }
                }
                CensusMessage::Heartbeat { .. } => {
                    increment_counter!("realtime_messages_received_heartbeat");
                }
                CensusMessage::ServiceStateChanged { .. } => {}
                CensusMessage::ServiceMessage { payload } => {
                    thread::spawn(|| receive_events(payload, state.active_players));
                }
                CensusMessage::Subscription { subscription } => {
                    debug!("Subscribed: {:?}", subscription);
                }
            }
        }
        Message::Binary(_) | Message::Pong(_) | Message::Frame(_) => {}
        Message::Ping(ping) => {
            socket.send(Message::Pong(ping))?;
        }
        Message::Close(close) => {
            increment_counter!("realtime.total_closed_connections");
            if let Some(close_frame) = close {
                error!(
                    "Websocket closed. Code: {}, Reason: {}",
                    close_frame.code, close_frame.reason
                );
            }
        }
    }

    Ok(())
}

pub fn client(realtime_client_config: RealtimeClientConfig, state: State) {
    let census_addr = format!(
        "{}?environment={}&service-id=s:{}",
        realtime_client_config
            .realtime_url
            .unwrap_or_else(|| Result::expect(
                Url::parse(REALTIME_URL),
                "Failed to parse constant realtime url"
            )),
        realtime_client_config.environment,
        realtime_client_config.service_id
    );

    info!("Connecting to realtime at {}", census_addr);

    let mut socket = match connect(&census_addr) {
        Ok((socket, _)) => socket,
        Err(err) => {
            error!("Failed to connect to realtime: {:?}", err);
            // Exit program
            return;
        }
    };

    let subscription_settings = SubscriptionSettings {
        event_names: Some(EventSubscription::Ids(vec![EventNames::GainExperience])),
        characters: Some(CharacterSubscription::All),
        worlds: Some(WorldSubscription::All),
        logical_and_characters_with_worlds: Some(true),
        ..SubscriptionSettings::default()
    };

    loop {
        let msg = match socket.read() {
            Ok(msg) => msg,
            Err(err) => {
                increment_counter!("realtime_messages_received_total_errored");

                match err {
                    Error::ConnectionClosed => {
                        error!("Connection closed");
                        increment_counter!("realtime_total_closed_connections");
                        // TODO: Reconnect
                    }
                    Error::AlreadyClosed
                    | Error::Io(_)
                    | Error::Tls(_)
                    | Error::Capacity(_)
                    | Error::Protocol(_)
                    | Error::Utf8
                    | Error::Url(_)
                    | Error::Http(_)
                    | Error::HttpFormat(_)
                    | Error::WriteBufferFull(_)
                    | Error::AttackAttempt => {
                        error!(
                            "Unhandled realtime client websocket error on read: {:?}",
                            err
                        );
                    }
                }
                return;
            }
        };
        match handle_ws_msg(
            &mut socket,
            state.clone(),
            subscription_settings.clone(),
            msg,
        ) {
            Ok(_) => {}
            Err(err) => {
                increment_counter!("realtime_messages_received_total_errored");
                error!("{:?}", err);
            }
        }
    }
}

// pub async fn init(
//     census_config: configuration::CensusConfig,
//     worlds: Vec<configuration::WorldConfig>,
// ) -> Result<RealtimeClient, AuraxisError> {
//     let realtime_config = RealtimeClientConfig {
//         service_id: census_config.service_id,
//         realtime_url: Some(census_config.realtime_base_url.to_string()),
//         ..RealtimeClientConfig::default()
//     };
//
//     let worlds = worlds.iter().map(|value| value.id).collect();
//
//     let subscription = SubscriptionSettings {
//         event_names: Some(EventSubscription::Ids(vec![EventNames::GainExperience])),
//         characters: Some(CharacterSubscription::All),
//         worlds: Some(WorldSubscription::Ids(worlds)),
//         logical_and_characters_with_worlds: Some(true),
//         ..SubscriptionSettings::default()
//     };
//
//     let mut client = RealtimeClient::new(realtime_config);
//
//     client.subscribe(subscription);
//
//     client.connect().await?;
//     Ok(client)
// }
