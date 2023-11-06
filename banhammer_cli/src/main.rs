mod handler;
use handler::{BanTypeOptionEnum, CliHandler};
use std::{env, process::exit};

use banhammer_grpc::{
    grpc::{validation_control_client::ValidationControlClient, BanType},
    BanTypesEnum,
};
use clap::Parser;
use dotenv::dotenv;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommands,
    #[arg(long, short)]
    /// Save the modifications to config files
    pub save: bool,
}

#[derive(Debug, Default)]
pub struct CliOptions {
    save: bool,
}

#[derive(Debug, PartialEq, Parser)]
pub enum Subcommands {
    #[clap(
        name = "List",
        about = "Provides commands for nostr-rs-relay banhammer",
        long_about = r#""#
    )]
    List {
        ban_type: BanTypeOptionEnum,
    },

    State,
    Add,
    Remove {
        index: i32,
        ban_type: BanTypeOptionEnum,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let grpc_address = env::var("GRPC_RELAY_ADDRESS").unwrap_or("[::1]:50051".to_string());
    let grpc_full_address = format!("{}{}", "http://", grpc_address);

    // Creates the gRPC client
    let client = match ValidationControlClient::connect(grpc_full_address).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Could not connect to core service. Are you sure it is up ?");
            panic!("{}", e);
        }
    };
    // Get CLI arguments and parameters
    let cli = Cli::parse();

    let opts = CliOptions {
        save: cli.save,
        ..Default::default()
    };
    let mut handler = CliHandler { client };
    handler.dispatcher(cli.subcommand, opts).await;

    exit(1);
}
