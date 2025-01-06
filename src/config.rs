use crate::log::LogConfig;

#[derive(clap::Parser)]
pub struct AppConfig {
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    pub cometbft_urls: Vec<String>,

    #[clap(long, env)]
    pub slack_token: Option<String>,

    #[clap(long, env)]
    pub slack_channel: Option<String>,

    #[clap(short, long, default_value_t = String::from("http://localhost:8000"))]
    pub apprise_url: String,

    #[clap(long, env, default_value_t = 9184)]
    pub prometheus_port: u64,

    #[clap(long, env, default_value_t = u64::MAX)]
    pub initial_block_height: u64,

    #[clap(long, env, default_value_t = 5)]
    pub sleep_for: u64,

    #[clap(flatten)]
    pub log: LogConfig,
}
