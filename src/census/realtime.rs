use crate::active_players;
use crate::census::event::EventNames;
use crate::census::subscription::{
    CharacterSubscription, EventSubscription, SubscriptionSettings, WorldSubscription,
};
use crate::census::{Action, AuraxisError};
use crate::census::{CensusMessage, REALTIME_URL};
use crate::event_handlers::receive_events;
use metrics::increment_counter;
use std::net::TcpStream;
use std::thread;
use tracing::{debug, error, info};
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{connect, Error, Message, WebSocket};
use url::Url;

#[cfg(test)]
mod tests {
    use crate::census::REALTIME_URL;
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

fn handle_connection_state(
    connected: bool,
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    subscription_config: SubscriptionSettings,
) -> Result<(), AuraxisError> {
    if !connected {
        return Ok(());
    }
    info!("Connected to Census!");

    increment_counter!("realtime_total_connections");

    debug!(
        "Subscribing with {:?}",
        serde_json::to_string(&Action::Subscribe((subscription_config).clone()))?
    );

    socket.send(Message::Text(serde_json::to_string(&Action::Subscribe(
        subscription_config,
    ))?))?;

    Ok(())
}

fn handle_census_msg(
    socket: &mut WebSocket<MaybeTlsStream<TcpStream>>,
    state: State,
    subscription_config: SubscriptionSettings,
    message: CensusMessage,
) -> Result<(), AuraxisError> {
    match message {
        CensusMessage::ConnectionStateChanged { connected } => {
            handle_connection_state(connected, socket, subscription_config)?;
        }
        CensusMessage::Heartbeat { .. } => {
            increment_counter!("realtime_messages_received_heartbeat");
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

            handle_census_msg(socket, state, subscription_config, message)?;
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

fn handle_websocket_error(err: &Error) {
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

pub fn connect_ws(
    realtime_client_config: RealtimeClientConfig,
) -> Option<WebSocket<MaybeTlsStream<TcpStream>>> {
    let census_addr = get_census_address(realtime_client_config);

    info!("Connecting to realtime at {}", census_addr);

    match connect(&census_addr) {
        Ok((socket, _)) => Some(socket),
        Err(err) => {
            error!("Failed to connect to realtime: {:?}", err);
            None
        }
    }
}

pub fn client(realtime_client_config: RealtimeClientConfig, state: &State) {
    let Some(mut socket) = connect_ws(realtime_client_config) else {
        error!("Failed to connect to realtime");
        return;
    };

    loop {
        let msg = match socket.read() {
            Ok(msg) => msg,
            Err(err) => {
                handle_websocket_error(&err);
                continue;
            }
        };

        match handle_ws_msg(&mut socket, state.clone(), get_subscription_settings(), msg) {
            Ok(_) => {}
            Err(err) => {
                increment_counter!("realtime_messages_received_total_errored");
                error!("{:?}", err);
            }
        }
    }
}
