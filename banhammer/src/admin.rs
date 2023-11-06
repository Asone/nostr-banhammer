use std::sync::Arc;

use banhammer_grpc::{grpc::{
    validation_control_server::ValidationControl, AddBanRequest, AddBanResponse, BanItem,
    ListBansRequest, ListBansResponse, RemoveBanRequest, RemoveBanResponse, StateRequest,
    StateResponse,
}, BanTypesEnum};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::hammer::{Ban, BanHammer};

// #[derive(Debug, Clone)]
pub struct Admin {
    pub banhammer: Arc<Mutex<BanHammer>>,
}

impl From<&Ban> for BanItem {
    fn from(value: &Ban) -> Self {
        Self {
            content: value.content.clone(),
            regex: value.regex,
            reason: value.reason.clone(),
            ban_type: value.ban_type.clone() as i32,
        }
    }
}

#[tonic::async_trait]
impl ValidationControl for Admin {
    async fn add_ban(
        &self,
        request: Request<AddBanRequest>,
    ) -> Result<Response<AddBanResponse>, Status> {

        let ban = Ban::from(request.into_inner());

        let mut banhammer = self.banhammer.lock().await;

        match ban.ban_type {
            BanTypesEnum::CONTENT => {
                banhammer.words.append(&mut [ban].to_vec());
            },
            BanTypesEnum::USER => {
                banhammer.users.append(&mut [ban].to_vec());
            },
            BanTypesEnum::IP => {
                banhammer.ips.append(&mut [ban].to_vec());
            },
            BanTypesEnum::TAG => {
                banhammer.tags.append(&mut [ban].to_vec());
            }
            _ => {}
        }
        Ok(Response::new(AddBanResponse {}))
    }

    async fn list_bans(
        &self,
        request: Request<ListBansRequest>,
    ) -> Result<Response<ListBansResponse>, Status> {
        let banhammer_lock = &self.banhammer.lock().await;

        let bans = match request.into_inner().ban_type {
            0 => banhammer_lock
                .words
                .iter()
                .map(|b| BanItem::from(b))
                .collect(),
            3 => banhammer_lock
                .ips
                .iter()
                .map(|b| BanItem::from(b))
                .collect(),
            2 => banhammer_lock
                .users
                .iter()
                .map(|b| BanItem::from(b))
                .collect(),
            _ => [].to_vec(),
        };

        Ok(Response::new(ListBansResponse { bans }))
    }

    async fn remove_ban(
        &self,
        request: Request<RemoveBanRequest>,
    ) -> Result<Response<RemoveBanResponse>, Status> {
        let banhammer_lock = &self.banhammer.lock().await;

        Ok(Response::new(RemoveBanResponse { result: true }))
    }

    async fn state(
        &self,
        request: Request<StateRequest>,
    ) -> Result<Response<StateResponse>, Status> {
        Ok(Response::new(StateResponse { state: true }))
    }
}
