use crate::census::constants::CharacterID;
use crate::census::utils::deserialize_from_str;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeAs, SerializeAs, TimestampMilliSeconds, TimestampSeconds};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct Character {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    pub name: CharacterName,
    pub times: CharacterTimes,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct CharacterName {
    pub first: String,
    pub first_lower: String,
}

// #[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct CharacterTimes {
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    // #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub creation: DateTime<Utc>,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub last_login: DateTime<Utc>,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub last_save: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub login_count: usize,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub minutes_played: usize,
}
