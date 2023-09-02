use serenity::framework::standard::StandardFramework;
use serenity::model::prelude::{ChannelId, Message};
use serenity::{async_trait, prelude::*};
use tokio::sync::broadcast::Sender;

use crate::docker::{MessageType, ServerEventMessage};

struct Handler {
    tx: Sender<ServerEventMessage>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let channel_id = ChannelId(std::env::var("DISCORD_CHANNEL").unwrap().parse().unwrap());
        if msg.channel_id != channel_id {
            return;
        }

        if msg.is_own(ctx.cache) {
            return;
        }

        self.tx
            .send(ServerEventMessage {
                source: String::from("Discord"),
                target: msg.author.name,
                message_type: MessageType::MESSAGE,
                content: msg.content,
            })
            .unwrap();
    }
}

pub async fn handle_connection(tx: Sender<ServerEventMessage>) {
    let framework = StandardFramework::new();
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Discord token is missing from the environment variables");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut rx = tx.subscribe();
    let mut client = Client::builder(token, intents)
        .framework(framework)
        .event_handler(Handler { tx })
        .await
        .expect("Error creating Discord client");

    let channel_id = ChannelId(std::env::var("DISCORD_CHANNEL").unwrap().parse().unwrap());

    let cache_and_http = client.cache_and_http.clone();
    tokio::spawn(async move {
        loop {
            let ServerEventMessage {
                source,
                message_type,
                content,
                target,
            } = rx.recv().await.unwrap();

            if source == "Discord" {
                continue;
            }

            let message = match message_type {
                MessageType::JOIN => format!("➡ {} has joined {}", target, source),
                MessageType::PART => format!("⬅ {} has left {}", target, source),
                MessageType::MESSAGE => format!("[{}] <{}> {}", source, target, content),
            };

            channel_id
                .say(cache_and_http.as_ref(), message)
                .await
                .unwrap();
        }
    });
    client.start().await.unwrap();
}
