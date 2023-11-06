use clap::Parser;
use std::env;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct App {
    #[arg(long, short)]
    pub address: Option<String>,
    #[arg(long, short)]
    pub banlist: Option<String>,
    #[arg(long, short)]
    /// Save the modifications to config files
    pub save: bool,
}

impl App {
    pub fn new() -> Self {
        let mut result = Self::parse();

        if result.address.is_none() {
            result.address =
                Some(env::var("GRPC_RELAY_ADDRESS").unwrap_or("[::1]:50051".to_string()));
        }

        if result.banlist.is_none() {
            result.banlist = Some(env::var("BANLIST").unwrap_or("bans.yaml".to_string()));
        }

        result
    }
}
