use std::{fmt, path::Path};

use banhammer_grpc::grpc::{
    authorization_server::Authorization, event::TagEntry, Decision, Event, EventReply, EventRequest, AddBanRequest,
};
use bech32::{ToBase32, Variant};
use chrono::{NaiveDateTime, Utc};
use regex::Regex;

use serde::{Deserialize, Serialize};

use tonic::{Request, Response, Status};

use banhammer_grpc::BanTypesEnum;
use num_traits::FromPrimitive;
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ban {
    pub content: String,
    pub regex: bool,
    pub reason: Option<String>,
    pub ban_type: BanTypesEnum,
}

impl From<AddBanRequest> for Ban {
    fn from(value: AddBanRequest) -> Self {
        Self {
            content: value.content,
            regex: value.regex,
            reason: value.reason,
            ban_type: FromPrimitive::from_i32(value.ban_type).unwrap()
        }
    }
}

#[derive(Debug, Clone)]
pub struct BanHammer {
    pub words: Vec<Ban>,
    pub tags: Vec<Ban>,
    pub users: Vec<Ban>,
    pub ips: Vec<Ban>,
}

impl Default for BanHammer {
    fn default() -> Self {
        Self {
            words: Vec::new(),
            tags: Vec::new(),
            users: Vec::new(),
            ips: Vec::new(),
        }
    }
}

impl BanHammer {
    pub fn new(path: &str) -> Self {
        let path = Path::new(path);
        Self::default().load(path)
    }

    // Will silently fail if file is not readible.
    // To do : Add panic! instead.
    pub fn load(mut self, path: &Path) -> BanHammer {
        // Load file
        let file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(e) => {
                println!("{}", e.to_string());
                return self;
            }
        };

        // Parse file
        let bans: Vec<Ban> = match serde_yaml::from_reader(file) {
            Ok(r) => r,
            Err(e) => {
                println!("{}", e.to_string());
                return self;
            }
        };

        // Extract content bans
        let ban_words: Vec<Ban> = bans
            .clone()
            .iter()
            .filter(|b| b.ban_type == BanTypesEnum::CONTENT)
            .cloned()
            .collect();

        // Extract tag ban
        let ban_tags: Vec<Ban> = bans
            .clone()
            .iter()
            .filter(|b| b.ban_type == BanTypesEnum::TAG)
            .cloned()
            .collect();

        // Extract user ban
        let ban_users: Vec<Ban> = bans
            .clone()
            .iter()
            .filter(|b| b.ban_type == BanTypesEnum::USER)
            .cloned()
            .collect();

        // Extract IP ban
        let ban_ips: Vec<Ban> = bans
            .clone()
            .iter()
            .filter(|b| b.ban_type == BanTypesEnum::IP)
            .cloned()
            .collect();

        self.words = ban_words;
        self.tags = ban_tags;
        self.ips = ban_ips;
        self.users = ban_users;

        self
    }

    // Helper to format stdout display.
    fn rejection_log_prompt(
        &self,
        ban_type: BanTypesEnum,
        event: Event,
        reply: EventReply,
        ip: String,
    ) -> () {
        let id = hex::encode(event.id).as_str().to_string();

        let created_at = event.created_at*1000;
        let datetime = match NaiveDateTime::from_timestamp_millis(created_at as i64) {
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

    pub fn invalidate_ip(&self, ip: Option<String>) -> bool {
        if ip.is_none() {
            return true;
        };

        let ip = ip.unwrap();

        self.find_in_list(self.ips.clone(), ip)
    }

    pub fn invalidate_content(&self, content: String) -> bool {
        self.find_in_list(self.words.clone(), content)
    }

    pub fn invalidate_user(&self, user: Vec<u8>) -> bool {
        match bech32::encode("npub", user.to_base32(), Variant::Bech32) {
            Ok(user) => self.find_in_list(self.users.clone(), user),
            Err(_) => {
                return true;
            }
        }
    }

    pub fn invalidate_tags(&self, tags: Vec<TagEntry>) -> bool {
        for tag in tags {
            for value in tag.values {
                let r = &self.tags.iter().find(|b| &b.content == &value);

                if r.is_some() {
                    return true;
                }
            }
        }

        return false;
    }

    // Takes a ban list and performs check for content validation
    fn find_in_list(&self, list: Vec<Ban>, content: String) -> bool {
        match list.iter().find(|b| {
            if b.regex == false {
                return b.content.contains(&content);
                // return content == b.content;
            }

            let regex = Regex::new(&b.content);

            if regex.is_err() {
                return false;
            }

            self.regex_search(regex.unwrap(), content.clone())
        }) {
            Some(_) => true,
            None => false,
        }
    }

    // Helper method to perform check when ban is a regex.
    fn regex_search(&self, regex: Regex, content: String) -> bool {
        match regex.captures(&content) {
            Some(_) => {
                return true;
            }
            None => {
                return false;
            }
        }
    }
}

#[tonic::async_trait]
impl Authorization for BanHammer {
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

        // let banhammer = BanHammer::new(&self.dict.as_str());

        if self.invalidate_ip(req.ip_addr.clone()) == true {
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

        if self.invalidate_content(event.content.clone()) == true {
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
        if self.invalidate_user(pubkey) == true {
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

        if self.invalidate_tags(event.tags.clone()) == true {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn _load_sample() -> Vec<Ban> {
        [
            Ban {
                content: "".to_string(),
                regex: false,
                reason: None,
                ban_type: BanTypesEnum::CONTENT,
            },
            Ban {
                content: "".to_string(),
                regex: false,
                reason: None,
                ban_type: BanTypesEnum::IP,
            },
            Ban {
                content: "1.2.3.4".to_string(),
                regex: false,
                reason: None,
                ban_type: BanTypesEnum::IP,
            },
        ]
        .to_vec()
    }

    #[test]
    fn test_banhammer_default() {
        let banhammer = BanHammer::new("");

        assert_eq!(banhammer.ips.len(), 0);
        assert_eq!(banhammer.tags.len(), 0);
        assert_eq!(banhammer.words.len(), 0);
        assert_eq!(banhammer.users.len(), 0);
    }

    #[test]
    fn test_content_invalidation() {
        let mut ban = Ban {
            content: "test".to_string(),
            regex: false,
            reason: Some("test reason".to_string()),
            ban_type: BanTypesEnum::CONTENT,
        };
        let banhammer = BanHammer {
            words: [ban].to_vec(),
            tags: [].to_vec(),
            users: [].to_vec(),
            ips: [].to_vec(),
        };

        let result = banhammer.invalidate_content("test".to_string());
        assert_eq!(true, result);

        let result = banhammer.invalidate_content("lipsum".to_string());
        assert_eq!(false, result);
    }

    #[test]
    fn test_user_invalidation() {
        let mut ban = Ban {
            content: "npub1234".to_string(),
            regex: false,
            reason: Some("test reason".to_string()),
            ban_type: BanTypesEnum::USER,
        };
        let banhammer = BanHammer {
            words: [].to_vec(),
            tags: [].to_vec(),
            users: [ban].to_vec(),
            ips: [].to_vec(),
        };

        let result = banhammer.invalidate_user("test".into());
        assert_eq!(true, result);
    }

    #[test]
    fn test_ip_invalidation() {
        let mut ban = Ban {
            content: "127.0.0.1".to_string(),
            regex: false,
            reason: Some("test reason".to_string()),
            ban_type: BanTypesEnum::IP,
        };
        let banhammer = BanHammer {
            words: [].to_vec(),
            tags: [].to_vec(),
            users: [].to_vec(),
            ips: [ban].to_vec(),
        };
    }

    #[test]
    fn test_tags_invalidation() {
        let mut ban = Ban {
            content: "banhammer".to_string(),
            regex: false,
            reason: Some("test reason".to_string()),
            ban_type: BanTypesEnum::IP,
        };
        let banhammer = BanHammer {
            words: [].to_vec(),
            tags: [ban].to_vec(),
            users: [].to_vec(),
            ips: [].to_vec(),
        };
    }
}
