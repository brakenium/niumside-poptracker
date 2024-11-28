-- Add migration script here
BEGIN;

CREATE TABLE IF NOT EXISTS public.users
(
    user_id    SERIAL NOT NULL,
    discord_id BIGINT
        CONSTRAINT "CH_users_discord_id" CHECK (is_i64(discord_id) AND is_u64(discord_id)),
    CONSTRAINT "PK_users" PRIMARY KEY (user_id),
    CONSTRAINT "UK_users_discord_id" UNIQUE (discord_id)
);

CREATE TABLE IF NOT EXISTS public.planetside_characters
(
    character_id             BIGINT   NOT NULL,
    user_id                  SERIAL   NOT NULL,
    name                     TEXT     NOT NULL,
    faction_id               SMALLINT NOT NULL,
    membership_reminder      BOOLEAN  NOT NULL DEFAULT FALSE,
    last_membership_reminder TIMESTAMP WITHOUT TIME ZONE,
    last_fetch               TIMESTAMP WITHOUT TIME ZONE,
    CONSTRAINT "PK_planetside_characters" PRIMARY KEY (character_id),
    CONSTRAINT "UK_planetside_characters_discord_id" UNIQUE (user_id, character_id),
    CONSTRAINT "FK_planetside_characters_users" FOREIGN KEY (user_id)
        REFERENCES public.users (user_id)
);

COMMIT;
