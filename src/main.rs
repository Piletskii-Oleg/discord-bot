use serenity::client::{Context, EventHandler};
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::GatewayIntents;
use serenity::{async_trait, Client};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            match msg.channel_id.say(&ctx.http, "Pog!").await {
                Err(why) => println!("Error sending message: {:?}", why),
                Ok(_) => println!("{} has sent a message to bot!", msg.author.name),
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"));

    let token = std::env::var("DISCORD_TOKEN").expect("Provide a valid discord API token");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while starting client: {:?}", why);
    }
}