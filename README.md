# Thanks_bot
This is a simple discord bot to keep track of helpfull people so mods can reward them.

Currently, it has several commands. You can easily see them using !help as well as get an explantion on how to use them. A few are

* !thx
* !top
* !help
* !questions

# Dependencies
- [rustup](https://www.rust-lang.org/tools/install) to download latest stable rust compiler.

- Cargo, Will be installed automatically by rustup. It manages direct dependencies (for example, the discord library).

- [sqlx-cli](https://crates.io/crates/sqlx-cli), this helps create and update the database.

- [postgresql](https://www.postgresql.org/download/), this is the database used by the server.

# Setup
1. rename .env.example to .env
2. replace `YourDiscordToken` with the token for your bot.
3. replace `postgresql://admin@localhost/thanks_bot` with the correct connection string
4. run `sqlx database create`. This creates the database
5. run `sqlx migrate run`. This runs the migrations so the database has the correct structure. Note: if it gives an error about the `_sqlx_migrations` not existing then that can be ignored (https://github.com/launchbadge/sqlx/issues/640 ) 
6. run `cargo run` to compile and start the bot in debug mode, or `cargo run --release` to compile and start in release mode.
