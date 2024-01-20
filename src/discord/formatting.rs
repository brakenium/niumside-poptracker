use poise::serenity_prelude::{CreateEmbed, Mentionable};
use crate::census::constants::{Loadout, WorldID, ZoneID, Faction, CharacterID};
use crate::controllers::population::{PopWorld, WorldBreakdown, ZoneBreakdown};
use crate::discord::icons::Icons;

pub fn world_breakdown_message(
    world_breakdown: &Vec<PopWorld>,
) -> Vec<CreateEmbed> {
    let mut embeds = Vec::new();

    for world in world_breakdown {
        embeds.push(single_world_breakdown_embed(world));
    }

    embeds
}

pub fn single_world_breakdown_embed(
    world: &PopWorld,
) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    embed.title(format!("{} Population", world.world_id));
    embed.thumbnail(format!("https://www.planetside2.com/images/ps2-logo.png"));
    embed.footer(|f| {
        f.text(format!("Last updated: {}", world.timestamp))
    });

    for zone in world.zones.iter() {
        let mut breakdown = "".into();
        for team in zone.teams.iter() {
            let team_icon = Icons::try_from(
                Faction::try_from(team.team_id)
                    .unwrap_or(Faction::Unknown)
            )
                .unwrap_or(Icons::Ps2White)
                .to_discord_emoji();
            breakdown = format!("{}{:?}: {}\n", breakdown, team_icon, team.team_population);
        }
    }

    embed
}