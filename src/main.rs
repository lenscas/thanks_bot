use async_trait::async_trait;
use dotenv::var;
use serenity::prelude::*;
use serenity::{
    framework::standard::{
        help_commands,
        macros::{ command, group, help, hook},
        Args, CommandGroup,  CommandResult,  HelpOptions,
        StandardFramework,
    },
    model::{channel::Message, gateway::Ready, id::UserId},
};
use sqlx::{query, PgPool};
use std::{
    collections::HashSet,
    time::{Duration, SystemTime},
};
use tokio::stream::StreamExt;
use prettytable::{Cell,Row,Table};

// The framework provides two built-in help commands for you to use.
// But you can also make your own customized help command that forwards
// to the behaviour of either of them.
#[help]
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip = "Hello! GameDev!\n\
I'm here to keep track of who is helpfull so the mods can reward them with a special role.\n\
Did you recently get help and want to show your aprication? Use the !thx command.\n\
If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "+"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Nothing"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip_{dm, guild}` aren't specified.
// If you pass in a value, it will be displayed instead.
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let channel_name = msg.channel_id.name(context).await;
    if let Some(channel_name) = channel_name {
        if var("ALLOWED_HELP_CHANNEL")
            .expect("Help command channel not set")
            .to_lowercase()
            == channel_name.to_lowercase()
        {
            help_commands::plain(context, msg, args, help_options, groups, owners).await;
        }
    }

    Ok(())
}

struct DbPool;

impl TypeMapKey for DbPool {
    type Value = PgPool;
}

struct BotId;

impl TypeMapKey for BotId {
    type Value = UserId;
}
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected! with id {}", ready.user.name, ready.user.id);
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
        .bucket("potentially_big_output", |b| {
            b.delay(5).time_span(30).limit(2)
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

fn get_time_as_unix_epoch(time: SystemTime) -> i64 {
    match time.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(x) => x,
        //this only happens if now - 1 minute is earlier than the unix_epoch.
        //the error returns how much earlier that was
        //so, this only happens if the system clock is messed up and I think it is still safe to do this in that case
        Err(x) => x.duration(),
    }
    .as_secs() as i64
}

#[group]
#[commands(thx, top)]
struct General;

#[command]
#[aliases("thanks","thank")]
#[description("Lets me know that someone helped you out or was awesome in another way.")]
#[usage("{some text containing one or more users who you want to thank for something}")]
#[example = "@thanks_bot for being AWESOME!"]
#[only_in("guild")]
#[help_available]
async fn thx(ctx: &Context, msg: &Message) -> CommandResult {
    let thanking: &Vec<_> = &msg.mentions;
    let mention_count = thanking.len();
    if mention_count == 0 {
        msg.channel_id.say(&ctx.http, "Please ping the user in your thank message so I know who you are thanking.").await?;
        return Ok(())
    }
    
    let mut transaction = {
        let data = ctx.data.read().await;
        let pool = data.get::<DbPool>().expect("No db pool?");
        pool.begin().await?
    };

    let current_time_minus_one_minute =
        get_time_as_unix_epoch(SystemTime::now() - Duration::new(60, 0));

    let thanker_id = i64::from(msg.author.id);
    let mut contains_at_least_one_to_recent = false;
    let mut thanked_self = false;
    for thanked_user in thanking {
        if thanked_user.id == msg.author.id {
            thanked_self=true;
        }
        let thanked_user_id = i64::from(thanked_user.id);
        let count = query!(
            "
            SELECT count(*) AS count 
            FROM recent_thanked
            WHERE user_id = $1 
            AND did_thank = $2
            AND at_time > $3
            ",
            thanker_id,
            thanked_user_id,
            current_time_minus_one_minute
        )
        .fetch_one(&mut transaction)
        .await?
        .count
        .expect("SQL COUNT returned NULL. The world is broken!");
        if count == 0 {
            query!(
                "
                INSERT INTO thanked_users (user_id, times)
                VALUES($1,1) 
                ON CONFLICT (user_id) 
                DO 
                UPDATE SET times = thanked_users.times + 1;
                ",
                thanked_user_id
            )
            .execute(&mut transaction)
            .await?;

            query!(
                "
                    INSERT INTO recent_thanked (user_id, did_thank, at_time)
                    VALUES ($1,$2,$3)
                    ON CONFLICT ON CONSTRAINT recent_thanked_pk 
                    DO
                    UPDATE SET at_time = $3;
                ",
                thanker_id,
                thanked_user_id,
                get_time_as_unix_epoch(SystemTime::now())
            )
            .execute(&mut transaction)
            .await?;
        } else {
            contains_at_least_one_to_recent = true;
        }
        println!("thanked user = {:?}", thanked_user.id);
    }
    transaction.commit().await?;
    let contents = match (mention_count,contains_at_least_one_to_recent,thanked_self, msg.mentions_me(&ctx).await?) {
        (1,_, true,_) =>String::from( "To keep it fair, you can not thank yourself."),
        (1,true,_,_) => String::from("Sorry, you already thanked this user not so long ago. Wait a minute before thanking him/her again"),
        (1,_,_,true) => String::from("Thank you! I just do my best! :)"),
        (1, false,false,false) => String::from("Thank you for letting me know that this person helped you out."),
        (2,true,true,_) => String::from("Sorry, but you can't thank yourself and you already recently thanked the other person"),
        (_,contains_too_recent,thanked_self, _ ) => {
            let mut base_message = String::from("Thanks for informing me that these users helped you out!");
            if contains_too_recent {
                base_message.push_str(" Your message contains users you already thanked. Wait a minute before thanking them again :).")
            }
            if thanked_self {
                base_message.push_str(" You can't thank yourself.")
            }
            base_message
        }
    };

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

#[command]
#[bucket = "potentially_big_output"]
#[aliases("list")]
#[description("Show the people who got thanked the most, mostly for the mods so they can give rewards.")]
#[usage("")]
#[only_in("guild")]
async fn top(ctx: &Context, msg: &Message) -> CommandResult {
    let is_in_incorrect_channel = msg
    .channel_id
    .name(&ctx)
    .await
    .map(|v| {
        v != var("ALLOWED_TOP_COMMAND")
            .expect("top channel not set")
            .to_lowercase()
    })
    .unwrap_or(true);
    if is_in_incorrect_channel {
        return Ok(())
    }
    let (mut con, own_id) = {
        let data = ctx.data.read().await;
        let id = i64::from(*data.get::<BotId>().expect("No bot id set"));
        let pool = data.get::<DbPool>().expect("No db pool?");
        (pool.acquire().await?,id)
    };
    let res: Vec<(UserId, i64)> = query!(
        "SELECT user_id, times
        FROM thanked_users
        WHERE user_id != $1
        ORDER BY times DESC
        LIMIT 10",own_id
    )
    .fetch(&mut con)
    .map(|v| v.map(|v| (UserId::from(v.user_id as u64), v.times)))
    .collect::<Result<_, _>>()
    .await?;

    let header_row = Row::new(vec![Cell::new("name"),Cell::new("times")]);
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(header_row);
    
    for (user_id, times) in res {
        let name = match (user_id.to_user(&ctx).await, msg.guild_id) {
            (Ok(user), Some(guild)) => user.nick_in(&ctx, guild).await.unwrap_or(user.name),
            (Ok(user), None) => user.name,
            (Err(x), _) => match x {
                SerenityError::Model(model) => match model {
                    ModelError::InvalidUser => String::from("InvalidUser"),
                    x => return Err(SerenityError::Model(x).into()),
                },
                x => return Err(x.into()),
            },
        };
        table.add_row(Row::new(vec![Cell::new(&name), Cell::new(&times.to_string())]));
    }

    let mut message = String::from("The top most thanked users are\n");
    message.push_str("```\n");
    message.push_str(&table.to_string());
    message.push_str("\n```");
    msg.channel_id.say(&ctx.http, message).await?;

    Ok(())
}
