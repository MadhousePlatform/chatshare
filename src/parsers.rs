use regex::Regex;

use crate::docker::{MessageType, ServerEventMessage};

pub struct Parser {
    join_re: Regex,
    part_re: Regex,
    message_re: Regex,
}

impl Parser {
    pub fn new(join_re: &str, part_re: &str, message_re: &str) -> Self {
        Parser {
            join_re: Regex::new(join_re).expect("This should not fail creating a RegEx"),
            part_re: Regex::new(part_re).expect("This should not fail creating a RegEx"),
            message_re: Regex::new(message_re).expect("This should not fail creating a RegEx"),
        }
    }

    pub fn parse_message(&self, message: String, server: String) -> Option<ServerEventMessage> {
        let mut emit_message = ServerEventMessage {
            content: String::new(),
            source: server.clone(),
            target: String::new(),
            message_type: MessageType::JOIN,
        };

        if self.join_re.is_match(&message) {
            let captures = self.join_re.captures(&message).unwrap();
            emit_message.target = String::from(captures.get(1).unwrap().as_str());
        } else if self.part_re.is_match(&message) {
            let captures = self.part_re.captures(&message).unwrap();
            emit_message.message_type = MessageType::PART;
            emit_message.target = String::from(captures.get(1).unwrap().as_str());
        } else if self.message_re.is_match(&message) {
            let captures = self.message_re.captures(&message).unwrap();
            emit_message.message_type = MessageType::MESSAGE;
            emit_message.content = String::from(captures.get(2).unwrap().as_str());
            emit_message.target = String::from(captures.get(1).unwrap().as_str());
        } else {
            return None;
        }

        return Some(emit_message);
    }
}

pub fn vanilla() -> Parser {
    let join_re = r"^\[.*\] \[Server thread/INFO\]: (.*) joined the game$";
    let part_re = r"^\[.*\] \[Server thread/INFO\]: (.*) left the game$";
    let message_re = r"^\[.*\] \[Server thread/INFO\]: <(.*)> (.*)$";

    Parser::new(join_re, part_re, message_re)
}

pub fn cobblemon() -> Parser {
    let join_re = r"^\[.*\] \[Server thread/INFO\]: (.*) joined the game$";
    let part_re = r"^\[.*\] \[Server thread/INFO\]: (.*) left the game$";
    let message_re = r"^\[.*\] \[Server thread/INFO\]: \[.*\] (.*) » (.*)$";

    Parser::new(join_re, part_re, message_re)
}

pub fn mechanical() -> Parser {
    let join_re =
        r"\[.*\] \[Server thread/INFO\] \[minecraft/DedicatedServer\]: (.*) joined the game$";
    let part_re =
        r"\[.*\] \[Server thread/INFO\] \[minecraft/DedicatedServer\]: (.*) left the game$";
    let message_re =
        r"\[.*\] \[Server thread/INFO\] \[minecraft/DedicatedServer\]: \[.*\] <(.*)> (.*)$";

    Parser::new(join_re, part_re, message_re)
}

pub fn atm() -> Parser {
    let join_re = r"\[.*\] \[Server thread/INFO\] \[minecraft/MinecraftServer\]: \[.*\] <(.*)> joined the game$";
    let part_re = r"\[.*\] \[Server thread/INFO\] \[minecraft/MinecraftServer\]: \[.*\] <(.*)> left the game$";
    let message_re =
        r"\[.*\] \[Server thread/INFO\] \[minecraft/MinecraftServer\]: <\[.*\] <(.*)>> (.*)$";

    Parser::new(join_re, part_re, message_re)
}

pub fn rr3() -> Parser {
    let join_re = r"\[.* INFO\]: (.*)\[.*\] logged in";
    let part_re = r"\[.* INFO\]: (.*) left the game.$";
    let message_re = r"\[.* INFO\]: \\u\{1b\}\[m<(.*)\\u\{1b\}\[m> (.*)$";

    Parser::new(join_re, part_re, message_re)
}
