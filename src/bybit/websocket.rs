use websocket::client::ClientBuilder;
use websocket::stream::sync::NetworkStream;
use websocket::sync::Client;
use websocket::OwnedMessage;

use std::time::{Duration, Instant};

const PUBLIC_BYBIT_WS_URL: &str = "wss://stream.bybit.com/v5/public/linear";
const PRIVATE_BYBIT_WS_URL: &str = "wss://stream.bybit.com/v5/private";

pub type Subscription = String;

pub struct BybitWebsocket {
    api_key: String,
    api_secret: String,
    ws_url: String,
    subscriptions: Vec<Subscription>,
    ws_client: Client<Box<dyn NetworkStream + Send>>,
    last_ping_sent: Instant,
}

impl BybitWebsocket {
    pub fn new(api_key: &String, api_secret: &String, ws_url: Option<String>, subscriptions: Option<Vec<Subscription>>) -> BybitWebsocket {
        let ws_url = if ws_url.is_some() { ws_url.unwrap() } else { PUBLIC_BYBIT_WS_URL.to_string() };
        let subscriptions = if subscriptions.is_some() { subscriptions.unwrap() } else { Vec::new() };
        let ws_client = ClientBuilder::new(&ws_url).unwrap().connect(None).unwrap();
        
        println!("new called with {}, {}, {}", api_key, api_secret, ws_url);

        BybitWebsocket {
            api_key: api_key.clone(),
            api_secret: api_secret.clone(),
            ws_url,
            subscriptions,
            ws_client,
            last_ping_sent: Instant::now() - Duration::from_secs(20),
        }
    }

    pub fn run<F: FnMut(String)>(&mut self, mut message_handler: F) {

        self.send_subscriptions();

        loop {
            let received = self.ws_client.recv_message();
            if received.is_err() {
                println!("Error receiving message in websocket: {:?}", received);
                self.ws_client = ClientBuilder::new(&self.ws_url).unwrap().connect(None).unwrap();
                println!("Reconnecting...");
                continue;
            }
            let msg = received.unwrap();
            match msg {
                OwnedMessage::Close(_) => break,
                OwnedMessage::Ping(_) => self.ws_client.send_message(&OwnedMessage::Pong(vec![])).unwrap(),
                OwnedMessage::Text(text) => message_handler(text),
                OwnedMessage::Binary(_) => println!("Received: binary data"),
                OwnedMessage::Pong(_) => println!("Received: pong"),
                _ => println!("[OTHER] Received: {:?}", msg)

            }

            if self.last_ping_sent.elapsed() > std::time::Duration::from_secs(20) {
                self.send_ping();
            }
        }
    }

    fn send_ping(&mut self) {
        let str_ping = String::from("{\"op\":\"ping\"}");
        let ping_message = OwnedMessage::Ping(str_ping.into_bytes());

        match self.ws_client.send_message(&ping_message) {
            Ok(()) => {
                self.last_ping_sent = Instant::now();
            }
            Err(e) => println!("Error sending ping: {:?}", e)
        }
    }

    fn send_subscriptions(&mut self) {
        for subscription in &self.subscriptions {
            let subscription_message = OwnedMessage::Text(format!("{{\"op\":\"subscribe\",\"args\": [\"{}\"]}}", subscription));
            self.ws_client.send_message(&subscription_message).unwrap();
        }
    }

}

