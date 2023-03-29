CREATE TABLE guilds (
    guild_id TEXT PRIMARY KEY
);

CREATE TABLE guild_access (
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    user_id TEXT NOT NULL,
    access_level TEXT NOT NULL
);

CREATE TABLE limits (
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    limit_id TEXT PRIMARY KEY,
    limit_name TEXT NOT NULL default 'Untitled',
    limit_type TEXT NOT NULL,
    limit_action TEXT NOT NULL,
    limit_per INTEGER NOT NULL,
    limit_time INTERVAL NOT NULL
);

CREATE TABLE user_actions (
    action_id TEXT PRIMARY KEY,
    limit_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    action_target TEXT NOT NULL,
    handled_for TEXT[] NOT NULL DEFAULT '{}'
);

-- Stores the past limits that have been applied to a user
CREATE TABLE hit_limits (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    limit_id TEXT NOT NULL REFERENCES limits(limit_id) ON DELETE CASCADE ON UPDATE CASCADE,
    cause TEXT[] NOT NULL DEFAULT '{}'
);