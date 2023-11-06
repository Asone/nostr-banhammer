use std::io::{self, stdin, Write};

use tabled::{Table, Tabled};
use tonic::async_trait;

struct Input;

impl Input {
    pub fn get(label: &str, validators: Option<Vec<fn(String) -> bool>>) -> String {
        let mut data = String::new();
        print!("{}", label);
        _ = io::stdout().flush();
        _ = stdin().read_line(&mut data);
        "".to_string()
    }
}
/// Common trait for sub-handlers.
#[async_trait]
pub trait CommandsHandler {
    // A default helper to get input data from user.
    fn get_input(&self, label: &str, validator: Option<fn(String) -> bool>) -> String {
        let mut data = String::new();
        print!("{}", label);
        _ = io::stdout().flush();
        _ = stdin().read_line(&mut data);

        match validator {
            Some(validator) => match validator(data.clone().trim().to_string()) {
                true => data.trim().to_string(),
                false => {
                    println!("Invalid value provided.");
                    self.get_input(label, Some(validator))
                }
            },
            None => data.trim().to_string(),
        }
    }

    fn print(&self, data: Vec<impl Tabled>) {
        let table = Table::new(data).to_string();
        println!("{}", table);
    }
}

#[derive(Debug, Clone)]
pub struct InputValidator {
    pub validator: fn(String) -> bool,
    pub error_message: Option<&'static str>,
}

pub struct InputValidators {}

impl InputValidators {
    pub const REQUIRED: InputValidator = InputValidator {
        validator: Self::required_input_validator,
        error_message: Some("Value must not be empty"),
    };

    pub fn required_input_validator(value: String) -> bool {
        if value.is_empty() {
            return false;
        }

        true
    }

    pub const DEFAULT_GUARD: InputValidator = InputValidator {
        validator: Self::default_guard_validator,
        error_message: Some("Value cannot be \"default\""),
    };

    pub fn default_guard_validator(value: String) -> bool {
        if value == "default".to_string() {
            return false;
        }

        true
    }

    pub const BAN_TYPE: InputValidator = InputValidator {
        validator: Self::ban_type_validator,
        error_message: Some("Invalid value. Must be one of the following: ip, tag, user, content"),
    };

    pub fn ban_type_validator(value: String) -> bool {
        let ban_types = [
            "ip".to_string(),
            "content".to_string(),
            "tag".to_string(),
            "user".to_string(),
        ]
        .to_vec();
        ban_types.contains(&value)
    }

    pub const BOOLEAN_TYPE: InputValidator = InputValidator {
        validator: Self::boolean_validator,
        error_message: Some(""),
    };

    pub fn boolean_validator(value: String) -> bool {
        let boolean_values = [
            "true".to_string(),
            "false".to_string(),
            "TRUE".to_string(),
            "FALSE".to_string(),
        ]
        .to_vec();
        boolean_values.contains(&value)
    }
}

pub struct InputFormatter {}

impl InputFormatter {
    pub fn input_to_vec(value: String) -> Vec<String> {
        value.split(',').map(|e| e.trim().to_string()).collect()
    }

    pub fn string_nullifier(value: String) -> Option<String> {
        match !value.is_empty() {
            true => Some(value.trim().to_string()),
            false => None,
        }
    }

    pub fn input_to_boolean(value: String) -> bool {
        match value.as_str() {
            "true" => true,
            "false" => false,
            _ => false,
        }
    }

    pub fn input_to_ban_type(value: String) -> i32 {
        match value.as_str() {
            "ip" => 0,
            "content" => 1,
            "word" => 2,
            "user" => 3,
            _ => -1,
        }
    }
}
