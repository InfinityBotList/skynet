CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE guilds (
    guild_id TEXT PRIMARY KEY
);

CREATE TABLE guild_admins (
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    user_id TEXT NOT NULL
);

-- Stores the limits that are applied to a guild
CREATE TABLE limits (
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    limit_id TEXT PRIMARY KEY DEFAULT uuid_generate_v4(),
    limit_name TEXT NOT NULL default 'Untitled',
    limit_type TEXT NOT NULL,
    limit_action TEXT NOT NULL,
    limit_per INTEGER NOT NULL,
    limit_time INTERVAL NOT NULL
);


-- Stores a list of user actions and which limits they have hit
CREATE TABLE user_actions (
    action_id TEXT PRIMARY KEY,
    limit_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    action_target TEXT NOT NULL,
    limits_hit TEXT[] NOT NULL DEFAULT '{}'
);

-- Stores the past limits that have been applied to a user, cannot be done using simple user_actions
CREATE TABLE hit_limits (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    limit_id TEXT NOT NULL REFERENCES limits(limit_id) ON DELETE CASCADE ON UPDATE CASCADE,
    cause TEXT[] NOT NULL DEFAULT '{}',
    notes TEXT[] NOT NULL DEFAULT '{}'
);