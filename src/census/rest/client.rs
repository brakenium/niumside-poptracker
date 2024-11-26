use crate::census::CENSUS_URL;
use crate::storage::configuration::CensusConfig;
use rocket::serde::{Deserialize, Serialize};
use strum::Display;
use url::{form_urlencoded, Url};

pub struct CensusRestClient {
    pub(crate) census_url: Url,
    // lithafalcon_url: Url,
    service_id: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct CensusResponse<T: CensusRequestableObject> {
    pub returned: usize,
    #[serde(alias = "character_list")]
    pub objects: Vec<T>,
}

#[derive(thiserror::Error, Debug, Display)]
pub enum CensusRequestError {
    ReqwestError(#[from] reqwest::Error),
    SerdeError(#[from] serde_json::Error),
    ParseError(#[from] url::ParseError),
    NotFound,
}

#[allow(dead_code)]
pub enum CensusRequestType {
    Get,
    Count,
}

impl Into<&str> for CensusRequestType {
    fn into(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Count => "count",
        }
    }
}

pub enum CensusNamespaces {
    Ps2V2,
}

impl Into<&str> for CensusNamespaces {
    fn into(self) -> &'static str {
        match self {
            Self::Ps2V2 => "ps2:v2",
        }
    }
}

pub enum CensusCollections {
    Character,
}

impl Into<&str> for CensusCollections {
    fn into(self) -> &'static str {
        match self {
            Self::Character => "character",
        }
    }
}

pub trait CensusRequestableObject: Sized {
    async fn get_by_id(client: &CensusRestClient, id: usize) -> Result<Self, CensusRequestError>;
    async fn get_by_name(client: &CensusRestClient, name: &str) -> Result<Self, CensusRequestError>;
    fn get_collection() -> CensusCollections;
    fn get_name() -> &'static str;
}

impl From<CensusConfig> for CensusRestClient {
    fn from(config: CensusConfig) -> Self {
        Self {
            census_url: config.census_base_url,
            service_id: config.service_id,
        }
    }
}

impl Default for CensusRestClient {
    fn default() -> Self {
        Self {
            census_url: CENSUS_URL.clone(),
            service_id: "s:example".to_string(),
        }
    }
}

impl CensusRestClient {
    pub fn new(census_url: Url, service_id: String) -> Self {
        Self {
            census_url,
            service_id,
        }
    }

    pub fn get_request_url(&self, request_type: CensusRequestType, collection: CensusCollections) -> Result<Url, CensusRequestError> {
        let request_type: &str = Into::<&str>::into(request_type);

        let census_namespace: String = form_urlencoded::byte_serialize(
            Into::<&str>::into(CensusNamespaces::Ps2V2).as_bytes()
        ).collect();
        let url = self.census_url
            .join(&format!("{request_type}/"))?
            .join(&format!("{census_namespace}/"))?
            .join(collection.into())?;

        Ok(url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_census_rest_client() {
        let client = CensusRestClient::default();
        let url = client.get_request_url(CensusRequestType::Get, CensusCollections::Character).unwrap();
        assert_eq!(url.as_str(), "https://census.daybreakgames.com/get/ps2%3Av2/character");
    }
}
