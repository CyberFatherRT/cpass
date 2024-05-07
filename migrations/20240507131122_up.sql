-- Add migration script here

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    username VARCHAR NOT NULL,
    password VARCHAR NOT NULL
);

CREATE TABLE IF NOT EXISTS passwords (
    id UUID PRIMARY KEY,
    owner_id UUID NOT NULL,
    website VARCHAR,
    username VARCHAR,
    description TEXT,
    password VARCHAR NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES users(id);
);

CREATE TABLE IF NOT EXISTS tags (
    password_id UUID,
    tag VARCHAR NOT NULL,
    FOREIGN KEY (password_id) REFERENCES passwords(id);
)
