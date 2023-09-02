use bollard::{
    container::{AttachContainerOptions, AttachContainerResults},
    Docker,
};
use regex::Regex;
use tokio::io::AsyncWriteExt;
use tokio::sync::broadcast::Sender;
use tokio_stream::StreamExt;

use crate::servers::ServerInfo;

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

pub async fn handle_server(server: ServerInfo, tx: Sender<ServerEventMessage>) {
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

    let join_re = Regex::new(r"^\[(.*)\] \[Server thread/INFO\]: (.*) joined the game$").unwrap();
    let part_re = Regex::new(r"^\[(.*)\] \[Server thread/INFO\]: (.*) left the game$").unwrap();
    let message_re = Regex::new(r"^\[(.*)\] \[Server thread/INFO\]: <(.*)> (.*)$").unwrap();

    let message_re_cobblemon = Regex::new(r"^\[(.*)\] \[Server thread/INFO\]: \[(.*)\] (.*) » (.*)$").unwrap();

    let join_re_mechanical = Regex::new(r"\[(.*)\] \[Server thread/INFO\] \[minecraft/DedicatedServer\]: (.*) joined the game$").unwrap();
    let part_re_mechanical = Regex::new(r"\[(.*)\] \[Server thread/INFO\] \[minecraft/DedicatedServer\]: (.*) left the game$").unwrap();
    let message_re_mechanical = Regex::new(r"\[(.*)\] \[Server thread/INFO\] \[minecraft/DedicatedServer\]: \[(.*)\] <(.*)> (.*)$").unwrap();

    let join_re_atm9 = Regex::new(r"\[(.*)\] \[Server thread/INFO\] \[minecraft/MinecraftServer\]: \[.*\] <(.*)> joined the game$").unwrap();
    let part_re_atm9 = Regex::new(r"\[(.*)\] \[Server thread/INFO\] \[minecraft/MinecraftServer\]: \[.*\] <(.*)> left the game$").unwrap();
    let message_re_atm9 = Regex::new(r"\[(.*)\] \[Server thread/INFO\] \[minecraft/MinecraftServer\]: <\[.*\] <(.*)>> (.*)$").unwrap();

    let join_re_rr3 = Regex::new(r"\[.* INFO\]: (.*)\[.*\] logged in").unwrap();
    let part_re_rr3 = Regex::new(r"\[.* INFO\]: (.*) left the game.$").unwrap();
    let message_re_rr3 = Regex::new(r"\[.* INFO\]: \\u\{1b\}\[m<(.*)\\u\{1b\}\[m> (.*)$").unwrap();

    let mut rx = tx.subscribe();
    let my_name = server.name.clone();
    tokio::spawn(async move {
        while let Some(Ok(output)) = output.next().await {
            let msg = String::from(output.to_string().trim());
            let mut emit_message = ServerEventMessage {
                content: String::new(),
                source: server.name.clone(),
                target: String::new(),
                message_type: MessageType::JOIN,
            };

            // Vanilla
            if join_re.is_match(&msg) {
                let captures = join_re.captures(&msg).unwrap();
                emit_message.target = String::from(captures.get(2).unwrap().as_str());
            } else if part_re.is_match(&msg) {
                let captures = part_re.captures(&msg).unwrap();
                emit_message.message_type = MessageType::PART;
                emit_message.target = String::from(captures.get(2).unwrap().as_str());
            } else if message_re.is_match(&msg) {
                let captures = message_re.captures(&msg).unwrap();
                emit_message.message_type = MessageType::MESSAGE;
                emit_message.content = String::from(captures.get(3).unwrap().as_str());
                emit_message.target = String::from(captures.get(2).unwrap().as_str());

            // Cobblemon
            } else if message_re_cobblemon.is_match(&msg) {
                let captures = message_re_cobblemon.captures(&msg).unwrap();
                emit_message.message_type = MessageType::MESSAGE;
                emit_message.content = String::from(captures.get(4).unwrap().as_str());
                emit_message.target = String::from(captures.get(3).unwrap().as_str());

            // Mechanical engineering
            } else if join_re_mechanical.is_match(&msg) {
                let captures = join_re_mechanical.captures(&msg).unwrap();
                emit_message.target = String::from(captures.get(2).unwrap().as_str());
            } else if part_re_mechanical.is_match(&msg) {
                let captures = part_re_mechanical.captures(&msg).unwrap();
                emit_message.message_type = MessageType::PART;
                emit_message.target = String::from(captures.get(2).unwrap().as_str());
            } else if message_re_mechanical.is_match(&msg) {
                let captures = message_re_mechanical.captures(&msg).unwrap();
                emit_message.message_type = MessageType::MESSAGE;
                emit_message.content = String::from(captures.get(4).unwrap().as_str());
                emit_message.target = String::from(captures.get(3).unwrap().as_str());

            // ATM9
            } else if join_re_atm9.is_match(&msg) {
                let captures = join_re_atm9.captures(&msg).unwrap();
                emit_message.target = String::from(captures.get(2).unwrap().as_str());
            } else if part_re_atm9.is_match(&msg) {
                let captures = part_re_atm9.captures(&msg).unwrap();
                emit_message.message_type = MessageType::PART;
                emit_message.target = String::from(captures.get(2).unwrap().as_str());
            } else if message_re_atm9.is_match(&msg) {
                let captures = message_re_atm9.captures(&msg).unwrap();
                emit_message.message_type = MessageType::MESSAGE;
                emit_message.content = String::from(captures.get(3).unwrap().as_str());
                emit_message.target = String::from(captures.get(2).unwrap().as_str());

            // RR3
            } else if join_re_rr3.is_match(&msg) {
                let captures = join_re_rr3.captures(&msg).unwrap();
                emit_message.target = String::from(captures.get(1).unwrap().as_str());
            } else if part_re_rr3.is_match(&msg) {
                let captures = part_re_rr3.captures(&msg).unwrap();
                emit_message.message_type = MessageType::PART;
                emit_message.target = String::from(captures.get(1).unwrap().as_str());
            } else if message_re_rr3.is_match(&msg) {
                let captures = message_re_rr3.captures(&msg).unwrap();
                emit_message.message_type = MessageType::MESSAGE;
                emit_message.content = String::from(captures.get(2).unwrap().as_str());
                emit_message.target = String::from(captures.get(1).unwrap().as_str());

            } else {
                continue;
            }
            tx.send(emit_message).unwrap();
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
