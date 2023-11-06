use banhammer_cli::{CommandsHandler, InputFormatter, InputValidators};
use banhammer_grpc::{
    grpc::{
        validation_control_client::ValidationControlClient, AddBanRequest, BanItem,
        ListBansRequest, RemoveBanRequest, StateRequest,
    },
    BanTypesEnum,
};
use clap::{Parser, ValueEnum};
use tonic::transport::Channel;

use crate::{CliOptions, Subcommands};

#[derive(Tabled)]
struct BanTemplate {
    index: i32,
    content: String,
    regex: bool,
    reason: String,
}

impl From<(usize, &BanItem)> for BanTemplate {
    fn from(value: (usize, &BanItem)) -> Self {
        Self {
            index: value.0 as i32,
            content: value.1.content.clone(),
            regex: value.1.regex,
            reason: value.1.reason.clone().unwrap_or("".to_string()),
        }
    }
}

#[derive(Clone, PartialEq, Parser, Debug, ValueEnum)]
pub enum BanTypeOptionEnum {
    CONTENT = 0,
    TAG = 1,
    USER = 2,
    IP = 3,
    NIP05 = 4,
    LUD16 = 5,
}

use tabled::{Table, Tabled};

/// Global handler for the CLI commands.
/// It provides a dispatcher that will send the command
/// details to sub-handlers
pub struct CliHandler {
    pub client: ValidationControlClient<Channel>,
}

impl CliHandler {
    pub async fn dispatcher(&mut self, command: Subcommands, opts: CliOptions) {
        match command {
            Subcommands::State => {
                let request = tonic::Request::new(StateRequest {});
                let response = self.client.state(request).await;
                match response {
                    Ok(r) => {
                        println!("{}", r.into_inner().state);
                    }
                    Err(e) => {
                        println!("error: {}", e);
                    }
                }
            }
            Subcommands::List { ban_type } => {
                let mut list_handler = ListHandler {
                    client: self.client.clone(),
                };
                list_handler.handle(ban_type).await
            }
            Subcommands::Remove { index, ban_type } => {
                let index = index as u32;
                let ban_type = ban_type as i32;

                let mut remove_handler = RemoveHandler {
                    client: self.client.clone(),
                };

                remove_handler.handle(index, ban_type).await
            }
            Subcommands::Add => {
                let mut add_handler = AddHandler {
                    client: self.client.clone(),
                };

                add_handler.handle().await
            }
            _ => {
                panic!("Oops. Looks like the matrix broke ! :(")
            }
        };
    }
}

pub struct ListHandler {
    client: ValidationControlClient<Channel>,
}

impl CommandsHandler for ListHandler {}

impl ListHandler {
    pub async fn handle(&mut self, ban_type: BanTypeOptionEnum) {
        let ban_type = ban_type as i32;
        let request = tonic::Request::new(ListBansRequest { ban_type });

        let response = self.client.list_bans(request).await;

        match response {
            Ok(response) => {
                let items: Vec<BanItem> = response.into_inner().bans;

                let rows: Vec<BanTemplate> = items
                    .iter()
                    .enumerate()
                    .map(|(i, ban)| BanTemplate::from((i, ban)))
                    .collect();

                if rows.len() == 0 {
                    println!("No ban listed.");
                    return;
                };

                self.print(rows)
            }
            Err(e) => {
                println!("Error | {}: {}", e.code(), e.message());
            }
        }
    }

    fn print(&self, data: Vec<impl Tabled>) {
        let table = Table::new(data).to_string();
        println!("{}", table);
    }
}

pub struct AddHandler {
    client: ValidationControlClient<Channel>,
}

impl CommandsHandler for AddHandler {}

impl AddHandler {
    pub async fn handle(&mut self) {
        // Add cli input here
        let ban_type = InputFormatter::input_to_ban_type(self.get_input(
            "Ban type (content/user/ip/tag) : ",
            Some(InputValidators::ban_type_validator),
        ));
        let regex = InputFormatter::input_to_boolean(self.get_input(
            "Is ban value a regex (true/false): ",
            Some(InputValidators::boolean_validator),
        ));
        let content = self.get_input(
            "Ban value :",
            Some(InputValidators::required_input_validator),
        );
        let reason = self.get_input("Ban reason : ", None);

        let ban = AddBanRequest {
            content,
            regex,
            reason: Some(reason),
            expires_in: None,
            ban_type: ban_type,
        };

        let response = self.client.add_ban(ban).await;

        match response {
            Ok(r) => {}
            Err(e) => {}
        }
    }
}

pub struct RemoveHandler {
    pub client: ValidationControlClient<Channel>,
}

impl CommandsHandler for RemoveHandler {}

impl RemoveHandler {
    pub async fn handle(&mut self, index: u32, ban_type: i32) {
        let request = RemoveBanRequest { index, ban_type };
        let response = self.client.remove_ban(request).await;
        println!("{}", "Ban removed.");
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    fn test_list_handler() {}
}
