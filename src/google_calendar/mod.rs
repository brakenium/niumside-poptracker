pub mod formatting;

use crate::storage::configuration::GoogleConfig;
use chrono::Utc;
use google_calendar3::CalendarHub;
use google_calendar3::api::{CalendarListEntry, Colors, Event, Events};
use google_calendar3::hyper_rustls::HttpsConnector;
use google_calendar3::hyper_util::client::legacy::connect::HttpConnector;
use google_calendar3::yup_oauth2::authenticator::Authenticator;
use google_calendar3::{hyper_rustls, hyper_util, yup_oauth2};
use tracing::info;

async fn creds(google: &GoogleConfig) -> Option<Authenticator<HttpsConnector<HttpConnector>>> {
    let connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .ok()?
        .https_only()
        .enable_http2()
        .build();

    let executor = hyper_util::rt::TokioExecutor::new();
    let creds = match yup_oauth2::ServiceAccountAuthenticator::with_client(
        google.auth.clone(),
        yup_oauth2::client::CustomHyperClientBuilder::from(
            hyper_util::client::legacy::Client::builder(executor).build(connector),
        ),
    )
    .build()
    .await
    {
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

    let token = auth
        .token(&["https://www.googleapis.com/auth/calendar.readonly"])
        .await
        .ok()?;

    info!("Google calendar token: {:?}", token);

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .ok()?
                .https_or_http()
                .enable_http2()
                .build(),
        );

    let hub = CalendarHub::new(client, auth);

    Some(hub)
}

pub async fn get_next_week(google: &GoogleConfig, calendar_id: &str) -> Option<Events> {
    let from_date = Utc::now();
    let to_date = from_date + chrono::Duration::days(7);

    info!("From date: {}", from_date);
    info!("To date: {}", to_date);

    let hub = get_hub(google).await?;

    info!("Calendar ID: >{}<", calendar_id);
    let events = hub
        .events()
        .list(calendar_id)
        .time_min(from_date)
        .time_max(to_date)
        .single_events(true)
        .max_results(2500)
        .doit()
        .await;

    match events {
        Ok((_, events)) => {
            for event in events.items.unwrap_or_default() {
                info!("Event: {:?}", event.summary);
            }
        }
        Err(err) => {
            info!("Failed to fetch events: {:?}", err);
            return None;
        }
    }

    let events = match hub
        .events()
        .list(calendar_id)
        .time_zone("Europe/Amsterdam")
        .time_min(from_date)
        .time_max(to_date)
        .single_events(true)
        .max_results(2500)
        .doit()
        .await
    {
        Ok(events) => events,
        Err(err) => {
            info!("Calendar ID: {}", calendar_id);
            info!("Google Calendar error: {:#?}", err);
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

async fn add_cal_to_list(
    google: &GoogleConfig,
    calendar_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let hub = get_hub(google)
        .await
        .ok_or("Failed to get hub for google calendar")?;

    let calendar_list_entry = CalendarListEntry {
        id: Some(calendar_id),
        // primary: Some(true),
        ..Default::default()
    };

    hub.calendar_list()
        .insert(calendar_list_entry)
        .doit()
        .await?;

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
            Ok(()) => {}
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
