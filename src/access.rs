use poise::serenity_prelude::{UserId, GuildId};
use sqlx::{PgPool};
use strum_macros::{EnumString, EnumVariantNames, Display};

use crate::{Error, cache::CacheHttpImpl};

/// Represents a list of access levels a user has
pub struct AccessLevelManager {
    perms: Vec<AccessLevel>
}

impl AccessLevelManager {
    pub fn has(&self, level: AccessLevel) -> bool {
        self.perms.contains(&AccessLevel::Owner) || self.perms.contains(&level)
    }

    /// Returns an error if the user does not have the specified access level
    /// 
    /// Returns the same AccessLevelManager if the user does have the specified access level
    pub fn error_if_not(self, level: AccessLevel) -> Result<Self, Error> {
        if !self.has(level) {
            Err(
                format!(
                    "Whoa there! You need to be a {} to do that!",
                    level
                ).into()
            )
        } else {
            Ok(self)
        }
    }
}

#[derive(EnumString, Display, PartialEq, EnumVariantNames, Clone, Debug, Copy)]
#[strum(serialize_all = "snake_case")]
pub enum AccessLevel {
    Owner,
    Admin,
    Manager
}

impl AccessLevel {
    pub async fn for_user(
        cache_http: &CacheHttpImpl,
        pool: &PgPool,
        user: UserId,
        guild: GuildId) -> Result<AccessLevelManager, Error> {
            let guild_owner = {
                let guild = cache_http.cache.guild(guild).ok_or("Could not find guild in cache!")?;
                guild.owner_id
            };

            if user == guild_owner {
                return Ok(AccessLevelManager {
                    perms: vec![AccessLevel::Owner]
                });
            }

            let rec = sqlx::query!(
                "
                    SELECT access_level
                    FROM guild_access
                    WHERE user_id = $1 AND guild_id = $2
                ",
                user.to_string(),
                guild.to_string()
            )
            .fetch_all(pool)
            .await?;

            let levels = rec.iter().map(|r| r.access_level.parse()).collect::<Result<Vec<_>, _>>()?;

            Ok(AccessLevelManager {
                perms: levels
            })         
        }
}