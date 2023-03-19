use log::info;
use poise::serenity_prelude::{GuildId, Member};
use sqlx::PgPool;

use crate::{Error, cache::CacheHttpImpl, limits};

/*
    limit_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id TEXT NOT NULL,
    guild_id TEXT NOT NULL REFERENCES guilds(guild_id) ON DELETE CASCADE ON UPDATE CASCADE,
    action_target TEXT NOT NULL

 */

pub async fn handle_mod_action(
    guild_id: GuildId,
    member: Member,
    pool: &PgPool,
    cache_http: &CacheHttpImpl,
    action: limits::UserLimitTypes,
    action_target: String
) -> Result<(), Error> {
    // Insert into user_actions
    sqlx::query!(
        "
            INSERT INTO user_actions (guild_id, user_id, limit_type, action_target)
            VALUES ($1, $2, $3, $4)
        ",
        guild_id.to_string(),
        member.user.id.to_string(),
        action.to_string(),
        action_target
    )
    .execute(pool)
    .await?;

    // Check if they hit any limits yet
    let hit = limits::UserLimitsHit::hit(guild_id, pool).await?;

    for hit_limit in hit {
        // We have a hit limit for this user
        info!("Hit limit: {:?}", hit_limit);

        for action in hit_limit.cause.iter() {
            sqlx::query!(
                "
                UPDATE user_actions
                SET handled_for = array_append(handled_for, $1)
                WHERE action_id = $2",
                hit_limit.limit.limit_id,
                action.action_id
            )
            .execute(pool)
            .await?;   
        }

        sqlx::query!(
            "
            INSERT INTO hit_limits
            (guild_id, user_id, limit_id, cause)
            VALUES ($1, $2, $3, $4)",
            guild_id.to_string(),
            member.user.id.to_string(),
            hit_limit.limit.limit_id,
            &hit_limit.cause.iter().map(|a| a.action_id).collect::<Vec<_>>()
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}