-- -- Add migration script here
-- -- Add not null definition_id to public.zone
-- -- The columns above are calculated from zone_id
-- -- Add zone_definition table to public
-- -- Add not null definition_id to public.zone
-- -- Create table public.zone_definition
-- -- Add name to public.zone_definition
-- -- Add description to public.zone_definition
-- -- Add last_update to public.zone_definition
-- -- Remove name from public.zone
-- -- Remove description from public.zone
-- -- Remove last_update from public.zone
--
-- -- Automatically calculate definition_id from the lower 4 zone_id bytes
--
-- -- Write SQL:
-- CREATE TABLE public.zone_definition (
--     definition_id integer NOT NULL,
--     name character varying,
--     description character varying,
--     last_update timestamp without time zone,
--     CONSTRAINT "PK_zone_definition" PRIMARY KEY (definition_id)
-- );
--
-- ALTER TABLE public.zone
--     ADD COLUMN definition_id integer NOT NULL
--         GENERATED ALWAYS AS (zone_id & x'0000FFFF'::integer) STORED;
--
-- ALTER TABLE public.zone
--     ADD CONSTRAINT "FK_zone_definition" FOREIGN KEY (definition_id)
--         REFERENCES public.zone_definition (definition_id)
--         ON UPDATE RESTRICT
--         ON DELETE RESTRICT
--         NOT VALID;
--
-- -- Copy all definitions from public.zone to public.zone_definition
-- -- Copy name from public.zone to public.zone_definition
-- -- Copy description from public.zone to public.zone_definition
-- -- Copy last_update from public.zone to public.zone_definition
--
-- -- Write SQL:
-- INSERT INTO public.zone_definition (definition_id, name, description, last_update)
--     SELECT definition_id, name, description, last_update
--     FROM public.zone
--     GROUP BY definition_id, name, description, last_update;
--
--