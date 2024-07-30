use crate::config;
use crate::websocket::{
    socket::{Client, ReaderArc, WriterArc},
    util::{self, log_time},
};
use futures_util::SinkExt;
use regex::Regex;
use tokio;
use tokio_tungstenite::tungstenite::protocol::Message;

// GLOBALUSERSTATE      -> bot connects to server
// ROOMSTATE            -> bot joins a channel OR mod changes chat room settings
// USERSTATE            -> user joins a channel OR this bot sends a PRIVMSG
//
// WHISPER              -> user sends a whisper message
// PRIVMSG              -> a message is posted in the chatroom
//
// CLEARCHAT            -> all messages (in chatroom OR from a specific user) have been removed
// CLEARMSG             -> specific message has been removed from chatroom
//
// USERNOTICE           -> event (e.g user subscription) occurs
//
// RECONNECT            -> server needs to perform maintenance, our bot will disconnect soon
// HOSTTARGET           -> channel has started/stopped host mode

pub const KEEPALIVE_PONG: &str = "PONG";
pub const KEEPALIVE_PING: &str = "PING";
pub const REQ: &str = "CAP REQ :twitch.tv/tags twitch.tv/commands";
pub const STATES: [&str; 3] = ["GLOBALUSERSTATE", "ROOMSTATE", "USERSTATE"];

pub struct ChatMessage {
    chatter: String,
    channel: String,
    content: String,
}

impl ChatMessage {
    fn new(chatter: String, channel: String, content: String) -> Self {
        return Self {
            chatter,
            channel,
            content,
        };
    }
}

pub struct StateChanged {
    state_type: String,
    chatter: Option<String>,
    channel: Option<String>,
}

impl StateChanged {
    fn new(state_type: String, chatter: Option<String>, channel: Option<String>) -> Self {
        return Self {
            state_type,
            chatter,
            channel,
        };
    }
}

pub struct Irc {
    nick: String,
    user: String,
    channel: String,
    auth: String,
}

impl Irc {
    fn new(nick: String, user: String, channel: String, auth: String) -> Self {
        return Self {
            nick,
            user,
            channel,
            auth,
        };
    }

    pub async fn irc_init(&self, url: &str) -> (WriterArc, ReaderArc) {
        let (writer, reader) = Client::open_streams(url).await;
        Client::write_socket(&writer, REQ).await;
        Client::write_socket(&writer, &self.auth).await;
        Client::write_socket(&writer, &self.nick).await;
        Client::write_socket(&writer, &self.user).await;
        Client::write_socket(&writer, &self.channel).await;

        return (writer, reader);
    }
}

pub async fn open_irc(url: &str, channel: &str) -> (ReaderArc, WriterArc) {
    let config = &config::CONFIG_READER;

    let auth = format!("PASS oauth:{}", config.authorization.auth);
    let nick = format!("NICK {}", config.authorization.user);
    let user = format!(
        "USER {} 8 * :{}", // idk what the `8` refers to here :3
        config.authorization.user, config.authorization.user
    );

    // channel name is passed as a CLI arg for this branch, auto-join twitch.tv/kori if no arg
    // supplied
    let channel = format!("JOIN #{}", channel);
    let irc = Irc::new(nick, user, channel, auth);
    let (irc_writer, irc_reader) = irc.irc_init(url).await;

    // we return the (writer, reader) bindings flipped to (reader, writer) as i just feel like this
    // is more intuitive
    return (irc_reader, irc_writer);
}

pub async fn parse_msg(data: Message, writer: &WriterArc) {
    let vec = data.to_string();
    let data_vec = vec
        .split("\r\n")
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>(); // let binding extends lifetime

    for data_str in data_vec {
        if data_str.starts_with("PING :tmi.twitch.tv") {
            keepalive(Some(writer)).await;
        } else if data_str.starts_with("PONG :tmi.twitch.tv") {
            keepalive(None).await;
        } else if data_str.contains("PRIVMSG") {
            let msg = chatroom(&data_str).await;
            println!(
                "[{}] [+] <[in '#{}'][from '/{}']> -> '{}'",
                log_time(),
                msg.chatter,
                msg.channel,
                msg.content
            );

        // GLOBALUSERSTATE      -> bot connects to server
        // ROOMSTATE            -> bot joins a channel OR mod changes chat room settings
        // USERSTATE            -> user joins a channel OR this bot sends a PRIVMSG
        } else if STATES.iter().any(|state| data_str.contains(state)) {
            let state = curr_state(data_str).await;
            print!("[{}] [*] New state: {}", log_time(), state.state_type);

            match (state.channel.as_ref(), state.chatter.as_ref()) {
                (Some(channel), Some(chatter)) => {
                    print!(" - in '#{}' - for '/{}'", channel, chatter)
                }
                (Some(channel), _) => print!(" - in '#{}'", channel),
                (None, Some(chatter)) => print!(" - for '/{}'", chatter),
                _ => (),
            }

            println!();
        }
    }
}

async fn curr_state(msg: &str) -> StateChanged {
    let state_re = Regex::new(r"^.*? :tmi.twitch.tv (?P<state_type>\w+)");
    let captures = state_re
        .unwrap()
        .captures(msg)
        .expect("[-] Error in state regex selection.");

    let state_type = captures.name("state_type").unwrap().as_str().to_string();

    if state_type == "GLOBALUSERSTATE".to_string() {
        let chatter_re = Regex::new(r"^.*?;display-name=(?P<chatter>[^;]+).*")
            .unwrap()
            .captures(msg)
            .expect("[-] Error in chatter_re regex.");
        let chatter = chatter_re.name("chatter").unwrap().as_str().to_string();
        return StateChanged::new(state_type, Some(chatter), None);
    }

    if state_type == "USERSTATE".to_string() {
        let chatter_re = Regex::new(r"^.*?;display-name=(?P<chatter>[^;]+).*")
            .unwrap()
            .captures(msg)
            .expect("[-] Error in chatter_re regex.");
        let channel_re = Regex::new(r"^.*? #(?P<channel>\w+)")
            .unwrap()
            .captures(msg)
            .expect("[-] Error in channel_re regex.");
        let chatter = chatter_re.name("chatter").unwrap().as_str().to_string();
        let channel = channel_re.name("channel").unwrap().as_str().to_string();

        return StateChanged::new(state_type, Some(chatter), Some(channel));
    }

    return StateChanged::new(state_type, None, None);
}

async fn chatroom(msg: &str) -> ChatMessage {
    let chat_re = Regex::new(
        r"^.*?display-name=(?P<chatter>[^;]+).*? PRIVMSG #(?P<channel>\w+) :(?P<content>.*)",
    );

    let captures = chat_re
        .unwrap()
        .captures(msg)
        .expect("[-] Error in regex selection.");

    let chatter = captures.name("chatter").unwrap().as_str().to_string();
    let channel = captures.name("channel").unwrap().as_str().to_string();
    let content = captures.name("content").unwrap().as_str().to_string();

    return ChatMessage::new(chatter, channel, content);
}

async fn keepalive(writer_opt: Option<&WriterArc>) {
    match writer_opt {
        Some(writer) => {
            let res = Message::Text(KEEPALIVE_PONG.into());
            writer
                .lock()
                .await
                .send(res)
                .await
                .expect("[-] Client PONG failed.");
            println!("[{}] [+] Client response to server PING ok.", log_time());
        }
        None => {
            println!(
                "[{}] [+] Server response to client PING (handshake ok).",
                log_time()
            );
        }
    }
}
