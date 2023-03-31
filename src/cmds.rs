use poise::{serenity_prelude::{CreateEmbed, Member}, CreateReply};

use crate::{Context, Error};

#[poise::command(
    prefix_command,
    slash_command
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// Add admin to the server
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
)]
pub async fn add_admin(ctx: Context<'_>, user: Member) -> Result<(), Error> {
    // Check that user is guild owner
    if ctx.author().id != ctx.guild().ok_or("Could not get guild id")?.owner_id {
        return Err("Only guild owners can add new admins".into());
    }

    // Check if user is already an admin, if so return
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM guild_admins WHERE guild_id = $1 AND user_id = $2",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        user.user.id.to_string()
    )
    .fetch_one(&ctx.data().pool)
    .await
    .map_err(|_| "Could not fetch guild admin status")?;

    if count.count.unwrap_or_default() > 0 {
        return Err("User is already an admin".into());
    }

    sqlx::query!(
        "INSERT INTO guild_admins (guild_id, user_id) VALUES ($1, $2)",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        user.user.id.to_string()
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Added admin successfully").await?;

    Ok(())
}

/// Remove admin from the server
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
)]
pub async fn remove_admin(ctx: Context<'_>, user: Member) -> Result<(), Error> {
    // Check that user is guild owner
    if ctx.author().id != ctx.guild().ok_or("Could not get guild id")?.owner_id {
        return Err("Only guild owners can remove admins".into());
    }

    // Check if user is not already an admin, if so return
    let count = sqlx::query!(
        "SELECT COUNT(*) FROM guild_admins WHERE guild_id = $1 AND user_id = $2",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        user.user.id.to_string()
    )
    .fetch_one(&ctx.data().pool)
    .await
    .map_err(|_| "Could not fetch guild admin status")?;

    if count.count.unwrap_or_default() == 0 {
        return Err("User is not already an admin?".into());
    }

    sqlx::query!(
        "DELETE FROM guild_admins WHERE guild_id = $1 AND user_id = $2",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        user.user.id.to_string()
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("Removed admin successfully").await?;

    Ok(())
}

/// Limits base command
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    subcommands("limits_add", "limits_view")
)]
pub async fn limits(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add a limit to the server
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    rename = "add"
)]
pub async fn limits_add(
    ctx: Context<'_>, 
    limit_name: String, 
    limit_type: crate::limits::UserLimitTypesChoices, 
    limit_per: i32, 
    limit_time: i64, 
    limit_time_unit: crate::utils::Unit,
    limit_action: crate::limits::UserLimitActionsChoices
) -> Result<(), Error> {
    crate::utils::is_guild_admin(
        &ctx.data().cache_http,
        &ctx.data().pool, 
        ctx.guild_id().ok_or("Could not get guild id")?, 
        ctx.author().id.to_string(),
    ).await?;

    let limit_type = limit_type.resolve();
    let limit_action = limit_action.resolve();

    // Add limit to db
    sqlx::query!(
        "
            INSERT INTO limits (
                guild_id,
                limit_name,
                limit_type,
                limit_action,
                limit_per,
                limit_time
            )
            VALUES (
                $1, 
                $2, 
                $3, 
                $4, 
                $5,
                make_interval(secs => $6)
            )
        ",
        ctx.guild_id().ok_or("Could not get guild id")?.to_string(),
        limit_name,
        limit_type.to_string(),
        limit_action.to_string(),
        limit_per,
        (limit_time * limit_time_unit.to_seconds()) as f64
    )
    .execute(&ctx.data().pool)  
    .await?;

    ctx.say("Added limit successfully").await?;

    Ok(())
}

/// View the limits setup for this server
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    rename = "view"
)]
pub async fn limits_view(ctx: Context<'_>) -> Result<(), Error> {    
    crate::utils::is_guild_admin(
        &ctx.data().cache_http,
        &ctx.data().pool, 
        ctx.guild_id().ok_or("Could not get guild id")?, 
        ctx.author().id.to_string()
    ).await?;

    let limits = crate::limits::Limit::from_guild(
        &ctx.data().pool, 
        ctx.guild_id().ok_or("Could not get guild id")?
    ).await?;
    
    let mut embeds = vec![];

    let mut added = 0;
    let mut i = 0;

    for limit in limits {
        added += 1;

        if added >= 25 {
            added = 0;
            i += 1;
        }

        if embeds.len() <= i {
            embeds.push(
                CreateEmbed::default()
                    .title("Limits")
                    .color(0x00ff00)
            );
        }

        embeds[i] = embeds[i].clone().field(
            limit.limit_name, 
            format!(
                "If {amount} ``{cond}`` happen every {time}: ``{then}``",
                amount = limit.limit_per,
                cond = limit.limit_type.to_cond(),
                time = crate::utils::parse_pg_interval(limit.limit_time),
                then = limit.limit_action.to_cond()
            ), 
            false
        );
    }

    let mut reply = CreateReply::new();

    reply.embeds = embeds;

    ctx.send(
        reply
    )
    .await?;

    Ok(())
}