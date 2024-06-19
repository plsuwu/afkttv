use futures_util::SinkExt;
use rand::prelude::*;
use tokio::{self, time};
use tokio_tungstenite::tungstenite::protocol::Message;

mod config;
mod util;
mod websocket;
use websocket::{WebsocketClient, WriterArc};

// https://dev.twitch.tv/docs/cli/websocket-event-command/
// https://dev.twitch.tv/docs/pubsub/#connection-management

#[allow(dead_code)] // we want to use this pubsub URL at some point.
const URL_PSUB: &str = "wss://pubsub-edge.twitch.tv/v1";
const URL_CHAT: &str = "wss://irc-ws.chat.twitch.tv/";

const KEEPALIVE_PONG: &str = "PONG";
const KEEPALIVE_PING: &str = "PING";

async fn get_rand_offset() -> u64 {
    let mut rng = thread_rng();
    let offset: f64 = rng.gen();

    return (offset * 10000.0) as u64;
}

#[tokio::main]
async fn main() {
    // automatically-derived `.clone()` on CONFIG_READER
    let config = &config::CONFIG_READER;

    let auth = format!("PASS oauth:{}", config.authorization.auth);
    let nick = format!("NICK {}", config.authorization.user);
    let user = format!(
        "USER {} 8 * :{}", // idk what `8` refers to here as it seems we can connect without it?
        config.authorization.user, config.authorization.user
    );

    // this just joins the user's own channel until i work out how i want to retrieve channel
    // activity
    let join = format!("JOIN #{}", config.authorization.user);
    let irc = websocket::WebsocketClient::new(nick, user, join, auth);
    let (irc_writer, irc_reader) = irc.open_connection(URL_CHAT).await;

    // let evt = websocket::WebsocketClient::new(
    //     // this socket wants json stuff and has many more options so i
    //     // will figure it out later.
    //  );

    //let (evt_writer, evt_reader) = evt.open_connection(URL_PSUB).await;
    let irc_writer_clone = irc_writer.clone();

    tokio::task::spawn(async move {
        loop {
            // add a small random jitter this timer for events sock (see linked docs)
            let rand_offset = get_rand_offset().await;
            println!(
                "[{}][PING]: Next ping timer in {}ms (offset {}ms)",
                util::log_time().await,
                240000 + rand_offset,
                rand_offset
            );

            // block this thread by waiting on a timer -> a base time of 240,000ms + an
            // offset of < 10,000ms
            time::sleep(time::Duration::from_millis(240000 + rand_offset)).await;

            println!(
                "[{}][PING]: Initiating a keepalive: '{}'",
                util::log_time().await,
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

// decide if we need to respond with 'PONG' or if we can write direct to stdout
async fn parse_incoming_irc(data: Message, writer: &WriterArc) {
    let data_string = data.to_string(); // let binding for lifetime wrangling

    let message = data_string
        .trim()
        .split("\r\n")
        .into_iter()
        .map(|l| l.trim())
        .collect::<Vec<_>>();

    // incoming is checking for client pulse
    if message.iter().all(|line| line.contains("PING :tmi.twitch.tv")) {
        let res = Message::Text(KEEPALIVE_PONG.into()); // we can also call `Message::Pong()`
        writer
            .lock()
            .await
            .send(res)
            .await
            .expect("[ERR]: Failed while responding to PING.");
        println!(
            "[{}][PING]: Keepalive response sent",
            util::log_time().await
        );
        return;
    }

    // incoming is acknowledging our ping
    if message.iter().all(|line| line.contains("PONG :tmi.twitch.tv")) {
        println!(
            "[{}][PING]: Received keepalive acknowledge",
            util::log_time().await
        );
        return;
    }

    // incoming is generic (indent for readability)
    println!("[{}][INCOMING]: Message: ", util::log_time().await);
    for line in message {
        println!("   {}", line);
    }
    println!("");

    return;
}
