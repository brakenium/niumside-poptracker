use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, VariantNames, FromRepr};
use niumside_poptracker::FromStr;

#[repr(u16)]
#[derive(
    FromStr,
    Serialize,
    Deserialize,
    Copy,
    Clone,
    Eq,
    Debug,
    PartialEq,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
    EnumIter,
    VariantNames,
    strum::Display,
    FromRepr,
)]
#[allow(clippy::upper_case_acronyms)]
pub enum Loadout {
    Unknown = 0,
    NCInfiltrator = 1,
    NCLightAssault = 3,
    NCMedic = 4,
    NCEngineer = 5,
    NCHeavyAssault = 6,
    NCMAX = 7,
    TRInfiltrator = 8,
    TRLightAssault = 10,
    TRMedic = 11,
    TREngineer = 12,
    TRHeavyAssault = 13,
    TRMAX = 14,
    VSInfiltrator = 15,
    VSLightAssault = 17,
    VSMedic = 18,
    VSEngineer = 19,
    VSHeavyAssault = 20,
    VSMAX = 21,
    NSInfiltrator = 28,
    NSLightAssault = 29,
    NSMedic = 30,
    NSEngineer = 31,
    NSHeavyAssault = 32,
    NSMAX = 45,
}

impl Loadout {
    pub const fn get_faction(self) -> Faction {
        match self {
            Self::Unknown => Faction::Unknown,
            Self::NCInfiltrator
            | Self::NCLightAssault
            | Self::NCMedic
            | Self::NCEngineer
            | Self::NCHeavyAssault
            | Self::NCMAX => Faction::NC,
            Self::TRInfiltrator
            | Self::TRLightAssault
            | Self::TRMedic
            | Self::TREngineer
            | Self::TRHeavyAssault
            | Self::TRMAX => Faction::TR,
            Self::VSInfiltrator
            | Self::VSLightAssault
            | Self::VSMedic
            | Self::VSEngineer
            | Self::VSHeavyAssault
            | Self::VSMAX => Faction::VS,
            Self::NSInfiltrator
            | Self::NSLightAssault
            | Self::NSMedic
            | Self::NSEngineer
            | Self::NSHeavyAssault
            | Self::NSMAX => Faction::NS,
        }
    }
}

#[repr(u16)]
#[derive(
    FromStr,
    Serialize,
    Deserialize,
    Copy,
    Clone,
    Eq,
    Debug,
    PartialEq,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
    EnumIter,
    VariantNames,
    strum::Display,
    FromRepr,
)]
pub enum Faction {
    Unknown = 0,
    VS = 1,
    NC = 2,
    TR = 3,
    NS = 4,
}

pub type TeamID = Faction;

#[repr(u16)]
#[derive(
    FromStr,
    Serialize,
    Deserialize,
    Copy,
    Clone,
    Eq,
    Debug,
    PartialEq,
    Hash,
    TryFromPrimitive,
    IntoPrimitive,
    EnumIter,
    VariantNames,
    strum::Display,
    FromRepr,
)]
#[strum(ascii_case_insensitive)]
pub enum WorldID {
    Jaeger = 19,
    Briggs = 25,
    Miller = 10,
    Cobalt = 13,
    Connery = 1,
    Emerald = 17,
    Soltech = 40,
}

pub type CharacterID = u64;
pub type OutfitID = u64;

#[derive(FromStr, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZoneID(pub u32);

#[derive(FromStr, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefinitionID(pub u16);
#[derive(FromStr, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceID(pub u16);
pub type FacilityID = u32;
pub type ExperienceID = u16;
pub type VehicleID = u16;
pub type WeaponID = u32;
pub type FiremodeID = u32;

impl From<ZoneID> for InstanceID {
    fn from(zone_id: ZoneID) -> Self {
        InstanceID(((zone_id.0 & 0xFFFF_0000) >> 16) as u16)
    }
}

impl From<ZoneID> for DefinitionID {
    fn from(zone_id: ZoneID) -> Self {
        DefinitionID((zone_id.0 & 0xFFFF) as u16)
    }
}
