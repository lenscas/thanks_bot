mod commands;

use async_trait::async_trait;
use dotenv::var;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::hook, CommandResult, StandardFramework},
    model::{channel::Message, gateway::Ready},
};
use sqlx::PgPool;

use commands::MY_HELP;

use crate::commands::{BotId, DbPool, GENERAL_GROUP,CONFIG_GROUP};
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!(
            "{} is connected! with id {}",
            ready.user.name, ready.user.id
        );
        let mut data = ctx.data.write().await;
        data.insert::<BotId>(ready.user.id);
    }
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => println!("Processed command '{}'", command_name),
        Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Could not read .env file");
    let discord_token = var("DISCORD_TOKEN").expect("DISCORD_TOKEN is not set.");
    let db_url = var("DATABASE_URL").expect("DATABASE_URL is not set.");
    let pool = PgPool::connect(&db_url)
        .await
        .expect("Couldn't connect to database");

    let framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .prefix("!")
                // In this case, if "," would be first, a message would never
                // be delimited at ", ", forcing you to trim your arguments if you
                // want to avoid whitespaces at the start of each.
                .delimiters(vec![", ", ","])
        })
        .help(&MY_HELP)
        .group(&GENERAL_GROUP)
        .group(&CONFIG_GROUP)
        .bucket("potentially_big_output", |b| {
            b.delay(10).time_span(120).limit(2)
        })
        .await
        .after(after);
    let mut client = Client::new(&discord_token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<DbPool>(pool);
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
