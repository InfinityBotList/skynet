use poise::serenity_prelude::{UserId, GuildId};
use sqlx::{types::{Uuid, chrono::{DateTime, Utc}}, PgPool};
use strum_macros::{EnumString, EnumVariantNames};

use crate::Error;

#[derive(EnumString, PartialEq, EnumVariantNames, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum UserLimitTypes {
    RoleAdd,
    RoleRemove,
    ChannelAdd,
    ChannelRemove,
    Kick,
    Ban,
    Unban,
}

#[derive(EnumString, PartialEq, EnumVariantNames, Clone)]
#[strum(serialize_all = "snake_case")]
pub enum UserLimitActions {
    RemoveAllRoles,
    KickUser,
    BanUser,
}

/*
    limit_type TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    action_target BIGINT NOT NULL
 */
#[derive(Clone)]
pub struct Action {
    pub limit_type: UserLimitTypes,
    pub created_at: DateTime<Utc>,
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub action_target: UserId,
}

impl Action {
    pub async fn from_guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT limit_type, created_at, user_id, action_target
                FROM user_actions
                WHERE guild_id = $1
            ",
            guild_id.to_string()
        )
        .fetch_all(pool)
        .await?;

        let mut actions = Vec::new();

        for r in rec {
            actions.push(Self {
                guild_id,
                limit_type: r.limit_type.parse()?,
                created_at: r.created_at,
                user_id: r.user_id.parse()?,
                action_target: r.action_target.parse()?,
            });
        }

        Ok(actions)
    }
}

pub struct Limit {
    pub guild_id: GuildId,
    pub limit_id: Uuid,
    pub limit_type: UserLimitTypes,
    pub limit_action: UserLimitActions,
    pub limit_per: i32,
}

impl Limit {
    pub async fn from_guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT limit_id, limit_type, limit_action, limit_per
                FROM limits
                WHERE guild_id = $1
            ",
            guild_id.to_string()
        )
        .fetch_all(pool)
        .await?;

        let mut limits = Vec::new();

        for r in rec {
            limits.push(Self {
                guild_id,
                limit_id: r.limit_id,
                limit_type: r.limit_type.parse()?,
                limit_action: r.limit_action.parse()?,
                limit_per: r.limit_per,
            });
        }

        Ok(limits)
    }
}

pub struct UserLimitsHit {
    pub limit_type: UserLimitTypes,
    pub limit_actions: UserLimitActions,
    pub limit_id: Uuid,
    pub cause: Vec<Action>, 
}

impl UserLimitsHit {
    /// Returns a list of all limits that have been hit for a specific user.
    pub fn from(limits: Vec<Limit>, actions: Vec<Action>) -> Vec<Self> {
        let mut hits = Vec::new();

        for limit in limits {
            let mut cause = Vec::new();

            for action in actions.iter() {
                if action.limit_type == limit.limit_type {
                    cause.push(action.clone());
                }
            }

            if cause.len() >= limit.limit_per as usize {
                hits.push(Self {
                    limit_type: limit.limit_type,
                    limit_actions: limit.limit_action,
                    limit_id: limit.limit_id,
                    cause,
                });
            }
        }

        hits
    }
}