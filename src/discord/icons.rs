use crate::census::constants::Faction;
use poise::serenity_prelude::{EmojiId, EmojiIdentifier};

#[repr(u64)]
pub enum Icons {
    VS = 683_285_085_818_191_976,
    NC = 683_285_084_320_694_302,
    TR = 683_285_084_463_431_720,
    NS = 722_816_749_707_198_574,
    Ps2White = 722_814_368_022_134_790,
}

impl TryFrom<Faction> for Icons {
    type Error = ();

    fn try_from(faction: Faction) -> Result<Self, Self::Error> {
        match faction {
            Faction::VS => Ok(Self::VS),
            Faction::NC => Ok(Self::NC),
            Faction::TR => Ok(Self::TR),
            Faction::NS => Ok(Self::NS),
            Faction::Unknown => Ok(Self::Ps2White),
        }
    }
}

impl TryInto<EmojiIdentifier> for Icons {
    type Error = ();

    fn try_into(self) -> Result<EmojiIdentifier, Self::Error> {
        Ok(self.to_discord_emoji())
    }
}

impl Icons {
    pub fn to_discord_emoji(&self) -> EmojiIdentifier {
        match self {
            Self::VS => EmojiIdentifier {
                id: EmojiId::from(Self::VS as u64),
                animated: false,
                name: "VS".to_string(),
            },
            Self::NC => EmojiIdentifier {
                id: EmojiId::from(Self::NC as u64),
                animated: false,
                name: "NC".to_string(),
            },
            Self::TR => EmojiIdentifier {
                id: EmojiId::from(Self::TR as u64),
                animated: false,
                name: "TR".to_string(),
            },
            Self::NS => EmojiIdentifier {
                id: EmojiId::from(Self::NS as u64),
                animated: false,
                name: "NS".to_string(),
            },
            Self::Ps2White => EmojiIdentifier {
                id: EmojiId::from(Self::Ps2White as u64),
                animated: false,
                name: "Ps2White".to_string(),
            },
        }
    }
}
