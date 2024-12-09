use crate::census::constants::{CharacterID, Faction};
use crate::census::utils::deserialize_from_str;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeAs, SerializeAs, TimestampMilliSeconds, TimestampSeconds};
#[allow(clippy::struct_field_names)]
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct Character {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    pub name: CharacterName,
    pub times: Option<CharacterTimes>,
    pub membership_reminder: Option<MembershipReminderStatus>,
    #[serde(deserialize_with = "deserialize_from_str")]
    #[serde(rename = "faction_id")]
    pub faction: Faction,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct MembershipReminderStatus {
    pub enabled: bool,
    pub last_reminder: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct CharacterName {
    pub first: String,
    pub first_lower: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct CharacterTimes {
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
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
    pub login_count: u64,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub minutes_played: u64,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            character_id: CharacterID::default(),
            name: CharacterName {
                first: String::new(),
                first_lower: String::new(),
            },
            times: None,
            membership_reminder: None,
            faction: Faction::Unknown,
        }
    }
}

impl Character {
    pub fn new(character_id: CharacterID) -> Self {
        Self {
            character_id,
            ..Default::default()
        }
    }
}
