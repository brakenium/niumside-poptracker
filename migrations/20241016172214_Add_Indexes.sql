-- Add migration script here
CREATE INDEX idx_world_population_population_id ON world_population (population_id);

CREATE INDEX idx_zone_population_world_population_id ON zone_population (world_population_id);

CREATE INDEX idx_team_population_zone_population_id ON team_population (zone_population_id);

CREATE INDEX idx_loadout_population_team_population_id ON loadout_population (team_population_id);
