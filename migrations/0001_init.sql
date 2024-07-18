CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users
(
    id       UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email    TEXT UNIQUE NOT NULL,
    username TEXT        NOT NULL,
    password TEXT        NOT NULL
);

CREATE TABLE IF NOT EXISTS passwords
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id    UUID   NOT NULL,
    name        TEXT   NOT NULL,
    password    BYTEA  NOT NULL,
    salt        BYTEA,
    website     TEXT,
    username    TEXT,
    description TEXT,
    tags        TEXT[] NOT NULL  DEFAULT ARRAY []::TEXT[],
    CONSTRAINT fk_owner FOREIGN KEY (owner_id) REFERENCES users (id)
);

CREATE INDEX idx_email ON users USING hash (email);
CREATE INDEX idx_passwords_owner_id ON passwords (owner_id);

