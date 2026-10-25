BEGIN;

CREATE SCHEMA IF NOT EXISTS toroauth;

CREATE TABLE IF NOT EXISTS toroauth.identity (
    id UUID PRIMARY KEY,
    data JSONB
);

CREATE UNIQUE INDEX toroauth_identity_username_idx ON toroauth.identity ((data->>'username'));

CREATE INDEX toroauth_identity_login_idx ON toroauth.identity ((data->>'username'), (data->>'password'));

CREATE TABLE IF NOT EXISTS toroauth.session (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES toroauth.identity(id)
);

COMMIT;
