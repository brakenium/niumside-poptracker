use crate::census::rest::client::{CensusCollections, CensusRequestError, CensusRequestType, CensusRequestableObject, CensusResponse, CensusRestClient};
use crate::census::structs::character::Character;

impl CensusRequestableObject for Character {
    async fn get_by_id(client: &CensusRestClient, id: usize) -> Result<Self, CensusRequestError> {
        let mut url = client.get_request_url(
            CensusRequestType::Get,
            Character::get_collection(),
        )?;

        println!("Getting character by ID: {} using url: {}", id, url);

        url.set_query(Some(&format!("character_id={id}")));

        println!("Getting character by ID: {} using url: {}", id, url);

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
            Character::get_collection(),
        )?;

        let name_lower = name.to_lowercase();
        url.set_query(Some(&format!("name.first_lower={name_lower}")));

        println!("Getting character by name: {} using url: {}", name, url);

        let response = reqwest::get(url).await?;
        let character: CensusResponse<Self> = response.json().await?;
        let return_value: Result<Self, CensusRequestError> = match character.objects.first() {
            Some(character) => Ok(character.clone()),
            None => Err(CensusRequestError::NotFound),
        };

        return_value
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

        let character = Character::get_by_id(&client, CHARACTER_ID as usize).await.unwrap();
        assert_eq!(character.character_id, CHARACTER_ID);
        assert_eq!(character.name.first, CHARACTER_NAME);
        assert_eq!(character.times.creation.timestamp(), CHARACTER_CREATION_TIMESTAMP);
    }

    #[tokio::test]
    async fn test_get_by_name() {
        let client = CensusRestClient::default();

        let character = Character::get_by_name(&client, CHARACTER_NAME).await.unwrap();
        assert_eq!(character.character_id, CHARACTER_ID);
        assert_eq!(character.name.first, CHARACTER_NAME);
        assert_eq!(character.times.creation.timestamp(), CHARACTER_CREATION_TIMESTAMP);
    }
}
