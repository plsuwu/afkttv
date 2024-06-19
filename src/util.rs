extern crate chrono;
use chrono::prelude::*;


#[allow(dead_code)]
// return the current time in a sensible format for stdout logging
pub async fn log_time() -> String {
    let curr = Utc::now();
    return curr.format("%d.%m@%H:%M:%S").to_string();
}
