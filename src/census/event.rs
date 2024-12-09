use crate::census::constants::{
    CharacterID, ExperienceID, FacilityID, Faction, FiremodeID, Loadout, OutfitID, VehicleID,
    WeaponID, WorldID, ZoneID,
};
use crate::census::utils::deserialize_from_str_custom_impl;
use crate::census::utils::{
    de_bool_from_str_int, deserialize_duration_from_str, deserialize_from_str, serialize_duration,
};
use std::fmt::{Display, Formatter};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeAs, SerializeAs, TimestampMilliSeconds, TimestampSeconds};

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
#[allow(dead_code)]
pub enum EventNames {
    AchievementEarned,
    BattleRankUp,
    Death,
    ItemAdded,
    SkillAdded,
    VehicleDestroy,
    GainExperience,
    GainExperienceId(ExperienceID),
    PlayerFacilityCapture,
    PlayerFacilityDefend,
    ContinentLock,
    ContinentUnlock,
    FacilityControl,
    MetagameEvent,
    PlayerLogin,
    PlayerLogout,
}

impl serde::Serialize for EventNames {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use self::EventNames::{
            AchievementEarned, BattleRankUp, ContinentLock, ContinentUnlock, Death,
            FacilityControl, GainExperience, GainExperienceId, ItemAdded, MetagameEvent,
            PlayerFacilityCapture, PlayerFacilityDefend, PlayerLogin, PlayerLogout, SkillAdded,
            VehicleDestroy,
        };

        match *self {
            AchievementEarned => {
                serializer.serialize_unit_variant("EventNames", 0, "AchievementEarned")
            }
            BattleRankUp => serializer.serialize_unit_variant("EventNames", 1, "BattleRankUp"),
            Death => serializer.serialize_unit_variant("EventNames", 2, "Death"),
            ItemAdded => serializer.serialize_unit_variant("EventNames", 3, "ItemAdded"),
            SkillAdded => serializer.serialize_unit_variant("EventNames", 4, "SkillAdded"),
            VehicleDestroy => serializer.serialize_unit_variant("EventNames", 5, "VehicleDestroy"),
            GainExperience => serializer.serialize_unit_variant("EventNames", 6, "GainExperience"),
            GainExperienceId(value) => {
                let event_name = format!("GainExperience_experience_id_{value}");

                serializer.serialize_str(&event_name)
            }
            PlayerFacilityCapture => {
                serializer.serialize_unit_variant("EventNames", 8, "PlayerFacilityCapture")
            }
            PlayerFacilityDefend => {
                serializer.serialize_unit_variant("EventNames", 9, "PlayerFacilityDefend")
            }
            ContinentLock => serializer.serialize_unit_variant("EventNames", 10, "ContinentLock"),
            ContinentUnlock => {
                serializer.serialize_unit_variant("EventNames", 11, "ContinentUnlock")
            }
            FacilityControl => {
                serializer.serialize_unit_variant("EventNames", 12, "FacilityControl")
            }
            MetagameEvent => serializer.serialize_unit_variant("EventNames", 13, "MetagameEvent"),
            PlayerLogin => serializer.serialize_unit_variant("EventNames", 14, "PlayerLogin"),
            PlayerLogout => serializer.serialize_unit_variant("EventNames", 15, "PlayerLogout"),
        }
    }
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
#[serde(tag = "event_name")]
#[allow(clippy::enum_variant_names)]
pub enum Event {
    PlayerLogin(PlayerLogin),
    PlayerLogout(PlayerLogout),
    Death(Death),
    VehicleDestroy(VehicleDestroy),
    GainExperience(GainExperience),
    PlayerFacilityCapture(PlayerFacilityCapture),
    PlayerFacilityDefend(PlayerFacilityDefend),
    ContinentLock(ContinentLock),
    ContinentUnlock(ContinentUnlock),
    FacilityControl(FacilityControl),
    MetagameEvent(MetagameEvent),
    ItemAdded,
    AchievementEarned,
    SkillAdded,
    BattleRankUp,
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayerLogin(_) => {
                write!(f, "PlayerLogin")
            }
            Self::PlayerLogout(_) => {
                write!(f, "PlayerLogout")
            }
            Self::Death(_) => {
                write!(f, "Death")
            }
            Self::VehicleDestroy(_) => {
                write!(f, "VehicleDestroy")
            }
            Self::GainExperience(_) => {
                write!(f, "GainExperience")
            }
            Self::PlayerFacilityCapture(_) => {
                write!(f, "PlayerFacilityCapture")
            }
            Self::PlayerFacilityDefend(_) => {
                write!(f, "PlayerFacilityDefend")
            }
            Self::ContinentLock(_) => {
                write!(f, "ContinentLock")
            }
            Self::ContinentUnlock(_) => {
                write!(f, "ContinentUnlock")
            }
            Self::FacilityControl(_) => {
                write!(f, "FacilityControl")
            }
            Self::MetagameEvent(_) => {
                write!(f, "MetagameEvent")
            }
            Self::ItemAdded => {
                write!(f, "ItemAdded")
            }
            Self::AchievementEarned => {
                write!(f, "AchievementEarned")
            }
            Self::SkillAdded => {
                write!(f, "SkillAdded")
            }
            Self::BattleRankUp => {
                write!(f, "BattleRankUp")
            }
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct PlayerLogin {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct PlayerLogout {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct Death {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub attacker_character_id: CharacterID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub attacker_fire_mode_id: FiremodeID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub attacker_loadout_id: Loadout,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub attacker_vehicle_id: VehicleID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub attacker_weapon_id: WeaponID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_loadout_id: Loadout,
    #[serde(deserialize_with = "de_bool_from_str_int")]
    pub is_headshot: bool,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(default, deserialize_with = "deserialize_from_str")]
    pub vehicle_id: VehicleID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub zone_id: ZoneID,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct VehicleDestroy {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub attacker_character_id: CharacterID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub attacker_loadout_id: Loadout,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub attacker_vehicle_id: VehicleID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub attacker_weapon_id: WeaponID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub facility_id: FacilityID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub faction_id: Faction,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub vehicle_id: VehicleID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub zone_id: ZoneID,
}

// Write it a test where it will parse the following string:
// "{"payload":{"amount":"28","character_id":"5429573939285739921","event_name":"GainExperience","experience_id":"140","loadout_id":"20","other_id":"34360508066","team_id":"1","timestamp":"1728117291","world_id":"13","zone_id":"8"},"service":"event","type":"serviceMessage"}"
// And check if it is parsed correctly.
// You can use the following code
mod tests {
    #[cfg(test)]
    use crate::census::constants::{Faction, Loadout, WorldID, ZoneID};
    #[cfg(test)]
    use crate::census::event::{Event, GainExperience};
    #[cfg(test)]
    use crate::census::CensusMessage;
    #[cfg(test)]
    use chrono::DateTime;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_gain_experience_deserialization() {
        let json_str = r#"{"payload":{"amount":"28","character_id":"5429573939285739921","event_name":"GainExperience","experience_id":"140","loadout_id":"20","other_id":"34360508066","team_id":"1","timestamp":"1728117291","world_id":"13","zone_id":"8"},"service":"event","type":"serviceMessage"}"#;
        let json_value = serde_json::from_str(json_str).unwrap();
        let event: CensusMessage = serde_json::from_value(json_value).unwrap();

        if let CensusMessage::ServiceMessage { payload, .. } = event {
            match payload {
                Event::GainExperience(GainExperience {
                    character_id,
                    experience_id,
                    loadout_id,
                    other_id,
                    timestamp,
                    world_id,
                    zone_id,
                    amount,
                    team_id,
                }) => {
                    assert_eq!(character_id, 5_429_573_939_285_739_921);
                    assert_eq!(experience_id, 140);
                    assert_eq!(loadout_id, Loadout::VSHeavyAssault);
                    assert_eq!(other_id, 34_360_508_066);
                    assert_eq!(
                        timestamp,
                        DateTime::from_timestamp(1_728_117_291, 0).unwrap()
                    );
                    assert_eq!(world_id, WorldID::Cobalt);
                    assert_eq!(zone_id, ZoneID(8));
                    assert_eq!(amount, 28);
                    assert_eq!(team_id, Faction::VS);
                }
                _ => panic!("Unexpected event type"),
            }
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct GainExperience {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub experience_id: ExperienceID,
    #[serde(deserialize_with = "deserialize_from_str_custom_impl")]
    pub loadout_id: Loadout,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub other_id: CharacterID,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub zone_id: ZoneID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub amount: u16,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub team_id: Faction,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct PlayerFacilityCapture {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub facility_id: FacilityID,
    pub outfit_id: OutfitID,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub zone_id: ZoneID,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct PlayerFacilityDefend {
    #[serde(deserialize_with = "deserialize_from_str")]
    pub character_id: CharacterID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub facility_id: FacilityID,
    pub outfit_id: OutfitID,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub zone_id: ZoneID,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct FacilityControl {
    #[serde(
        deserialize_with = "deserialize_duration_from_str",
        serialize_with = "serialize_duration"
    )]
    pub duration_held: Duration,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub facility_id: FacilityID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub new_faction_id: Faction,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub old_faction_id: Faction,
    pub outfit_id: OutfitID,
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub zone_id: ZoneID,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ContinentLock {
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub zone_id: ZoneID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub triggering_faction: Faction,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub previous_faction: Faction,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub vs_population: u16,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub nc_population: u16,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub tr_population: u16,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub metagame_event_id: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ContinentUnlock {
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub zone_id: ZoneID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub triggering_faction: Faction,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub previous_faction: Faction,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub vs_population: u16,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub nc_population: u16,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub tr_population: u16,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub metagame_event_id: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub struct MetagameEvent {
    #[serde(
        deserialize_with = "TimestampSeconds::<String>::deserialize_as",
        serialize_with = "TimestampMilliSeconds::<i64>::serialize_as"
    )]
    pub timestamp: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub world_id: WorldID,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub instance_id: u32,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub experience_bonus: f32,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub faction_nc: f32,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub faction_tr: f32,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub faction_vs: f32,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub metagame_event_id: u8,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub metagame_event_state: u8,
    pub metagame_event_state_name: String,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub zone_id: ZoneID,
}
