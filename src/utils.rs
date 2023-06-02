use poise::serenity_prelude::GuildId;
use sqlx::postgres::types::PgInterval;

use crate::cache::CacheHttpImpl;

pub fn parse_pg_interval(i: PgInterval) -> String {
    let seconds =
        i.microseconds / 1000000 + ((i.days * 86400) as i64) + ((i.months * 2628000) as i64);

    let dur = std::time::Duration::from_secs(seconds.try_into().unwrap_or_default());

    format!("{:?}", dur)
}

pub async fn is_guild_admin(
    cache_http: &CacheHttpImpl,
    pool: &sqlx::PgPool,
    guild_id: GuildId,
    user_id: String,
) -> Result<(), crate::Error> {
    // Convert guild_id to guild
    {
        let guild = guild_id
            .to_guild_cached(&cache_http.cache)
            .ok_or("Could not fetch guild from cache")?;

        if user_id == guild.owner_id.to_string() {
            return Ok(());
        }
    }

    // Check if user in guild_admins
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM guild_admins WHERE guild_id = $1 AND user_id = $2",
        guild_id.to_string(),
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|_| "Could not fetch guild admin status")?;

    if count.count.unwrap_or_default() == 0 {
        return Err("You are not a guild admin".into());
    }

    Ok(())
}

#[derive(poise::ChoiceParameter)]
pub enum Unit {
    #[name = "Seconds"]
    Seconds,
    #[name = "Minutes"]
    Minutes,
    #[name = "Hours"]
    Hours,
    #[name = "Days"]
    Days,
}

impl Unit {
    pub fn to_seconds(&self) -> i64 {
        match self {
            Unit::Seconds => 1,
            Unit::Minutes => 60,
            Unit::Hours => 3600,
            Unit::Days => 86400,
        }
    }
}
