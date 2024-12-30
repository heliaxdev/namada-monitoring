use std::str::FromStr;

use anyhow::Context;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRiseWrapper<T: Serialize> {
    pub urls: String,
    #[serde(flatten)]
    pub body: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRiseSlackPayload {
    pub body: String,
}

pub struct AppRise {
    pub url: Url,
    pub client: Client,
    pub slack_webhook_secret: Option<String>,
    pub channel: Option<String>,
}

impl AppRise {
    pub fn new(url: String, slack_secret: Option<String>, slack_channel: Option<String>) -> Self {
        Self {
            url: Url::from_str(&url).unwrap(),
            client: reqwest::Client::new(),
            slack_webhook_secret: slack_secret,
            channel: slack_channel,
        }
    }

    pub async fn send_to_slack(&self, payload: String) -> anyhow::Result<()> {
        let data = AppRiseSlackPayload { body: payload };
        let (token, channel) = if let (Some(token), Some(channel)) =
            (self.slack_webhook_secret.clone(), self.channel.clone())
        {
            (token, channel)
        } else {
            return Ok(());
        };

        let url = self.url.join("/notify").context("Url should be valid")?;
        let payload = AppRiseWrapper {
            urls: format!("slack:///{}/#{}", token, channel),
            body: data,
        };

        self.client
            .post(url)
            .json(&payload)
            .send()
            .await
            .context("Should be able to send slack notification")?;

        Ok(())
    }
}
