use poise::{serenity_prelude::CreateEmbed, CreateReply};

use crate::{Context, Error};

#[poise::command(
    prefix_command,
    slash_command
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// View the limits setup for this server
#[poise::command(
    prefix_command,
    slash_command,
    guild_only,
    required_permissions = "MANAGE_GUILD"
)]
pub async fn limits(ctx: Context<'_>) -> Result<(), Error> {    
    crate::access::AccessLevel::for_user(
        &ctx.data().cache_http,
        &ctx.data().pool,
        ctx.author().id, 
        ctx.guild_id().ok_or("Could not get guild id")?
    )
    .await?
    .error_if_not(crate::access::AccessLevel::Admin)?;

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
                "If {amount} ``{cond}`` happen every {time} {then}",
                amount = limit.limit_per,
                cond = limit.limit_type.to_cond(),
                time = crate::utils::parse_pg_interval(limit.limit_time),
                then = limit.limit_action
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