use crate::log::LogConfig;

#[derive(clap::Parser)]
pub struct AppConfig {
    #[clap(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    pub tendermint_url: Vec<String>,

    #[clap(short, long, default_value_t = String::from("http://localhost:8000"))]
    pub apprise_url: String,

    #[clap(long, env, default_value_t = 5)]
    pub sleep_for: u64,

    #[clap(flatten)]
    pub log: LogConfig,
}