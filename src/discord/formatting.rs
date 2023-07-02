use poise::serenity_prelude::{CreateEmbed, Embed, Mentionable};
use auraxis::WorldID;
use crate::controllers::population::{WorldBreakdown, ZoneBreakdown};
use crate::discord::icons::Icons;

pub fn world_breakdown_message(
    world_breakdown: &WorldBreakdown,
) -> Vec<CreateEmbed> {
    let mut embeds = Vec::new();

    for world in world_breakdown {
        embeds.push(single_world_breakdown_embed(world.0, world.1));
    }

    embeds
}

pub fn single_world_breakdown_embed(
    world: &WorldID,
    zone: &ZoneBreakdown,
) -> CreateEmbed {
    let mut world_embed = CreateEmbed::default();
    let mut world_population = 0;
    for (zone_id, faction_breakdown) in zone {
        let mut zone_population = 0;
        let mut zone_faction_pop = String::new();
        for (faction_id, team_breakdown) in faction_breakdown.iter() {
            let mut faction_population = 0;
            for (team_id, loadout_breakdown) in team_breakdown.iter() {
                for (_, loadout_population) in loadout_breakdown.iter() {
                    world_population += loadout_population;
                    zone_population += loadout_population;
                    faction_population += loadout_population;
                }
            }
            let faction_emoji = Icons::try_from(*faction_id)
                .unwrap_or(Icons::Ps2White)
                .to_discord_emoji()
                .mention();
            zone_faction_pop.push_str(&format!("{faction_emoji}: {faction_population}\n"));
        }
        world_embed.field(
            format!("Zone: {zone_id}"),
            format!("Active players:\n{zone_faction_pop}"),
            true,
        );
    }
    world_embed.field(
        format!("World: {world}"),
        format!("Active players: {world_population}"),
        false,
    );
    world_embed
}