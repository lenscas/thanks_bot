mod codeblock;
mod config;
mod delete;
mod github;
mod google;
mod help;
mod learnprogramming;
mod questions;
mod thanks;
mod top;

pub(crate) use help::MY_HELP;

use codeblock::CODEBLOCK_COMMAND;
use config::SET_DELAY_COMMAND;
use delete::DELETE_COMMAND;
use github::{BUG_COMMAND, GITHUB_COMMAND};
use google::GOOGLE_COMMAND;
use learnprogramming::LEARNPROGRAMMING_COMMAND;
use questions::QUESTIONS_COMMAND;
use thanks::THX_COMMAND;
use top::TOP_COMMAND;

use dotenv::var;
use serenity::{
    client::Context,
    framework::standard::{macros::group},
    model::{channel::Message}
};

pub(crate) const NON_THANKS_COMMANDS_VAR_KEY: &str = "OTHER_NON_THANKS_COMMANDS";



#[group]
#[commands(thx, top, github, bug, codeblock, google, questions, learnprogramming)]
pub(crate) struct General;

#[group]
#[commands(set_delay, delete)]
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

