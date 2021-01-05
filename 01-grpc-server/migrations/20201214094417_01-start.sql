-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users
(
	id uuid DEFAULT uuid_generate_v1() NOT NULL CONSTRAINT users_pkey PRIMARY KEY,
	name text NOT NULL,
	birth_date date NOT NULL,
	created_at timestamp with time zone default CURRENT_TIMESTAMP,
	updated_at timestamp with time zone,
	custom_data jsonb
);

CREATE UNIQUE INDEX users_name ON users (name);

INSERT INTO public.users (id, name, birth_date, created_at, updated_at, custom_data) VALUES (DEFAULT, 'Roberto', '1977-03-10', DEFAULT, null, '{"points": 10}');
