use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::sync::Arc;
use tokio::{self, net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

use crate::util::log_time;

// fuck their chungus lives
pub type WriterArc = Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>;
pub type ReaderArc = Arc<Mutex<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>>;

pub struct Client;
impl Client {
    pub async fn write_socket(writer: &WriterArc, data: &str) {
        let msg = Message::Text(data.to_string());
        if !data.contains("oauth:") {
            println!("[{}] [+] Send: '{}'", log_time(), data);
        }

        writer
            .lock()
            .await
            .send(msg)
            .await
            .expect("[-] Failed to send data.");
    }

    pub async fn read_socket(reader: &ReaderArc) -> Option<Message> {
        let mut reader_lock = reader.lock().await;

        if let Some(data) = reader_lock.next().await {
            return Some(data.expect("[-] Incoming data was found, but we couldn't unwrap it."));
        }

        return None;
    }

    // generic func
    pub async fn open_streams(url: &str) -> (WriterArc, ReaderArc) {
        println!(
            "\n[{}] [*] Opening client r/w streams to '{}'...",
            log_time(),
            url
        );
        let (stream, _) = connect_async(url)
            .await
            .expect("[-] Couldn't connect to the socket.");
        println!("[{}] [+] Initial handshake ok.\n", log_time());

        // IO streams -> Arc<Mutex<_>>
        let (writer, reader) = stream.split();
        let writer_arc = Arc::new(Mutex::new(writer));
        let reader_arc = Arc::new(Mutex::new(reader));

        // return streams (+ their ownership) to caller
        return (writer_arc, reader_arc);
    }
}
