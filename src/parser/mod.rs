use regex::Regex;

use crate::{Enum, KeyValuePair};

pub mod scala;

pub trait Parser {
    fn parse_enums(input: &str) -> Vec<Enum>;
}

pub trait RegularExpressionParser {
    fn block_regex(&self) -> &'static str;
    fn values_regex(&self) -> &'static str;

    fn parse(&self, input: &str) -> Vec<Enum> {
        let enum_block_regex = Regex::new(self.block_regex())
            .expect(&format!("Failed to compile regex: {}", self.block_regex()));

        let enum_values_regex = Regex::new(self.values_regex())
            .expect(&format!("Failed to compile regex: {}", self.values_regex()));

        let enum_blocks_iter = enum_block_regex.captures_iter(input);

        let enums_blocks = enum_blocks_iter.filter_map(|enum_block| {
            let name = match enum_block.get(1) {
                Some(name) => name.as_str(),
                None => return None,
            };

            let values_block = match enum_block.get(2) {
                Some(key_value_pairs) => key_value_pairs.as_str(),
                None => return None,
            };

            Some((name, values_block))
        });

        let enums: Vec<Enum> = enums_blocks
            .map(|(name, values_block)| {
                let values_iter = enum_values_regex.captures_iter(values_block);

                let key_value_pairs = values_iter
                    .filter_map(|key_value_pair| {
                        let key = match key_value_pair.get(1) {
                            Some(key) => key.as_str(),
                            None => return None,
                        };

                        let value = match key_value_pair.get(2) {
                            Some(value) => value.as_str(),
                            None => return None,
                        };

                        Some(KeyValuePair {
                            key: key.to_string(),
                            value: value.to_string(),
                        })
                    })
                    .collect();

                Enum {
                    name: name.to_string(),
                    key_value_pairs,
                }
            })
            .collect();

        enums
    }
}
