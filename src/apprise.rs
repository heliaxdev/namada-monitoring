use std::str::FromStr;

use anyhow::Context;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRiseWrapper<T: Serialize> {
    pub urls: String,
    #[serde(flatten)]
    pub body: T
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRiseSlackPayload {
    pub body: String
}

pub struct AppRise {
    pub url: Url,
    pub client: Client,
    pub slack_webhook_secret: String,
    pub channel: String
}

impl AppRise {
    pub fn new(url: String, slack_secret: String, slack_channel: String) -> Self {
        Self {
            url: Url::from_str(&url).unwrap(),
            client: reqwest::Client::new(),
            slack_webhook_secret: slack_secret,
            channel: slack_channel
        }
    }

    pub async fn send_to_slack(&self, payload: &AppRiseSlackPayload) -> anyhow::Result<()> {
        let url = self.url.join("/notify").context("Url should be valid")?;
        let payload = AppRiseWrapper {
            urls: format!("slack:///{}/#{}", self.slack_webhook_secret, self.channel),
            body: payload,
        };
        
        self.client.post(url).json(&payload).send().await.context("Should be able to send slack notification")?;

        Ok(())
    }
}