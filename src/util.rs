use chrono::prelude::*;
use rand::prelude::*;
extern crate chrono;

pub const URL_CHAT: &str = "wss://irc-ws.chat.twitch.tv/";
pub const URL_EVNT: &str = "wss://pubsub-edge.twitch.tv/v1";


#[allow(dead_code)]
// return the current time in a sensible format for stdout logging
pub fn log_time() -> String {
    let curr_time = Local::now();
    return curr_time.format("%d.%m@%H:%M:%S").to_string();
}

// generate a random number from 0 to 10,000
pub async fn get_rand_offset() -> u64 {
    let mut rng = thread_rng();
    let offset: f64 = rng.gen();

    return (offset * 10_000.0) as u64;
}

