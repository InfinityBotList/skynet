use std::collections::HashSet;

use log::{error, info};
use poise::serenity_prelude::{FullEvent, RoleAction, UserId};
use serenity::model::guild::audit_log::{Action, ChannelAction};
use sqlx::postgres::PgPoolOptions;

use crate::cache::CacheHttpImpl;

mod autocompletes;
mod cache;
mod cmds;
mod config;
mod crypto;
mod handler;
mod help;
mod core;
mod owner;
mod server;
mod stats;
mod utils;
mod tasks;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// User data, which is stored and accessible in all command invocations
pub struct Data {
    pool: sqlx::PgPool,
    cache_http: cache::CacheHttpImpl,
}

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
            let err = ctx
                .say(format!(
                    "There was an error running this command: {}",
                    error
                ))
                .await;

            if let Err(e) = err {
                error!("SQLX Error: {}", e);
            }
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            error!(
                "[Possible] error in command `{}`: {:?}",
                ctx.command().name,
                error,
            );
            if let Some(error) = error {
                error!("Error in command `{}`: {:?}", ctx.command().name, error,);
                let err = ctx.say(format!("**{}**", error)).await;

                if let Err(e) = err {
                    error!("Error while sending error message: {}", e);
                }
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e);
            }
        }
    }
}

async fn event_listener(ctx: &serenity::client::Context, event: &FullEvent, user_data: &Data) -> Result<(), Error> {
    match event {
        FullEvent::InteractionCreate {
            interaction,
        } => {
            info!("Interaction received: {:?}", interaction.id());
        }
        FullEvent::Ready {
            data_about_bot,
        } => {
            info!("{} is ready!", data_about_bot.user.name);

            tokio::task::spawn(server::setup_server(
                user_data.pool.clone(),
                user_data.cache_http.clone(),
            ));

            tokio::task::spawn(crate::tasks::taskcat::start_all_tasks(
                user_data.pool.clone(),
                user_data.cache_http.clone(),
                ctx.clone(),
            ));
        }
        FullEvent::GuildAuditLogEntryCreate {
            entry,
            guild_id,
        } => {
            info!("Audit log created: {:?}. Guild: {}", entry, guild_id);

            let res = match entry.action {
                Action::Channel(ch) => {
                    let ch_id = entry.target_id.ok_or("No channel ID found")?;

                    match ch {
                        ChannelAction::Create => {
                            info!("Channel created: {}", ch_id);

                            handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                core::UserLimitTypes::ChannelAdd,
                                ch_id.to_string(),
                            )
                            .await
                        }
                        ChannelAction::Delete => {
                            info!("Channel deleted: {}", ch_id);

                            handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                core::UserLimitTypes::ChannelRemove,
                                ch_id.to_string(),
                            )
                            .await
                        }
                        ChannelAction::Update => {
                            info!("Channel updated: {}", ch_id);

                            handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                core::UserLimitTypes::ChannelUpdate,
                                ch_id.to_string(),
                            )
                            .await
                        }
                        _ => Ok(()),
                    }
                }
                Action::Role(ra) => {
                    let r_id = entry.target_id.ok_or("No role ID found")?;

                    match ra {
                        RoleAction::Create => {
                            info!("Role created: {}", r_id);

                            handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                core::UserLimitTypes::RoleAdd,
                                r_id.to_string(),
                            )
                            .await
                        }
                        RoleAction::Update => {
                            info!("Role updated: {}", r_id);

                            handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                core::UserLimitTypes::RoleUpdate,
                                r_id.to_string(),
                            )
                            .await
                        }
                        RoleAction::Delete => {
                            info!("Role deleted: {}", r_id);

                            handler::handle_mod_action(
                                *guild_id,
                                entry.user_id,
                                &user_data.pool,
                                &user_data.cache_http,
                                core::UserLimitTypes::RoleRemove,
                                r_id.to_string(),
                            )
                            .await
                        }
                        _ => Ok(()),
                    }
                }
                _ => Ok(()),
            };

            if let Err(res) = res {
                error!("Error while handling audit log: {}", res);
                return Err(res);
            }
        }
        _ => {}
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    const MAX_CONNECTIONS: u32 = 3; // max connections to the database, we don't need too many here

    std::env::set_var("RUST_LOG", "skynet=info");

    env_logger::init();

    info!("Proxy URL: {}", config::CONFIG.proxy_url);

    let http = serenity::all::HttpBuilder::new(&config::CONFIG.token)
        .proxy(config::CONFIG.proxy_url.clone())
        .ratelimiter_disabled(true)
        .build();

    let client_builder = serenity::all::ClientBuilder::new_with_http(
        http,
        serenity::all::GatewayIntents::non_privileged(),
    );

    // Convert owners to a HashSet
    let owners = config::CONFIG
        .owners
        .iter()
        .cloned()
        .collect::<HashSet<UserId>>();

    let framework = poise::Framework::new(
        poise::FrameworkOptions {
            owners,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("sky.".into()),
                ..poise::PrefixFrameworkOptions::default()
            },
            event_handler: |ctx, event, _fc, user_data| Box::pin(event_listener(ctx, event, user_data)),
            commands: vec![
                register(),
                help::help(),
                help::simplehelp(),
                stats::stats(),
                cmds::ping(),
                cmds::perms(),
                cmds::setup(),
                cmds::limits(),
                cmds::actions(),
                owner::guild(),
            ],
            command_check: Some(|ctx| {
                Box::pin(async move {
                    // Guild admin check
                    if ["help", "simplehelp", "ping"].contains(&ctx.command().name.as_str()) {
                        return Ok(true);
                    }

                    crate::utils::is_guild_admin(
                        &ctx.data().cache_http,
                        &ctx.data().pool,
                        ctx.guild_id().ok_or("Could not get guild id")?,
                        ctx.author().id.to_string(),
                    )
                    .await?;

                    // Look for guild
                    if let Some(guild_id) = ctx.guild_id() {
                        if ["register", "setup"].contains(&ctx.command().name.as_str()) {
                            return Ok(true);
                        }

                        let data = ctx.data();

                        let guild = sqlx::query!(
                            "
                            SELECT COUNT(*)
                            FROM guilds
                            WHERE guild_id = $1
                        ",
                            guild_id.to_string()
                        )
                        .fetch_one(&data.pool)
                        .await?;

                        if guild.count.unwrap_or_default() == 0 {
                            // Guild not found
                            return Err("Please run ``/setup`` to get started!".into());
                        }

                        Ok(true)
                    } else {
                        Err("This command can only be run from servers".into())
                    }
                })
            }),
            pre_command: |ctx| {
                Box::pin(async move {
                    info!(
                        "Executing command {} for user {} ({})...",
                        ctx.command().qualified_name,
                        ctx.author().name,
                        ctx.author().id
                    );
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    info!(
                        "Done executing command {} for user {} ({})...",
                        ctx.command().qualified_name,
                        ctx.author().name,
                        ctx.author().id
                    );
                })
            },
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        },
        move |ctx, _ready, _framework| {
            Box::pin(async move {
                Ok(Data {
                    cache_http: CacheHttpImpl {
                        cache: ctx.cache.clone(),
                        http: ctx.http.clone(),
                    },
                    pool: PgPoolOptions::new()
                        .max_connections(MAX_CONNECTIONS)
                        .connect(&config::CONFIG.database_url)
                        .await
                        .expect("Could not initialize connection"),
                })
            })
        },
    );

    let mut client = client_builder
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
