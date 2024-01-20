use crate::census::constants::Faction;
use poise::serenity_prelude::{parse_emoji, EmojiId, EmojiIdentifier};

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

impl TryInto<Option<EmojiIdentifier>> for Icons {
    type Error = ();

    fn try_into(self) -> Result<Option<EmojiIdentifier>, Self::Error> {
        Ok(self.to_discord_emoji())
    }
}

impl Icons {
    pub fn to_discord_emoji(&self) -> Option<EmojiIdentifier> {
        match self {
            Self::VS => parse_emoji(format!("<:VS:{}", Self::VS as u64)),
            Self::NC => parse_emoji(format!("<:NC:{}", Self::NC as u64)),
            Self::TR => parse_emoji(format!("<:TR:{}", Self::TR as u64)),
            Self::NS => parse_emoji(format!("<:NS:{}", Self::NS as u64)),
            Self::Ps2White => parse_emoji(format!("<:Ps2White:{}", Self::Ps2White as u64)),
        }
    }
}
