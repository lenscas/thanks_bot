use serenity::framework::standard::macros::group;

mod codeblock;
mod google;
mod learnprogramming;
mod questions;

use codeblock::CODEBLOCK_COMMAND;
use google::GOOGLE_COMMAND;
use learnprogramming::LEARNPROGRAMMING_COMMAND;
use questions::QUESTIONS_COMMAND;
#[group]
#[commands(codeblock, google, questions, learnprogramming)]
pub(crate) struct AllChannels;
