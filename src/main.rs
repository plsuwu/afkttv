pub mod util;
pub mod irc;
pub mod config;
pub mod socket;

pub const URL_CHAT: &str = "wss://irc-ws.chat.twitch.tv/"; // standard twitch chat WS API
pub const URL_EVNT: &str = "wss://pubsub-edge.twitch.tv/v1"; // subscribes to a 'topic' for
                                                             // updates
#[tokio::main]
async fn main() {
    // irc sock
    irc::open_irc(URL_CHAT).await;
}

// @badge-info=;badges=twitch-recap-2023/1;client-nonce=e89f4edd42f0fa45a92a478315eaf2d0;color=#008000;
// display-name=Froguto;emotes=;first-msg=1;flags=;id=a16ccb0c-319f-4231-936c-a64a82d51075;mod=0;
// returning-chatter=0;room-id=1054004535;subscriber=0;tmi-sent-ts=1721571417416;turbo=0;user-id=81244346;
// user-type= :froguto!froguto@froguto.tmi.twitch.tv PRIVMSG #camizolecorzette :Don't supply them with crimes against humanity please

