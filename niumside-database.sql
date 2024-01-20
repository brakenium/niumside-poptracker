BEGIN;

CREATE TABLE IF NOT EXISTS public.population
(
    population_id serial,
    "timestamp" timestamp without time zone NOT NULL DEFAULT NOW(),
    CONSTRAINT "PK_population" PRIMARY KEY (population_id),
    CONSTRAINT "AK_Unique_Timestamp" UNIQUE ("timestamp")
    );

CREATE TABLE IF NOT EXISTS public.world_population
(
    world_id integer NOT NULL,
    population_id integer NOT NULL,
    world_population_id serial,
    CONSTRAINT "PK_world_population" PRIMARY KEY (world_population_id),
    CONSTRAINT "AK_UQ_population_world" UNIQUE (world_id, population_id)
    );

CREATE TABLE IF NOT EXISTS public.zone_population
(
    zone_population_id serial,
    zone_id integer NOT NULL,
    world_population_id serial NOT NULL,
    CONSTRAINT "PK_zone_population" PRIMARY KEY (zone_population_id),
    CONSTRAINT "AK_UQ_population_zone" UNIQUE (zone_id, world_population_id)
    );

CREATE TABLE IF NOT EXISTS public.team_population
(
    team_population_id serial,
    zone_population_id serial NOT NULL,
    team_id smallint NOT NULL,
    CONSTRAINT "PK_team_population" PRIMARY KEY (team_population_id),
    CONSTRAINT "AK_UQ_team_zone" UNIQUE (zone_population_id, team_id)
    );

CREATE TABLE IF NOT EXISTS public.loadout_population
(
    loadout_population_id serial,
    loadout_id smallint NOT NULL,
    team_population_id serial NOT NULL,
    amount smallint NOT NULL,
    CONSTRAINT "PK_loadout_population" PRIMARY KEY (loadout_population_id),
    CONSTRAINT "AK_UQ_loadout_team" UNIQUE (team_population_id, loadout_id)
    );

CREATE TABLE IF NOT EXISTS public.outfit
(
    outfit_id integer NOT NULL,
    name character varying,
    last_fetch timestamp without time zone,
    PRIMARY KEY (outfit_id)
    );

CREATE TABLE IF NOT EXISTS public.character_session
(
    character_id integer NOT NULL,
    outfit_id integer,
    session_start timestamp without time zone NOT NULL,
    session_end timestamp without time zone,
    character_session_id integer NOT NULL,
    PRIMARY KEY (character_session_id),
    CONSTRAINT "AK_UQ_Session" UNIQUE (character_id, session_start)
    );

CREATE TABLE IF NOT EXISTS public."character"
(
    character_id integer NOT NULL,
    name character varying,
    last_update timestamp without time zone,
    PRIMARY KEY (character_id)
    );

CREATE TABLE IF NOT EXISTS public.faction
(
    faction_id smallint NOT NULL,
    name character varying,
    description character varying,
    last_update timestamp without time zone,
    CONSTRAINT "PK_faction" PRIMARY KEY (faction_id)
    );

CREATE TABLE IF NOT EXISTS public.zone
(
    zone_id integer NOT NULL,
    name character varying,
    description character varying,
    last_update timestamp without time zone,
    CONSTRAINT "PK_zone" PRIMARY KEY (zone_id)
    );

CREATE TABLE IF NOT EXISTS public.world
(
    world_id integer NOT NULL,
    name character varying,
    description character varying,
    last_update timestamp without time zone,
    CONSTRAINT "PK_world" PRIMARY KEY (world_id)
    );

CREATE TABLE IF NOT EXISTS public.loadout
(
    loadout_id smallint NOT NULL,
    name character varying,
    description character varying,
    last_update timestamp without time zone,
    CONSTRAINT "PK_loadout" PRIMARY KEY (loadout_id)
    );

ALTER TABLE IF EXISTS public.world_population
    ADD CONSTRAINT "FK_world_population_population" FOREIGN KEY (population_id)
    REFERENCES public.population (population_id) MATCH SIMPLE
    ON UPDATE NO ACTION
       ON DELETE NO ACTION
		NOT VALID;

ALTER TABLE IF EXISTS public.world_population
    ADD CONSTRAINT "FK_world_population_world" FOREIGN KEY (world_id)
    REFERENCES public.world (world_id) MATCH SIMPLE
    ON UPDATE NO ACTION
       ON DELETE NO ACTION
        NOT VALID;


ALTER TABLE IF EXISTS public.zone_population
    ADD CONSTRAINT "FK_zone_population_world_population" FOREIGN KEY (world_population_id)
    REFERENCES public.world_population (world_population_id) MATCH SIMPLE
    ON UPDATE NO ACTION
       ON DELETE NO ACTION
        NOT VALID;


ALTER TABLE IF EXISTS public.zone_population
    ADD CONSTRAINT "FK_zone_population_zone" FOREIGN KEY (zone_id)
    REFERENCES public.zone (zone_id) MATCH SIMPLE
    ON UPDATE RESTRICT
       ON DELETE RESTRICT
        NOT VALID;


ALTER TABLE IF EXISTS public.team_population
    ADD CONSTRAINT "FK_team_zone_population" FOREIGN KEY (zone_population_id)
    REFERENCES public.zone_population (zone_population_id) MATCH SIMPLE
    ON UPDATE RESTRICT
       ON DELETE RESTRICT
        NOT VALID;

ALTER TABLE IF EXISTS public.team_population
    ADD CONSTRAINT "FK_team_population_team" FOREIGN KEY (team_id)
    REFERENCES public.faction (faction_id) MATCH SIMPLE
    ON UPDATE RESTRICT
       ON DELETE RESTRICT
        NOT VALID;


ALTER TABLE IF EXISTS public.loadout_population
    ADD CONSTRAINT "FK_loadout_team" FOREIGN KEY (team_population_id)
    REFERENCES public.team_population (team_population_id) MATCH SIMPLE
    ON UPDATE RESTRICT
       ON DELETE RESTRICT
        NOT VALID;


ALTER TABLE IF EXISTS public.loadout_population
    ADD CONSTRAINT "FK_loadout_population_loadout" FOREIGN KEY (loadout_id)
    REFERENCES public.loadout (loadout_id) MATCH SIMPLE
    ON UPDATE NO ACTION
       ON DELETE NO ACTION
        NOT VALID;


ALTER TABLE IF EXISTS public.character_session
    ADD CONSTRAINT "FK_character_session_outfit" FOREIGN KEY (outfit_id)
    REFERENCES public.outfit (outfit_id) MATCH SIMPLE
    ON UPDATE RESTRICT
       ON DELETE RESTRICT
        NOT VALID;


ALTER TABLE IF EXISTS public.character_session
    ADD CONSTRAINT "FK_character_session_character" FOREIGN KEY (character_id)
    REFERENCES public."character" (character_id) MATCH SIMPLE
    ON UPDATE RESTRICT
       ON DELETE RESTRICT
        NOT VALID;

END;