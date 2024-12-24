pub mod apprise;
pub mod config;
pub mod log;
pub mod state;

use std::{thread::sleep, time::Duration};

use apprise::{AppRise, AppRiseSlackPayload};
use clap::Parser;
use config::AppConfig;

#[tokio::main]
async fn main() {
    let config = AppConfig::parse();

    // let apprise = AppRise::new(
    //     config.apprise_url,
    //     "T01J3DMLWUW/B086DJEEQKD/pNDFKSQIiiEkMmqrCtKOuzTr".to_string(),
    //     "namada-alerts".to_string(),
    // );

    // let slack_payload = AppRiseSlackPayload {
    //     body: "test".to_string(),
    // };
    // apprise.send_to_slack(&slack_payload).await.unwrap();

    loop {
        println!("Test...");
        sleep(Duration::from_secs(3));
    }
}
