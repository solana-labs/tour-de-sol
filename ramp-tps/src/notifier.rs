use log::*;
use reqwest::Client;
use serde_json::json;
use std::env;

/// For each notification
///   1) Log an info level message
///   2) Notify Slack channel if Slack is configured
///   3) Notify Discord channel if Discord is configured
pub struct Notifier {
    client: Client,
    discord_webhook: Option<String>,
    slack_webhook: Option<String>,
}

impl Notifier {
    pub fn new() -> Self {
        let discord_webhook = env::var("DISCORD_WEBHOOK")
            .map_err(|_| {
                warn!("Discord notifications disabled");
            })
            .ok();
        let slack_webhook = env::var("SLACK_WEBHOOK")
            .map_err(|_| {
                warn!("Slack notifications disabled");
            })
            .ok();
        Notifier {
            client: Client::new(),
            discord_webhook,
            slack_webhook,
        }
    }

    fn send(&self, msg: &str) {
        if let Some(webhook) = &self.discord_webhook {
            let data = json!({ "content": msg });
            if let Err(err) = self.client.post(webhook).json(&data).send() {
                warn!("Failed to send Discord message: {:?}", err);
            }
        }

        if let Some(webhook) = &self.slack_webhook {
            let data = json!({ "text": msg });
            if let Err(err) = self.client.post(webhook).json(&data).send() {
                warn!("Failed to send Slack message: {:?}", err);
            }
        }
    }

    pub fn notify(&self, msg: &str) {
        info!("{}", msg);
        self.send(msg);
    }
}
