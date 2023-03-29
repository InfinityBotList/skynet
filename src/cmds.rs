use crate::{Context, Error};

#[poise::command(
    prefix_command,
    slash_command
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn pings(ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}