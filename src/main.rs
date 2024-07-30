pub mod coinflip;
pub mod config;
pub mod websocket;

// pub mod headless;
// use crate::coinflip::flipper;

// use crate::headless;
use crate::websocket::{irc,events, socket::Client, util};
use clap::Parser;
use coinflip::flipper;
use futures_util::{self, future};

pub const URL_CHAT: &str = "wss://irc-ws.chat.twitch.tv/";
pub const URL_EVNT: &str = "wss://pubsub-edge.twitch.tv/v1";

#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "Automatically bet channel points on a coinflip"
)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    irc: bool,

    #[arg(short, long, default_value_t = true)]
    event_edge: bool,

    #[arg(
        short,
        long,
        default_value_t = 100,
        help = "Sets the bet as a percentage of available channel points (whole integer; 0 - 100)."
    )]
    percent: u8,

    #[arg(short, long)]
    channel: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut futures = Vec::new();

    let channel: &str = match &args.channel {
        Some(name) => name,
        // None => "kori",
        None => "plss",
    };


    if args.irc {
        let (irc_reader, irc_writer) = irc::open_irc(URL_CHAT, &channel).await;
        let irc_writer_clone = irc_writer.clone();

        let keepalive = tokio::task::spawn(async move {
            loop {
                util::jitter(&irc_writer_clone).await;
            }
        });
        futures.push(keepalive);

        let irc_client = tokio::task::spawn(async move {
            loop {
                if let Some(data) = Client::read_socket(&irc_reader).await {
                    irc::parse_msg(data, &irc_writer).await;
                }
            }
        });
        futures.push(irc_client);
    }

    if args.event_edge {

        let (event_reader, event_writer) = events::open_event(URL_EVNT, &channel).await;
        let event_writer_clone = event_writer.clone();
        let keepalive_event = tokio::task::spawn(async move {
            loop {
                util::jitter(&event_writer_clone).await;
            }
        });
        futures.push(keepalive_event);

        let event_client = tokio::task::spawn(async move {
            loop {
                if let Some(data) = Client::read_socket(&event_reader).await {
                    // events::parse_msg(data, &event_writer).await;
                    println!("{}", &data);
                }
            }
        });
        futures.push(event_client);

    }

    // futures = vec![keepalive, irc_client];
    let _future_handles = future::join_all(futures).await;

    // tokio::task::spawn(async move {
    //     headless::browser::browser("plss").await.unwrap();
    // });
}
