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

async fn creds(google: &GoogleConfig) -> Authenticator<HttpsConnector<HttpConnector>> {
    oauth2::ServiceAccountAuthenticator::builder(
        google.auth.clone(),
    )
        .build().await.unwrap()
}

pub async fn get_hub(google: &GoogleConfig) -> CalendarHub<HttpsConnector<HttpConnector>> {
    let auth = creds(&google).await;
    let hub = CalendarHub::new(hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().unwrap().https_or_http().enable_http1().build()), auth);

    hub
}

pub async fn get_next_week(google: &GoogleConfig, calendar_id: &str) -> Events {
    let from_date = Utc::now();
    let to_date = from_date + chrono::Duration::days(7);

    let hub = get_hub(&google).await;

    let events = hub.events().list(calendar_id)
        .time_zone("Europe/Amsterdam")
        .time_min(from_date)
        .time_max(to_date)
        .doit().await.unwrap();

    events.1
}

pub async fn get_colors(google: &GoogleConfig) -> Colors {
    let hub = get_hub(&google).await;

    hub.colors().get()
        .doit().await.unwrap().1
}

pub async fn get_event_color(google: &GoogleConfig, event: &Event) -> Option<String> {
    let colors = get_colors(&google).await;

    let color_id = match event.color_id.clone() {
        Some(color_id) => color_id,
        None =>  {
            return None;
        }
    };

    let event_colors = colors.event.clone().unwrap();

    info!("Color ID: {}", color_id);

    let color = event_colors.get(&color_id).unwrap();

    Some(color.foreground.clone().unwrap())
}

async fn add_cal_to_list(google: &GoogleConfig, calendar_id: String) {
    let hub = get_hub(&google).await;

    let calendar_list_entry = CalendarListEntry {
        id: Some(calendar_id),
        // primary: Some(true),
        ..Default::default()
    };

    hub.calendar_list().insert(calendar_list_entry)
        .doit().await.unwrap();
}

pub async fn get_calendar_color(google: &GoogleConfig, calendar_id: &str) -> Option<String> {
    let colors = get_colors(&google).await;

    let calendar_colors = colors.calendar.clone().unwrap();

    let hub = get_hub(&google).await;

    let calendar_result = hub.calendar_list().get(calendar_id).doit().await;

    let calendar = match calendar_result {
        Ok(cal) => cal,
        Err(_) => {
            add_cal_to_list(&google, calendar_id.to_string()).await;
            hub.calendar_list().get(calendar_id).doit().await.unwrap()
        }
    };

    let color_id = calendar.1.color_id.unwrap();

    let color = calendar_colors.get(&color_id).unwrap();

    Some(color.background.clone().unwrap())
}
