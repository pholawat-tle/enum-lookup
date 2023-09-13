use crate::Enum;

pub mod scala;

pub trait Parser {
    fn parse_enums(input: &str) -> Vec<Enum>;
}
