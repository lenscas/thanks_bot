mod codeblock;
mod config;
mod github;
mod help;
mod thanks;
mod top;

pub(crate) use help::MY_HELP;

use codeblock::CODEBLOCK_COMMAND;
use config::SET_DELAY_COMMAND;
use github::{BUG_COMMAND, GITHUB_COMMAND};
use thanks::THX_COMMAND;
use top::TOP_COMMAND;

use dotenv::var;
use serenity::{
    client::Context,
    framework::standard::macros::group,
    model::{channel::Message, id::UserId},
    prelude::TypeMapKey,
};
use sqlx::PgPool;
use std::time::SystemTime;

pub(crate) const NON_THANKS_COMMANDS_VAR_KEY: &str = "OTHER_NON_THANKS_COMMANDS";

fn get_time_as_unix_epoch(time: SystemTime) -> i64 {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(x) => x,
        //this happens if earlier > time. Which would mean that the system time is screwed up.
        //the duration I get in the error then refers to how much time it was earlier
        //maybe I should just panic instead?
        Err(x) => x.duration(),
    }
    .as_secs() as i64
}

pub(crate) struct BotId;

impl TypeMapKey for BotId {
    type Value = UserId;
}
pub(crate) struct DbPool;

impl TypeMapKey for DbPool {
    type Value = PgPool;
}

#[group]
#[commands(thx, top, github, bug, codeblock)]
pub(crate) struct General;

#[group]
#[commands(set_delay)]
pub(crate) struct Config;

async fn is_in_incorrect_channel(ctx: &Context, msg: &Message) -> bool {
    msg.channel_id
        .name(&ctx)
        .await
        .map(|v| {
            v != var(NON_THANKS_COMMANDS_VAR_KEY)
                .expect("top channel not set")
                .to_lowercase()
        })
        .unwrap_or(true)
}
