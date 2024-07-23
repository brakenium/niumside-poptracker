-- Add migration script here
BEGIN;

CREATE FUNCTION public.is_u64(value NUMERIC(20, 0)) RETURNS BOOLEAN AS $$
BEGIN
    RETURN value >= 0 AND value <= 18446744073709551615;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION public.is_i64(value NUMERIC(19, 0)) RETURNS BOOLEAN AS $$
BEGIN
    RETURN value >= -9223372036854775808 AND value <= 9223372036854775807;
END;
$$ LANGUAGE plpgsql;

CREATE TABLE public.calendar_events (
    calendar_events_id SERIAL,
    calendar_id TEXT NOT NULL,
    calendar_event_id TEXT NOT NULL,
    CONSTRAINT "PK_calendar_events" PRIMARY KEY (calendar_events_id),
    CONSTRAINT "UK_calendar_events_unique_events" UNIQUE (calendar_id, calendar_event_id)
);

CREATE TABLE public.discord_events (
    calendar_events_id SERIAL NOT NULL,
    guild_id BIGINT NOT NULL CONSTRAINT "CH_discord_events_guild_id" CHECK (is_i64(guild_id) AND is_u64(guild_id)),
    discord_id BIGINT NOT NULL CONSTRAINT "CH_discord_events_discord_id" CHECK (is_i64(guild_id) AND is_u64(discord_id)),
    CONSTRAINT "PK_discord_events" PRIMARY KEY (discord_id),
    CONSTRAINT "FK_discord_events_calendar_events" FOREIGN KEY (calendar_events_id)
        REFERENCES public.calendar_events (calendar_events_id),
    CONSTRAINT "UK_discord_events_unique_guild" UNIQUE (calendar_events_id, guild_id)
);

END;
