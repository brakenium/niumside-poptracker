use crate::census::constants::Faction;
use crate::controllers::population::PopWorld;
use crate::discord::icons::Icons;
use poise::serenity_prelude::CreateEmbed;

pub fn world_breakdown_message(world_breakdown: &Vec<PopWorld>) -> Vec<CreateEmbed> {
    let mut embeds = Vec::new();

    for world in world_breakdown {
        embeds.push(single_world_breakdown_embed(world));
    }

    embeds
}

pub fn single_world_breakdown_embed(world: &PopWorld) -> CreateEmbed {
    let mut embed = CreateEmbed::default();

    embed.title(format!("{} Population", world.world_id));
    embed.thumbnail("https://www.planetside2.com/images/ps2-logo.png".to_string());
    embed.footer(|f| f.text(format!("Last updated: {}", world.timestamp)));

    for zone in &world.zones {
        let mut breakdown = String::new();
        for team in &zone.teams {
            let team_icon =
                Icons::try_from(Faction::try_from(team.team_id).unwrap_or(Faction::Unknown))
                    .unwrap_or(Icons::Ps2White)
                    .to_discord_emoji();
            breakdown = format!("{}{:?}: {}\n", breakdown, team_icon, team.team_population);
        }
    }

    embed
}
