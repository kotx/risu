mod commands;
mod sources;
mod state;
mod tags;
mod utils;

use crate::commands::game::*;
use crate::state::{GameLock, GameLockContainer, ShardManagerContainer};
use serde::Deserialize;
use serenity::model::prelude::Activity;
use serenity::prelude::{Mutex, GatewayIntents};
use serenity::{
    client::{Context, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{event::ResumedEvent, prelude::Ready},
    Client,
};
use std::{collections::HashSet, sync::Arc};

#[derive(Deserialize, Debug)]
struct Config {
    prefix: String,
    token: String,
    #[serde(default = "default_level")]
    log: log::LevelFilter,
}

fn default_level() -> log::LevelFilter {
    if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    }
}

struct Handler;

#[group]
#[commands(start)]
struct Game;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        log::info!(
            "Connected as {}#{} ({})",
            ready.user.name,
            ready.user.discriminator,
            ready.user.id
        );

        ctx.set_activity(Activity::playing("with %start!")).await;
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        log::debug!("Resumed");
    }
}

#[tokio::main]
async fn main() {
    let config = load_config().expect("Failed to get config from env");

    kaf::with_filter(
        Box::new(|target, level| {
            (target == "risu" || target.starts_with("risu::")) || (level <= log::Level::Warn)
        }),
        config.log,
    );
    log::info!("Logging started");

    let http = Http::new(&config.token);

    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
        .configure(|c| c.allow_dm(false).owners(owners).prefix(&config.prefix)) // set the bot's prefix to "~"
        .group(&GAME_GROUP);

    let mut client = Client::builder(config.token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<GameLockContainer>(Arc::new(Mutex::new(GameLock::new())));
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    client.start().await.unwrap();
}

fn load_config() -> Result<Config, envy::Error> {
    dotenv::from_filename(".env.local").ok();
    dotenv::from_filename(".env").ok();

    envy::prefixed("RISU_").from_env::<Config>()
}
