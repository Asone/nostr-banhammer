use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use banhammer_grpc::grpc::authorization_server::{Authorization, AuthorizationServer};
use banhammer_grpc::grpc::validation_control_server::ValidationControlServer;
use banhammer_grpc::grpc::{Decision, Event, EventReply, EventRequest};
use banhammer_grpc::BanTypesEnum;
use bech32::{ToBase32, Variant};
use chrono::{DateTime, NaiveDateTime, Utc};
use clap::Parser;

use tokio::sync::Mutex;
use tonic::{transport::Server, Request, Response, Status};

use dotenv::dotenv;

use crate::admin::Admin;
use crate::app::App;
use crate::hammer::BanHammer;

mod admin;
mod app;
mod hammer;

#[derive(Default)]
pub struct EventAuthz {
    addr: String,
    dict: String,
}

impl EventAuthz {
    pub fn new() -> Self {
        Self {
            addr: env::var("GRPC_RELAY_ADDRESS").unwrap_or("[::1]:50051".to_string()),
            dict: env::var("BANLIST").unwrap_or("bans.yaml".to_string()),
        }
    }

    fn rejection_log_prompt(
        &self,
        ban_type: BanTypesEnum,
        event: Event,
        reply: EventReply,
        ip: String,
    ) -> () {
        let id = hex::encode(event.id).as_str().to_string();

        let datetime = match NaiveDateTime::from_timestamp_millis(event.created_at as i64) {
            Some(v) => v,
            None => Utc::now().naive_utc(),
        };

        let pubkey = match bech32::encode("npub", event.pubkey.to_base32(), Variant::Bech32) {
            Ok(v) => v,
            Err(_) => "".to_string(),
        };

        println!("REJECTED | {} | {{\"event\": {} , \"ip\" : \"{}\" , \"type\": \"{}\", \"pubkey\": \"{}\" }}",
            datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
            id,
            ip,
            ban_type,
            pubkey
            );
    }
}

#[tonic::async_trait]
impl Authorization for EventAuthz {
    async fn event_admit(
        &self,
        request: Request<EventRequest>,
    ) -> Result<Response<EventReply>, Status> {
        let mut reply;
        let req: EventRequest = request.into_inner();
        let event = req.event.unwrap();

        reply = EventReply {
            decision: Decision::Permit as i32,
            message: None,
        };

        let banhammer = BanHammer::new(&self.dict.as_str());

        if banhammer.invalidate_ip(req.ip_addr.clone()) == true {
            reply = EventReply {
                decision: Decision::Deny as i32,
                message: None,
            };

            _ = &self.rejection_log_prompt(
                BanTypesEnum::IP,
                event.clone(),
                reply.clone(),
                req.ip_addr.clone().unwrap_or("".to_string()),
            );
        }

        if banhammer.invalidate_content(event.content.clone()) == true {
            reply = EventReply {
                decision: Decision::Deny as i32,
                message: None,
            };

            _ = &self.rejection_log_prompt(
                BanTypesEnum::CONTENT,
                event.clone(),
                reply.clone(),
                req.ip_addr.clone().unwrap_or("".to_string()),
            );
        }

        let pubkey = event.pubkey.clone();
        if banhammer.invalidate_user(pubkey) == true {
            reply = EventReply {
                decision: Decision::Deny as i32,
                message: None,
            };

            _ = &self.rejection_log_prompt(
                BanTypesEnum::USER,
                event.clone(),
                reply.clone(),
                req.ip_addr.clone().unwrap_or("".to_string()),
            );
        }

        if banhammer.invalidate_tags(event.tags.clone()) == true {
            reply = EventReply {
                decision: Decision::Deny as i32,
                message: None,
            };

            _ = &self.rejection_log_prompt(
                BanTypesEnum::TAG,
                event.clone(),
                reply.clone(),
                req.ip_addr.clone().unwrap_or("".to_string()),
            );
        }

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let app = App::new();

    let ban_list = match app.banlist {
        Some(value) => value,
        None => env::var("GRPC_RELAY_ADDRESS").unwrap_or("[::1]:50051".to_string()),
    };

    let checker = BanHammer::new(&ban_list);
    let checker_arc = Arc::new(Mutex::new(checker.clone()));
    let admin = Admin {
        banhammer: checker_arc,
    };

    println!(
        "Validation Server listening on {}",
        app.address.clone().unwrap()
    );
    // Start serving
    Server::builder()
        .add_service(ValidationControlServer::new(admin))
        .add_service(AuthorizationServer::new(checker))
        .serve(app.address.clone().unwrap().parse().unwrap())
        .await?;
    Ok(())
}

pub struct AuthorizationServerConfig {
    addr: String,
    dict: String,
}
