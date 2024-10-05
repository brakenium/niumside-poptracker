use crate::census::constants::Faction;
use poise::serenity_prelude::{parse_emoji, EmojiIdentifier};

// Write a test for the `Icons` enum
mod tests {
    use super::*;

    // #[test]
    // fn test_icons_try_into() {
    //     assert_eq!(Icons::VS.try_into().unwrap(), Some(parse_emoji("<:VS:683285085818191976".to_string())));
    //     assert_eq!(Icons::NC.try_into().unwrap(), Some(parse_emoji("<:NC:683285084320694302".to_string())));
    //     assert_eq!(Icons::TR.try_into().unwrap(), Some(parse_emoji("<:TR:683285084463431720".to_string())));
    //     assert_eq!(Icons::NS.try_into().unwrap(), Some(parse_emoji("<:NS:722816749707198574".to_string())));
    //     assert_eq!(Icons::Ps2White.try_into().unwrap(), Some(parse_emoji("<:Ps2White:722814368022134790".to_string())));
    // }

    #[test]
    fn test_icons_try_from() {
        assert_eq!(Icons::try_from(Faction::VS).unwrap(), Icons::VS);
        assert_eq!(Icons::try_from(Faction::NC).unwrap(), Icons::NC);
        assert_eq!(Icons::try_from(Faction::TR).unwrap(), Icons::TR);
        assert_eq!(Icons::try_from(Faction::NS).unwrap(), Icons::NS);
        assert_eq!(Icons::try_from(Faction::Unknown).unwrap(), Icons::Ps2White);
    }

    #[test]
    fn test_icons_to_discord_emoji() {
        assert_eq!(Icons::VS.to_discord_emoji(), parse_emoji("<:VS:683285085818191976>".to_string()));
        assert_eq!(Icons::NC.to_discord_emoji(), parse_emoji("<:NC:683285084320694302>".to_string()));
        assert_eq!(Icons::TR.to_discord_emoji(), parse_emoji("<:TR:683285084463431720>".to_string()));
        assert_eq!(Icons::NS.to_discord_emoji(), parse_emoji("<:NS:722816749707198574>".to_string()));
        assert_eq!(Icons::Ps2White.to_discord_emoji(), parse_emoji("<:Ps2White:722814368022134790>".to_string()));
    }
}

#[repr(u64)]
#[derive(Debug, PartialEq)]
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
            Self::VS => parse_emoji(format!("<:VS:{}>", Self::VS as u64)),
            Self::NC => parse_emoji(format!("<:NC:{}>", Self::NC as u64)),
            Self::TR => parse_emoji(format!("<:TR:{}>", Self::TR as u64)),
            Self::NS => parse_emoji(format!("<:NS:{}>", Self::NS as u64)),
            Self::Ps2White => parse_emoji(format!("<:Ps2White:{}>", Self::Ps2White as u64)),
        }
    }
}
