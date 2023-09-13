mod parser;
pub use parser::*;

mod app_config;
pub use app_config::*;

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub key_value_pairs: Vec<KeyValuePair>,
}

#[derive(Debug, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}
