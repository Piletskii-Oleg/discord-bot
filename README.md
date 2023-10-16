# discord-bot
A simple discord bot written in Rust using [serenity-rs][serenity-rs].

## How to use
The bot can be invited to a server using [this link][invite-link].

## Build from source
Before building, `.env` file must be created, similar to `.env.example`, which contains
* Discord API Token, which can be found [here][discord-token]. You can visit [this page][find-token] for detailed instructions.
* Database URL. Currently, it should be `sqlite:database.sqlite` due to how `main.rs` is written, but it may change later.

After that use the following commands:
```shell
cargo install sqlx-cli
sqlx database setup
```
This will create the database and apply necessary migrations.

[serenity-rs]: https://github.com/serenity-rs/serenity
[invite-link]: https://discord.com/api/oauth2/authorize?client_id=504276268670517260&permissions=402729984&scope=bot
[discord-token]: https://discord.com/developers/applications
[find-token]: https://www.writebots.com/discord-bot-token/