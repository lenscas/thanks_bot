# Thanks_bot
This is a simple discord bot to keep track of helpfull people so mods can reward them.

Currently, it has only 3 commands.:

!help -> Shows how to use this bot (Is more in depth than this readme)

!thx -> is to let the bot know if/when someone is helpful

!top -> gets the top most helpfull people so they can be rewarded.

# Dependencies
- [rustup](https://www.rust-lang.org/tools/install) to download latest stable rust compiler.

- Cargo, Will be installed automatically by rustup. It manages direct dependencies (for example, the discord library).

- [sqlx-cli](https://crates.io/crates/sqlx-cli), this helps create and update the database.

- [postgresql]](https://www.postgresql.org/download/), this is the database used by the server.

# Setup
1. rename .env.example to .env
2. replace `YourDiscordToken` with the token for your bot.
3. replace `postgresql://admin@localhost/thanks_bot` with the correct connection string
4. run `sqlx database create`. This creates the database
5. run `sqlx migrate run`. This runs the migrations so the database has the correct structure. Note: if it gives an error about the `_sqlx_migrations` not existing then that can be ignored (https://github.com/launchbadge/sqlx/issues/640 ) 
6. run `cargo run` to compile and start the bot in debug more, or `cargo run --release` to compile and start in release mode.