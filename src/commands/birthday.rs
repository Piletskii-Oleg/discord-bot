use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;

#[group]
#[commands(birthday)]
struct Birthday;

#[command]
#[sub_commands(add, edit, remove, mod_menu)]
async fn birthday(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(ctx, "what???").await?;
    Ok(())
}

#[command]
#[required_permissions("MANAGE_ROLES")]
async fn mod_menu(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(ctx, "mod_menu").await?;
    Ok(())
}

#[command]
async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(ctx, "added?").await?;
    Ok(())
}

#[command]
async fn edit(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg.reply(ctx, "edited").await?;
    Ok(())
}

#[command]
async fn remove(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}
