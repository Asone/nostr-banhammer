use serde::{Deserialize, Serialize};
use std::fmt;

use num_derive::*;

pub mod grpc {
    include!("nauthz.rs");
    include!("validationcontrol.rs");
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, FromPrimitive)]
pub enum BanTypesEnum {
    CONTENT = 0,
    TAG = 1,
    USER = 2,
    IP = 3,
    NIP05 = 4,
    LUD16 = 5,
}

impl fmt::Display for BanTypesEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let variant = match self {
            BanTypesEnum::CONTENT => "content",
            BanTypesEnum::TAG => "tag",
            BanTypesEnum::USER => "user",
            BanTypesEnum::IP => "ip",
            BanTypesEnum::NIP05 => "nip05",
            BanTypesEnum::LUD16 => "lud16",
        };

        write!(f, "{}", variant)
    }
}
