use futures_util::StreamExt;
use lazy_static::lazy_static;
use lol_quotes::{
    champions::CHAMPIONS,
    commands::{
        ChampionsCommand, Context, IamCommand, SetrateCommand, WhoamiCommand, WhoisCommand,
    },
    db::Database,
    webhooks::Webhooks,
};
use rand::{prelude::IteratorRandom, thread_rng, Rng};
use tracing::{error, info};
use twilight_gateway::{
    cluster::{Cluster, ShardScheme},
    Event, Intents,
};
use twilight_http::Client;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::Interaction,
    id::{ApplicationId, UserId},
};

use std::{env, error::Error, path::Path, sync::Arc};

lazy_static! {
    static ref IGNORED_CHANNELS: Vec<u64> = env::var("IGNORED_CHANNELS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|id_str| id_str.parse().ok())
        .collect();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    let database = match Database::connect("bot.db").await {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("Failed to open database: {}", e);
            return Ok(());
        }
    };

    info!("Running migrations");

    if let Err(e) = database.run_migrations(Path::new("./migrations")).await {
        error!("Failed to run migrations: {}", e);
        return Ok(());
    };

    info!("Done running migrations, booting...");

    let token = match env::var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            error!("DISCORD_TOKEN is not set");
            return Ok(());
        }
    };

    let application_id = match env::var("APPLICATION_ID").ok().and_then(|v| v.parse().ok()) {
        Some(id) => id,
        None => {
            error!("APPLICATION_ID is not set or not a valid integer");
            return Ok(());
        }
    };

    let bot_user_id = match env::var("BOT_USER_ID").ok().and_then(|v| v.parse().ok()) {
        Some(id) => UserId::new(id).unwrap(),
        None => {
            error!("BOT_USER_ID is not set or not a valid integer");
            return Ok(());
        }
    };

    let register_commands = env::var("REGISTER_COMMANDS").is_ok();

    let http = Arc::new(Client::new(token.clone()));
    http.set_application_id(ApplicationId::new(application_id).unwrap());

    if register_commands {
        let commands = &[
            IamCommand::create_command().into(),
            WhoisCommand::create_command().into(),
            WhoamiCommand::create_command().into(),
            SetrateCommand::create_command().into(),
            ChampionsCommand::create_command().into(),
        ];
        http.set_global_commands(commands)?.exec().await?;
    }

    let webhooks = Arc::new(Webhooks::with_client(http.clone()));

    let (cluster, mut events) = Cluster::builder(token, Intents::GUILD_MESSAGES)
        .shard_scheme(ShardScheme::Auto)
        .http_client(http.clone())
        .build()
        .await?;
    let cluster = Arc::new(cluster);

    let cluster_spawn = cluster.clone();

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    info!("Processing events");

    while let Some((_, event)) = events.next().await {
        let http = http.clone();
        let database = database.clone();
        let webhooks = webhooks.clone();

        tokio::spawn(async move {
            if let Event::InteractionCreate(interaction) = event {
                let id = interaction.id();

                let is_autocomplete = matches!(
                    (*interaction).0,
                    Interaction::ApplicationCommandAutocomplete(_)
                );

                match (*interaction).0 {
                    Interaction::ApplicationCommand(application_command)
                    | Interaction::ApplicationCommandAutocomplete(application_command) => {
                        let token = application_command.token;

                        let maybe_user = match application_command.member {
                            Some(member) => member.user,
                            None => application_command.user,
                        };
                        let user_id = match maybe_user {
                            Some(user) => user.id.get() as i64,
                            None => return,
                        };

                        let context = Context {
                            http,
                            database,
                            user_id,
                            interaction_id: id,
                            interaction_token: token,
                        };

                        match application_command.data.name.as_str() {
                            "iam" => {
                                let command = match IamCommand::from_interaction(
                                    application_command.data.into(),
                                ) {
                                    Ok(command) => command,
                                    Err(_) => return,
                                };

                                if is_autocomplete {
                                    let _ = command.autocomplete(context).await;
                                } else {
                                    let _ = command.run(context).await;
                                }
                            }
                            "whois" => {
                                let command = match WhoisCommand::from_interaction(
                                    application_command.data.into(),
                                ) {
                                    Ok(command) => command,
                                    Err(_) => return,
                                };

                                let _ = command.run(context).await;
                            }
                            "whoami" => {
                                let command = match WhoamiCommand::from_interaction(
                                    application_command.data.into(),
                                ) {
                                    Ok(command) => command,
                                    Err(_) => return,
                                };

                                let _ = command.run(context).await;
                            }
                            "setrate" => {
                                let command = match SetrateCommand::from_interaction(
                                    application_command.data.into(),
                                ) {
                                    Ok(command) => command,
                                    Err(_) => return,
                                };

                                let _ = command.run(context).await;
                            }
                            "champions" => {
                                let command = match ChampionsCommand::from_interaction(
                                    application_command.data.into(),
                                ) {
                                    Ok(command) => command,
                                    Err(_) => return,
                                };

                                let _ = command.run(context).await;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            } else if let Event::MessageCreate(message) = event {
                if !message.author.bot && !IGNORED_CHANNELS.contains(&message.channel_id.get()) {
                    let (champion, rate) = match database
                        .get_champion_and_rate(message.author.id.get() as i64)
                        .await
                    {
                        Ok(res) => res,
                        Err(_) => return,
                    };

                    let am_i_mentioned = message
                        .mentions
                        .iter()
                        .any(|mention| mention.id == bot_user_id);

                    let champion = &CHAMPIONS[&champion];

                    let (maybe_quote, odd) = {
                        let mut rng = thread_rng();
                        let maybe_quote = champion.quotes.iter().choose(&mut rng);
                        let odd = rng.gen_range(0..100);

                        (maybe_quote, odd)
                    };

                    if am_i_mentioned || odd <= rate {
                        let webhook =
                            match webhooks.get_webhook_for_channel(message.channel_id).await {
                                Ok(webhook) => webhook,
                                Err(_) => return,
                            };

                        if let Some(token) = webhook.token {
                            let user_display_name = &message
                                .member
                                .as_ref()
                                .and_then(|member| member.nick.as_ref())
                                .unwrap_or(&message.author.name);

                            if let Some(quote) = maybe_quote {
                                let _ = http
                                    .execute_webhook(webhook.id, &token)
                                    .content(quote)
                                    .avatar_url(&champion.icon)
                                    .username(user_display_name)
                                    .exec()
                                    .await;
                            }
                        }
                    }
                }
            }
        });
    }

    Ok(())
}
