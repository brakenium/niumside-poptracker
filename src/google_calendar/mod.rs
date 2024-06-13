pub mod formatting;

use calendar3::api::{CalendarListEntry, Colors, Event, Events};
use calendar3::CalendarHub;
use calendar3::hyper::client::HttpConnector;
use calendar3::hyper_rustls::HttpsConnector;
use calendar3::oauth2::authenticator::Authenticator;
use chrono::{Utc};
use crate::storage::configuration::GoogleConfig;
use google_calendar3::oauth2;
use google_calendar3::{hyper, hyper_rustls, chrono};
use tracing::info;

async fn creds(google: &GoogleConfig) -> Option<Authenticator<HttpsConnector<HttpConnector>>> {
    let creds = match oauth2::ServiceAccountAuthenticator::builder(
        google.auth.clone(),
    )
        .build().await {
        Ok(creds) => creds,
        Err(err) => {
            info!("Failed to get creds for Google calendar: {:?}", err);
            return None;
        }
    };
    
    Some(creds)
}

pub async fn get_hub(google: &GoogleConfig) -> Option<CalendarHub<HttpsConnector<HttpConnector>>> {
    let auth = creds(google).await?;

    let tls_connector = match hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots() {
        Ok(connector) => connector
            .https_or_http()
            .enable_http1()
            .build(),
        Err(err) => {
            info!("Failed to build TLS connector for Google calendar: {:?}", err);
            return None;
        }
    };

    let http_client = hyper::Client::builder().build(tls_connector);

    let hub = CalendarHub::new(http_client, auth);

    Some(hub)
}

pub async fn get_next_week(google: &GoogleConfig, calendar_id: &str) -> Option<Events> {
    let from_date = Utc::now();
    let to_date = from_date + chrono::Duration::days(7);

    let hub = get_hub(google).await?;

    let events = match hub.events().list(calendar_id)
        .time_zone("Europe/Amsterdam")
        .time_min(from_date)
        .time_max(to_date)
        .doit().await {
        Ok(events) => events,
        Err(err) => {
            info!("Failed to get events for Google calendar: {:?}", err);
            return None;
        }
    };

    Some(events.1)
}

pub async fn get_colors(google: &GoogleConfig) -> Option<Colors> {
    let hub = get_hub(google).await?;

    let req = match hub.colors().get().doit().await {
        Ok(req) => req,
        Err(err) => {
            info!("Failed to get colors for Google calendar: {:?}", err);
            return None;
        }
    };

    Some(req.1)
}

pub async fn get_event_color(google: &GoogleConfig, event: &Event) -> Option<String> {
    let colors = get_colors(google).await;

    let color_id = event.color_id.clone()?;

    let event_colors = colors?.event?;

    info!("Color ID: {}", color_id);

    let color = event_colors.get(&color_id)?;

    color.foreground.clone()
}

async fn add_cal_to_list(google: &GoogleConfig, calendar_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let hub = get_hub(google).await.ok_or("Failed to get hub for google calendar")?;

    let calendar_list_entry = CalendarListEntry {
        id: Some(calendar_id),
        // primary: Some(true),
        ..Default::default()
    };

    hub.calendar_list().insert(calendar_list_entry)
        .doit().await?;

    Ok(())
}

pub async fn get_calendar_color(google: &GoogleConfig, calendar_id: &str) -> Option<String> {
    let colors = get_colors(google).await;

    let calendar_colors = colors?.calendar?;

    let hub = get_hub(google).await?;

    let calendar_result = hub.calendar_list().get(calendar_id).doit().await;

    let calendar = if let Ok(cal) = calendar_result {
        cal
    } else {
        match add_cal_to_list(google, calendar_id.to_string()).await {
            Ok(()) => {},
            Err(err) => {
                info!("Failed to add calendar to list: {:?}", err);
                return None;
            }
        }
        match hub.calendar_list().get(calendar_id).doit().await {
            Ok(cal) => cal,
            Err(err) => {
                info!("Failed to get calendar: {:?}", err);
                return None;
            }
        }
    };

    let color_id = calendar.1.color_id?;

    let color = calendar_colors.get(&color_id)?;

    color.background.clone()
}
