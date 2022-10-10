use std::{collections::HashSet, sync::Arc};

use serenity::{
    client::bridge::gateway::ShardManager,
    model::id::ChannelId,
    prelude::{Mutex, TypeMapKey},
};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub type GameLock = HashSet<ChannelId>;
pub struct GameLockContainer;

impl TypeMapKey for GameLockContainer {
    type Value = Arc<Mutex<GameLock>>;
}
