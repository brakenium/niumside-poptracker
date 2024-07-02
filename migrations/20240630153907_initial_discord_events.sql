-- Add migration script here
BEGIN;

CREATE TABLE public.calendar_events (
    calendar_events_id SERIAL,
    calendar_id TEXT NOT NULL,
    calendar_event_id TEXT NOT NULL,
    CONSTRAINT "PK_calendar_events" PRIMARY KEY (calendar_events_id),
    CONSTRAINT "AK_calendar_events_unique_events" UNIQUE (calendar_id, calendar_event_id)
);

CREATE TABLE public.discord_events (
    calendar_events_id SERIAL NOT NULL,
    guild_id NUMERIC(0) NOT NULL,
    discord_id NUMERIC(0) NOT NULL,
    CONSTRAINT "PK_discord_events" PRIMARY KEY (discord_id),
    CONSTRAINT "FK_discord_events_calendar_events" FOREIGN KEY (calendar_events_id)
        REFERENCES public.calendar_events (calendar_events_id)
);

END;
