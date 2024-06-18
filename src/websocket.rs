use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::sync::Arc;
use tokio::{self, net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use crate::util;

// actual constants we will need to use
const URL_CHAT: &str = "wss://irc-ws.chat.twitch.tv/";
const URL_PSUB: &str = "wss://pubsub-edge.twitch.tv/v1";
const REQ: &str = "CAP REQ :twitch.tv/tags twitch.tv/commands";

// fuck their chungus lives
pub type WriterArc = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;
pub type ReaderArc = Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>;

pub struct WebsocketClient {
    nick: String,
    user: String,
    channel: String,
    auth: String,
}

impl WebsocketClient {
    pub fn new(nick: String, user: String, channel: String, auth: String) -> Self {
        return Self {
            nick,
            user,
            channel,
            auth,
        };
    }

    // parse an &str into a Message then await a lock on the writer mutex and send the Message data
    async fn write_socket(writer: &WriterArc, data: &str) {
        let msg = Message::Text(data.to_string());

        println!("[{}][WRITE]: Sending data: '{}'",
            util::log_time().await, data);

        writer.lock().await.send(msg).await
            .expect("[ERR]: Failed to send the previous message.");
    }

    // await a lock on the reader mutex and parse its contents into an Option,
    // returning the data if available, otherwise None.
    pub async fn read_socket(reader: &ReaderArc) -> Option<Message> {
        let mut reader_lock = reader.lock().await;

        if let Some(data) = reader_lock.next().await {
            return Some(
                data.expect("[ERR]: Incoming data was found, but the program failed to return the data."),
            );
        }

        return None;
    }

    // open a twitch websocket connection, send identifying data, and return that connection's
    // separate reader/writer streams as two (owned) `Arc<Mutex<_>>`
    pub async fn open_connection(&self, url: &str) -> (WriterArc, ReaderArc) {
        println!("\n[{}][INIT]: Connecting to socket @ '{}'...", util::log_time().await, url);

        // open a secure websocket connection (ie, `wss://`) with the given URL
        let (stream, _) = connect_async(url).await.expect("[ERR]: Couldn't connect to the socket!");
        println!("[{}][INIT]: Initial handshake ok\n", util::log_time().await);

        // split IO streams and load into arc mutex
        let (writer, reader) = stream.split();
        let writer_arc = Arc::new(Mutex::new(writer));
        let reader_arc = Arc::new(Mutex::new(reader));

        match url {
            URL_CHAT => {

                // handle IRC init
                Self::write_socket(&writer_arc, REQ).await;
                Self::write_socket(&writer_arc, &self.auth).await;
                Self::write_socket(&writer_arc, &self.nick).await;
                Self::write_socket(&writer_arc, &self.user).await;

                // we get two commands from the socket around here:
                // ```
                // :tmi.twitch.tv CAP * ACK :twitch.tv/tags twitch.tv/commands
                // :tmi.twitch.tv 001 <USER> :Welcome, GLHF! // ... more words in this msg, none important...
                // ```
                // however we aren't required to acknowledge them

                Self::write_socket(&writer_arc, &self.channel).await;
            }
            URL_PSUB => {
                // handle sending events init
                // (this socket has many more options and uses JSON and i cant be bothered rn)
            }
            _ => {
                panic!("[ERR]: Unable to match a Twitch WS URL; this shouldn't ever occur.");
            }
        }

        // return stream mutex arcs as a tuple
        return (writer_arc, reader_arc);
    }
}
