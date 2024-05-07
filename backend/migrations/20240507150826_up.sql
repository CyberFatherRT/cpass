-- Add migration script here

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email VARCHAR NOT NULL,
    username VARCHAR NOT NULL,
    password VARCHAR NOT NULL,
    password_hint TEXT
);

CREATE TABLE IF NOT EXISTS passwords (
    id UUID PRIMARY KEY,
    owner_id UUID,
    name VARCHAR NOT NULL,
    website VARCHAR,
    username VARCHAR,
    description TEXT,
    CONSTRAINT fk_owner FOREIGN KEY (owner_id) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS tags (
    id SERIAL,
    password_id UUID,
    content VARCHAR,
    CONSTRAINT fk_tag FOREIGN KEY (password_id) REFERENCES passwords(id)
)
