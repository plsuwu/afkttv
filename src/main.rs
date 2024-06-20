pub mod util;
pub mod websocket;
pub mod config;

#[tokio::main]
async fn main() {
    // irc sock
    websocket::open_conn(util::URL_CHAT).await;
}
