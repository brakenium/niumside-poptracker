BEGIN;

ALTER TABLE public.users
    ADD COLUMN membership_reminder_forgotten_interval INTERVAL DEFAULT '1 day' NOT NULL;
ALTER TABLE public.users
    ADD COLUMN membership_reminder_first_interval INTERVAL DEFAULT '21 hours' NOT NULL;

COMMIT;
