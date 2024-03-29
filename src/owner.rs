use poise::serenity_prelude::GuildId;

use crate::{config, Context, Error};

async fn is_owner(ctx: Context<'_>) -> Result<bool, Error> {
    if config::CONFIG.owners.contains(&ctx.author().id) {
        Ok(true)
    } else {
        Err("You are not a bot owner".into())
    }
}

/// Guild base command
#[poise::command(
    prefix_command,
    check = "is_owner",
    subcommands("staff_guilddel", "staff_guildleave")
)]
pub async fn guild(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Delete server
#[poise::command(
    rename = "del",
    track_edits,
    prefix_command,
    check = "is_owner"
)]
pub async fn staff_guilddel(
    ctx: Context<'_>,
    #[description = "The guild ID to remove"] guild: String,
) -> Result<(), Error> {
    let gid = guild.parse::<GuildId>()?;

    ctx.serenity_context().http.delete_guild(gid).await?;

    ctx.say("Removed guild").await?;

    Ok(())
}

/// Delete server
#[poise::command(
    rename = "leave",
    track_edits,
    prefix_command,
    check = "is_owner"
)]
pub async fn staff_guildleave(
    ctx: Context<'_>,
    #[description = "The guild ID to leave"] guild: String,
) -> Result<(), Error> {
    let gid = guild.parse::<GuildId>()?;

    ctx.serenity_context().http.leave_guild(gid).await?;

    ctx.say("Removed guild").await?;

    Ok(())
}
