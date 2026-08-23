use crate::discord::updaters::utils::{
    create_or_edit_event, get_message_or_create_new, ToScheduleEventFields,
};
use crate::discord::updaters::Updater;
use crate::discord::{formatting, Data};
use crate::google_calendar::formatting::html_to_md;
use crate::google_calendar::get_calendar_color;
use crate::storage::configuration::{DataSource, DiscordCalendarConfig, GoogleConfig};
use crate::{discord, google_calendar};
use calendar3::api::{Event, Events};
use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{Colour, CreateEmbed, EditMessage, User};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tracing::error;
use url::Url;

pub const NEWS_BASE_URL: &str = "https://www.planetside2.com";

pub struct PlanetSideNews;

#[derive(Serialize, Deserialize)]
struct PlanetSideNewsItem {
    name: String,
    start_date_epoch: DateTime<Utc>,
    title: String,
    summary: String,
    thumbnail: Url,
    #[serde(rename = "pageName")]
    page_name: String,
    #[serde(rename = "shareUrl")]
    share_url: Url,
}

#[derive(Serialize, Deserialize)]
struct PlanetsideNewsFeed {
    data: PlanetSideNewsData,
    options: PlanetsideNewsOptions,
}

#[derive(Serialize, Deserialize)]
struct PlanetsideNewsOptions {
    r#type: String,
    #[serde(rename = "defaultImage")]
    default_image: Url,
}

#[derive(Serialize, Deserialize)]
struct PlanetSideNewsData {
    list: Vec<PlanetSideNewsItem>,
}

pub async fn check_sent_news_items(news_items: &PlanetSideNewsData) -> Result<Vec<PlanetSideNewsItem>, discord::Error> {

}

pub async fn update_single_news_source(ctx: &serenity::Context, news_source: &DataSource) -> Result<(), discord::Error> {
    let current_news_items = reqwest::get(news_source.url.clone())
        .await?
        .json::<PlanetsideNewsFeed>()
        .await?;

    let news_channels =

    Ok(())
}

impl Updater for PlanetSideNews {
    async fn update(ctx: &serenity::Context, data: &Data) -> Result<(), discord::Error> {
        for news_source in &data.planetside_news.sources {
            let result = update_single_news_source(ctx, news_source).await;

            if let Err(error) = result {
                error!("Failed to update news: {:?}", error);
            }
        }

        Ok(())
    }
}
