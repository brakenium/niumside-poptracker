use crate::census::rest::client::{CensusCollections, CensusRequestError, CensusRequestType, CensusRequestableObject, CensusResponse, CensusRestClient};
use crate::census::structs::character::Character;
use tracing::debug;

impl CensusRequestableObject for Character {
    async fn get_by_id(client: &CensusRestClient, id: u64) -> Result<Self, CensusRequestError> {
        let mut url = client.get_request_url(
            CensusRequestType::Get,
            Self::get_collection(),
        )?;

        url.set_query(Some(&format!("character_id={id}")));

        debug!("Getting character by id: {} using url: {}", id, url);

        let response = reqwest::get(url).await?;
        let character: CensusResponse<Self> = response.json().await?;
        let return_value: Result<Self, CensusRequestError> = match character.objects.first() {
            Some(character) => Ok(character.clone()),
            None => Err(CensusRequestError::NotFound),
        };

        return_value
    }

    async fn get_by_name(client: &CensusRestClient, name: &str) -> Result<Self, CensusRequestError> {
        let mut url = client.get_request_url(
            CensusRequestType::Get,
            Self::get_collection(),
        )?;

        let name_lower = name.to_lowercase();
        url.set_query(Some(&format!("name.first_lower={name_lower}")));

        debug!("Getting character by name: {} using url: {}", name, url);

        let response = reqwest::get(url).await?;
        let character: CensusResponse<Self> = response.json().await?;
        let return_value: Result<Self, CensusRequestError> = character.objects.first()
            .map_or_else(
                || Err(CensusRequestError::NotFound),
                |character| Ok(character.clone()),
            );

        return_value
    }

    async fn update_from_rest(&mut self, client: &CensusRestClient) -> Result<(), CensusRequestError> {
        let character = Self::get_by_id(client, self.character_id).await?;
        self.name = character.name;
        self.times = character.times;
        self.faction = character.faction;

        Ok(())
    }

    fn get_collection() -> CensusCollections {
        CensusCollections::Character
    }

    fn get_name() -> &'static str {
        "character"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::census::structs::character::Character;

    const CHARACTER_ID: u64 = 5_428_830_384_575_692_145;
    const CHARACTER_NAME: &str = "brakenium";
    const CHARACTER_CREATION_TIMESTAMP: i64 = 1_549_564_351;

    #[tokio::test]
    async fn test_get_by_id() {
        let client = CensusRestClient::default();

        let character = Character::get_by_id(&client, CHARACTER_ID).await.unwrap();
        assert_eq!(character.character_id, CHARACTER_ID);
        assert_eq!(character.name.first, CHARACTER_NAME);
        assert_eq!(character.times.unwrap().creation.timestamp(), CHARACTER_CREATION_TIMESTAMP);
    }

    #[tokio::test]
    async fn test_get_by_name() {
        let client = CensusRestClient::default();

        let character = Character::get_by_name(&client, CHARACTER_NAME).await.unwrap();
        assert_eq!(character.character_id, CHARACTER_ID);
        assert_eq!(character.name.first, CHARACTER_NAME);
        assert_eq!(character.times.unwrap().creation.timestamp(), CHARACTER_CREATION_TIMESTAMP);
    }
}
