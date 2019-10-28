use log::*;
use slack::{Event, EventHandler, RtmClient, Sender};
use std::sync::{Arc, RwLock};
use std::thread;

#[derive(Default)]
pub struct Logger {
    sender: Option<Sender>,
    channel_id: Option<String>,
    connected: Connected,
}

#[derive(Clone, Default)]
struct Connected(Arc<RwLock<bool>>);
impl EventHandler for Connected {
    fn on_event(&mut self, _client: &RtmClient, _event: Event) {}

    fn on_close(&mut self, _client: &RtmClient) {
        debug!("Disconnected from Slack");
        *self.0.write().unwrap() = false;
    }

    fn on_connect(&mut self, _client: &RtmClient) {
        debug!("Connected to Slack");
        *self.0.write().unwrap() = true;
    }
}

impl Logger {
    /// If slack env vars are present, send a slack message for each log call
    /// otherwise, simply log
    pub fn new() -> Self {
        let mut logger = Logger::default();
        let connected = Connected::default();
        logger.sender = std::env::var("SLACK_TOKEN").ok().and_then(|token| {
            let mut sender = None;
            if let Ok(client) = slack::RtmClient::login(&token) {
                debug!("Logging into Slack...");
                sender = Some(client.sender().clone());
                let mut handler = connected.clone();
                thread::spawn(move || client.run(&mut handler));
            }
            sender
        });

        logger.channel_id = std::env::var("SLACK_CHANNEL_ID").ok();
        logger.connected = connected;
        logger
    }

    pub fn connected(&self) -> bool {
        *self.connected.0.read().unwrap()
    }

    fn send(&self, msg: &str) {
        if *self.connected.0.read().unwrap() {
            let sender = self.sender.as_ref().unwrap();
            let channel_id = self.channel_id.as_ref().unwrap();
            if let Err(err) = sender.send_message(channel_id, &format!("`{}`", msg)) {
                warn!("Failed to send slack message: {:?}", err);
            }
        }
    }

    pub fn info(&self, msg: &str) {
        info!("{}", msg);
        self.send(msg);
    }
}
