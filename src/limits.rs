use poise::serenity_prelude::{UserId, GuildId};
use sqlx::{types::{Uuid, chrono::{DateTime, Utc}}, PgPool, postgres::types::PgInterval};
use strum_macros::{EnumString, EnumVariantNames, Display};

use crate::Error;

#[derive(EnumString, Display, PartialEq, EnumVariantNames, Clone, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum UserLimitTypes {
    RoleAdd,
    RoleRemove,
    ChannelAdd,
    ChannelUpdate,
    ChannelRemove,
    Kick,
    Ban,
    Unban,
}

#[derive(EnumString, PartialEq, EnumVariantNames, Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct Action {
    pub action_id: Uuid,
    pub limit_type: UserLimitTypes,
    pub created_at: DateTime<Utc>,
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub action_target: UserId,
    pub handled_for: Vec<Uuid>,
}

impl Action {
    pub async fn from_guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT action_id, limit_type, created_at, user_id, action_target, handled_for
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
                action_id: r.action_id,
                limit_type: r.limit_type.parse()?,
                created_at: r.created_at,
                user_id: r.user_id.parse()?,
                action_target: r.action_target.parse()?,
                handled_for: r.handled_for
            });
        }

        Ok(actions)
    }
}

#[derive(Debug)]
pub struct Limit {
    pub guild_id: GuildId,
    pub limit_id: Uuid,
    pub limit_type: UserLimitTypes,
    pub limit_action: UserLimitActions,
    pub limit_per: i32,
    pub limit_time: PgInterval,
}

impl Limit {
    pub async fn from_guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT limit_id, limit_type, limit_action, limit_per, limit_time
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
                limit_time: r.limit_time
            });
        }

        Ok(limits)
    }
}

#[derive(Debug)]
pub struct UserLimitsHit {
    pub limit: Limit,
    pub cause: Vec<Action>, 
}

impl UserLimitsHit {
    /// Returns a list of all limits that have been hit for a specific guild
    pub async fn hit(guild_id: GuildId, pool: &PgPool) -> Result<Vec<Self>, Error> {
        let limits = Limit::from_guild(pool, guild_id).await?;
        
        let mut hits = Vec::new();

        for limit in limits {
            let mut cause = Vec::new();

            // Find all actions that apply to this limit
            let rec = sqlx::query!(
                "
                    SELECT action_id, created_at, user_id, action_target, handled_for
                    FROM user_actions
                    WHERE guild_id = $1
                    AND $4 <> ANY(handled_for)
                    AND NOW() - created_at < $2
                    AND limit_type = $3
                ",
                guild_id.to_string(),
                limit.limit_time,
                limit.limit_type.to_string(),
                limit.limit_id

            )
            .fetch_all(pool)
            .await?;
        
            for r in rec {
                cause.push(Action {
                    guild_id,
                    limit_type: limit.limit_type.clone(),
                    created_at: r.created_at,
                    user_id: r.user_id.parse()?,
                    action_target: r.action_target.parse()?,
                    action_id: r.action_id,
                    handled_for: r.handled_for
                });
            }
    
            if cause.len() >= limit.limit_per as usize {
                hits.push(Self {
                    limit,
                    cause,
                });
            }
        }

        Ok(hits)
    }
}