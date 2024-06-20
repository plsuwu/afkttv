use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::sync::Arc;
use tokio::{self, net::TcpStream, sync::Mutex, time};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use crate::{
    config,
    util::{self, URL_CHAT, URL_EVNT},
};

// actual constants we will need to use
const KEEPALIVE_PONG: &str = "PONG";
const KEEPALIVE_PING: &str = "PING";
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

        println!(
            "[{}][WRITE]: Sending data: '{}'",
            util::log_time(),
            data
        );

        writer
            .lock()
            .await
            .send(msg)
            .await
            .expect("[ERR]: Failed to send the previous message.");
    }

    // await a lock on the reader mutex and parse its contents into an Option,
    // returning the data if available, otherwise None.
    pub async fn read_socket(reader: &ReaderArc) -> Option<Message> {
        let mut reader_lock = reader.lock().await;

        if let Some(data) = reader_lock.next().await {
            return Some(data.expect(
                "[ERR]: Incoming data was found, but the program failed to return the data.",
            ));
        }

        return None;
    }

    // open a twitch websocket connection, send identifying data, and return that connection's
    // separate reader/writer streams as two (owned) `Arc<Mutex<_>>`
    pub async fn open_connection(&self, url: &str) -> (WriterArc, ReaderArc) {
        println!(
            "\n[{}][INIT]: Connecting to socket @ '{}'...",
            util::log_time(),
            url
        );

        // open a secure websocket connection (ie, `wss://`) with the given URL
        let (stream, _) = connect_async(url)
            .await
            .expect("[ERR]: Couldn't connect to the socket!");
        println!("[{}][INIT]: Initial handshake ok\n", util::log_time());

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

            URL_EVNT => {
                // handle sending events init
                // (this socket has many more options and uses JSON and other things and i cant be bothered rn)
            }

            _ => {
                panic!("[ERR]: Unable to match an expected Twitch websocket URL!");
            }
        }

        // return stream references
        return (writer_arc, reader_arc);
    }
}

// https://dev.twitch.tv/docs/cli/websocket-event-command/
// https://dev.twitch.tv/docs/pubsub/#connection-management
pub async fn open_conn(url: &str) {
    // automatically-derived `.clone()` on CONFIG_READER
    let config = &config::CONFIG_READER;

    let auth = format!("PASS oauth:{}", config.authorization.auth);
    let nick = format!("NICK {}", config.authorization.user);
    let user = format!(
        "USER {} 8 * :{}", // idk what `8` refers to here as it seems we can connect without it
        config.authorization.user, config.authorization.user
    );

    // this just joins the user's own channel until i work out how i want to retrieve channel
    // activity
    let join = format!("JOIN #{}", config.authorization.user);
    let irc = WebsocketClient::new(nick, user, join, auth);
    let (irc_writer, irc_reader) = irc.open_connection(url).await;

    // let evt = websocket::WebsocketClient::new(
    //     // this socket wants json stuff and has many more options so i
    //     // will figure it out later.
    //  );

    //let (evt_writer, evt_reader) = evt.open_connection(URL_PSUB).await;
    let irc_writer_clone = irc_writer.clone();

    tokio::task::spawn(async move {
        loop {
            // add a small random jitter this timer for events sock (see linked docs)
            let rand_offset = util::get_rand_offset().await;
            println!(
                "[{}][PING]: Next ping timer in {}ms (offset {}ms)",
                util::log_time(),
                240000 + rand_offset,
                rand_offset
            );

            // block this thread by waiting on a timer -> a base time of 240,000ms + an
            // offset of < 10,000ms
            time::sleep(time::Duration::from_millis(240000 + rand_offset)).await;

            println!(
                "[{}][PING]: Initiating a keepalive: '{}'",
                util::log_time(),
                KEEPALIVE_PONG
            );

            irc_writer_clone
                .clone()
                .clone()
                .lock()
                .await
                .send(KEEPALIVE_PING.into())
                .await
                .expect("[ERR]: Failed to send keepalive ping.");
        }
    });

    loop {
        if let Some(data) = WebsocketClient::read_socket(&irc_reader).await {
            parse_incoming_irc(data, &irc_writer).await;
        }
    }
}

// determine if we should send a KEEPALIVE or write content to STDOUT
async fn parse_incoming_irc(data: Message, writer: &WriterArc) {
    let data_string = data.to_string(); // let binding extends lifetime to format message data
    let message = data_string
        .trim()
        .split("\r\n")
        .into_iter()
        .map(|l| l.trim())
        .collect::<Vec<_>>();

    match message[0] {
        "PING :tmi.twitch.tv" => {

            // incoming is checking for client pulse
            let res = Message::Text(KEEPALIVE_PONG.into()); // we can also call `Message::Pong()`
            writer.lock().await.send(res).await.expect("[ERR]: Failed while responding to PING.");
            println!("[{}][PING]: Keepalive ack sent", util::log_time());
            return;
        }

        "PONG :tmi.twitch.tv" => {

            // incoming is acknowledging our ping
            println!("[{}][PING]: Received keepalive ack", util::log_time());
            return;
        }
        _ => {

            // incoming is generic (we iterate & indent for readability)
            println!("[{}][INCOMING]: Message: ", util::log_time());
            for line in message {

                println!("   {}", line);
            }

            println!("");
            return;
        }
    }
}
