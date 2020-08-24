mod help;
mod thanks;
mod top;

pub(crate) use help::MY_HELP;
use thanks::THX_COMMAND;
use top::TOP_COMMAND;

use serenity::{framework::standard::macros::group, model::id::UserId, prelude::TypeMapKey};
use sqlx::PgPool;
use std::time::SystemTime;

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
#[commands(thx, top)]
pub(crate) struct General;
