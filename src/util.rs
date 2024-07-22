use crate::{irc, socket::WriterArc};
use chrono::prelude::*;
use futures_util::SinkExt;
use rand::prelude::*;
use tokio::time;
extern crate chrono;

pub const TMI: &str = ":tmi.twitch.tv";

// #[allow(dead_code)]
pub fn log_time() -> String {
    let curr_time = Local::now();
    return curr_time.format("%d/%m]|[%H:%M:%S").to_string();
}

pub async fn get_rand_offset() -> f64 {
    let mut rng = thread_rng();
    let offset: f64 = rng.gen();

    return offset * 10_000.0;
}

// [21/07]|[22:11:03] [*] Initiating a keepalive: 'PING'
//                    [*] Next ping timer triggers in 242692ms (offset 2692ms)

pub async fn jitter(writer: &WriterArc) {
    let rand_offset = get_rand_offset().await;
    let ping_timer = 240_000 + rand_offset as u64;
    println!(
        "                       Next ping timer set for ~{:.2}mins (offset {:.2}ms)\n",
        (240_000.0 + rand_offset) / 60_000.0,
        rand_offset
    );

    // block this thread by waiting on a timer -> a base time of 240,000ms + an
    // offset in the range 0ms..=10,000ms
    time::sleep(time::Duration::from_millis(ping_timer)).await;

    println!(
        "[{}] [*] Initiating a keepalive: '{}'",
        log_time(),
        irc::KEEPALIVE_PING
    );

    writer
        .clone()
        .lock()
        .await
        .send(irc::KEEPALIVE_PING.into())
        .await
        .expect("[-] Failed to send keepalive ping.");
}
