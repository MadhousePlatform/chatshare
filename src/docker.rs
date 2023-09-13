use bollard::{
    container::{AttachContainerOptions, AttachContainerResults},
    Docker,
};
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast::Sender;
use tokio_stream::StreamExt;

use crate::{parsers::Parser, servers::ServerInfo};

#[derive(Clone, Debug)]
pub enum MessageType {
    JOIN,
    PART,
    MESSAGE,
}

#[derive(Clone, Debug)]
pub struct ServerEventMessage {
    pub source: String,
    pub message_type: MessageType,
    pub target: String,
    pub content: String,
}

pub async fn handle_server(server: ServerInfo, tx: Sender<ServerEventMessage>, parser: Parser) {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let AttachContainerResults {
        mut output,
        mut input,
    } = docker
        .attach_container(
            server.uuid.clone().as_str(),
            Some(AttachContainerOptions::<String> {
                stdin: Some(true),
                stdout: Some(true),
                stream: Some(true),
                ..Default::default()
            }),
        )
        .await
        .unwrap();

    let mut rx = tx.subscribe();
    let my_name = server.name.clone();
    tokio::spawn(async move {
        while let Some(Ok(output)) = output.next().await {
            let msg = String::from(output.to_string().trim());
            let emit_message = parser.parse_message(msg, server.name.clone());
            if emit_message.is_some() {
                tx.send(emit_message.unwrap()).unwrap();
            }
        }
    });

    tokio::spawn(async move {
        loop {
            let ServerEventMessage {
                target,
                source,
                message_type,
                content,
            } = rx.recv().await.unwrap();

            if source == my_name {
                continue;
            }

            let message = match message_type {
                MessageType::JOIN => {
                    format!("tellraw @a [{{\"text\":\"[{}] \",\"color\":\"red\"}},{{\"text\":\"{} has joined the server\",\"color\":\"white\"}}]\n", source, target)
                }
                MessageType::PART => {
                    format!("tellraw @a [{{\"text\":\"[{}] \",\"color\":\"red\"}},{{\"text\":\"{} has left the server\",\"color\":\"white\"}}]\n", source, target)
                }
                MessageType::MESSAGE => {
                    format!("tellraw @a [{{\"text\":\"[{}] \",\"color\":\"red\"}},{{\"text\":\"<{}> \",\"color\":\"blue\"}},{{\"text\":\"{}\",\"color\":\"white\"}}]\n", source, target, content)
                }
            };

            input.write(message.as_bytes()).await.unwrap();
        }
    });
}
