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
    let creds = oauth2::ServiceAccountAuthenticator::builder(
        google.auth.clone(),
    )
        .build().await.ok()?;
    
    Some(creds)
}

pub async fn get_hub(google: &GoogleConfig) -> Option<CalendarHub<HttpsConnector<HttpConnector>>> {
    let auth = creds(google).await?;

    let tls_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots().ok()?
        .https_or_http()
        .enable_http1()
        .build();

    let http_client = hyper::Client::builder().build(tls_connector);

    let hub = CalendarHub::new(http_client, auth);

    Some(hub)
}

pub async fn get_next_week(google: &GoogleConfig, calendar_id: &str) -> Option<Events> {
    let from_date = Utc::now();
    let to_date = from_date + chrono::Duration::days(7);

    let hub = get_hub(google).await?;

    let events = hub.events().list(calendar_id)
        .time_zone("Europe/Amsterdam")
        .time_min(from_date)
        .time_max(to_date)
        .doit().await.ok()?;

    Some(events.1)
}

pub async fn get_colors(google: &GoogleConfig) -> Option<Colors> {
    let hub = get_hub(google).await?;

    let req = hub.colors().get().doit().await.ok()?;

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

    // let calendar = match calendar_result {
    //     Ok(cal) => cal,
    //     Err(_) => {
    //         if add_cal_to_list(google, calendar_id.to_string()).await.is_err() {
    //             return None;
    //         }
    //         match hub.calendar_list().get(calendar_id).doit().await {
    //             Ok(cal) => cal,
    //             Err(_) => {
    //                 return None;
    //             }
    //         }
    //     }
    // };

    let calendar = if let Ok(cal) = calendar_result {
        cal
    } else {
        if add_cal_to_list(google, calendar_id.to_string()).await.is_err() {
            return None;
        }
        match hub.calendar_list().get(calendar_id).doit().await {
            Ok(cal) => cal,
            Err(_) => {
                return None;
            }
        }
    };

    let color_id = calendar.1.color_id?;

    let color = calendar_colors.get(&color_id)?;

    color.background.clone()
}
