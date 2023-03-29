use poise::serenity_prelude::{UserId, GuildId};
use sqlx::{types::{chrono::{DateTime, Utc}}, PgPool, postgres::types::PgInterval};
use strum_macros::{EnumString, EnumVariantNames, Display};

use crate::Error;

#[derive(EnumString, Display, PartialEq, EnumVariantNames, Clone, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum UserLimitTypes {
    RoleAdd, // set
    RoleUpdate, // set
    RoleRemove, // set
    ChannelAdd, // set
    ChannelUpdate, // set
    ChannelRemove, //set
    Kick,
    Ban,
    Unban,
}

impl UserLimitTypes {
    pub fn to_cond(&self) -> String {
        match &self {
            Self::RoleAdd => "Roles Created".to_string(),
            Self::RoleUpdate => "Roles Updated".to_string(),
            Self::RoleRemove => "Role Removed".to_string(),
            Self::ChannelAdd => "Channels Created".to_string(),
            Self::ChannelUpdate => "Channels Updated".to_string(),
            Self::ChannelRemove => "Channels Removed".to_string(),
            Self::Kick => "Kicks".to_string(),
            Self::Ban => "Bans".to_string(),
            Self::Unban => "Unbans".to_string(),
        }
    }
}

#[derive(EnumString, Display, PartialEq, EnumVariantNames, Clone, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum UserLimitActions {
    RemoveAllRoles,
    KickUser,
    BanUser,
}

#[derive(Clone, Debug)]
pub struct Action {
    pub action_id: String,
    pub limit_type: UserLimitTypes,
    pub created_at: DateTime<Utc>,
    pub user_id: UserId,
    pub guild_id: GuildId,
    pub action_target: UserId,
    pub handled_for: Vec<String>,
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
    pub limit_id: String,
    pub limit_name: String,
    pub limit_type: UserLimitTypes,
    pub limit_action: UserLimitActions,
    pub limit_per: i32,
    pub limit_time: PgInterval,
}

impl Limit {
    pub async fn from_guild(pool: &PgPool, guild_id: GuildId) -> Result<Vec<Self>, Error> {
        let rec = sqlx::query!(
            "
                SELECT limit_id, limit_name, limit_type, limit_action, limit_per, limit_time
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
                limit_name: r.limit_name,
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
                    AND NOT($4 = ANY(handled_for))
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