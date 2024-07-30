use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use serde_json::{self, json, map::Values, to_string, Value};

use crate::config::CONFIG_READER;

use super::socket::{self, Client, ReaderArc, WriterArc};

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestData {
    topics: Vec<String>,
    auth_token: String,
}

impl RequestData {
    pub fn new(topics: Vec<String>, auth_token: String) -> Self {
        return Self { topics, auth_token };
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeepaliveData {
    r#type: String,
}

impl KeepaliveData {
    pub fn new() -> Self {
        Self {
            r#type: "PING".to_string(),
        }
    }
}

impl ToString for KeepaliveData {
    fn to_string(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    r#type: String,
    nonce: String,
    data: RequestData,
}

impl ToString for Request {
    fn to_string(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }
}

impl Request {
    pub fn new(
        r#type: &str,
        nonce: Option<String>,
        topics: Vec<String>,
        auth_token: String,
    ) -> Self {
        return Self {
            r#type: r#type.to_string(),
            nonce: match nonce {
                Some(data) => data,
                None => Self::generate_nonce(),
            },
            data: RequestData::new(topics, auth_token),
        };
    }

    pub async fn init_event(url: &str, channel: &str, topics: Vec<&str>, auth_token: String) -> (ReaderArc, WriterArc) {
        let (writer, reader) = Client::open_streams(url).await;
        let keepalive = KeepaliveData::new().to_string();
        let initial_msg = Request::new(
            "data",
            None,
            topics
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>(),
            auth_token,
        )
        .to_string();

        Client::write_socket(&writer, &keepalive).await;
        Client::write_socket(&writer, &initial_msg).await;

        return (reader, writer);
    }

    pub fn generate_nonce() -> String {
        return rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(18)
            .map(char::from)
            .collect();
    }
}

pub async fn open_event(url: &str, channel: &str) -> (ReaderArc, WriterArc) {
    let auth = "NOPE";
    let topics = vec!["predictions-user-v1.103033809"];

    let (reader, writer) = Request::init_event(url, channel, topics, auth.to_string()).await;

    return (reader, writer);
}

// pub struct Irc {
//     nick: String,
//     user: String,
//     channel: String,
//     auth: String,
// }
//
// impl Irc {
//     fn new(nick: String, user: String, channel: String, auth: String) -> Self {
//         return Self {
//             nick,
//             user,
//             channel,
//             auth,
//         };
//     }
