-- Add migration script here
CREATE INDEX idx_world_population_world_id ON world_population (world_id);

CREATE INDEX idx_zone_population_zone_id ON zone_population (zone_id);

CREATE INDEX idx_team_population_team_id ON team_population (team_id);

CREATE INDEX idx_loadout_population_loadout_id ON loadout_population (loadout_id);
