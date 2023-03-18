CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


CREATE TABLE guilds (
    guild_id TEXT PRIMARY KEY
);

CREATE TABLE limits (
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    limit_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    limit_type TEXT NOT NULL,
    limit_action TEXT NOT NULL,
    limit_per INTEGER NOT NULL,
    limit_time INTERVAL NOT NULL
);

CREATE TABLE user_actions (
    limit_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    action_target TEXT NOT NULL
);