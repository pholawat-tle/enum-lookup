use regex::Regex;

use crate::{Enum, KeyValuePair, Parser};

pub struct EnumerationParser;

const ENUMERATION_BLOCK_REGEX: &str = r"object\s+(\w+)\s+extends\s+Enumeration\s*\{([\s\S]*?)\}";
const ENUMERATION_VALUES_REGEX: &str = r"val\s+(\w+)[^\r\n]*?=\s*[^\r\n]*?\(([\d]+)([^(\r\n)]*)?\)";

impl Parser for EnumerationParser {
    fn parse_enums(input: &str) -> Vec<Enum> {
        let enum_block_regex = Regex::new(ENUMERATION_BLOCK_REGEX).expect(&format!(
            "Failed to compile regex: {}",
            ENUMERATION_BLOCK_REGEX
        ));
        let enum_values_regex = Regex::new(ENUMERATION_VALUES_REGEX).expect(&format!(
            "Failed to compile regex: {}",
            ENUMERATION_VALUES_REGEX
        ));

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
#[cfg(test)]
mod tests {
    use super::*;

    const SINGLE_ENUM_INPUT: &str = "
        object ProductType extends Enumeration {
          type ProductType = Value

          val Food: ProductType          = Value(1)
          val Cloth: ProductType          = Value(2)
          val Electronic: ProductType        = Value(3, 'Electronic')
        }
    ";

    const MULTIPLE_ENUMS_INPUT: &str = "
        object ProductType extends Enumeration {
          type ProductType = Value

          val Food: ProductType          = Value(1)
          val Cloth: ProductType          = Value(2)
          val Electronic: ProductType        = Value(3)
        }

        object ProductTypeV2 extends Enumeration {
          type ProductTypeV2 = Value

          val Food: ProductTypeV2          = Value(2)
          val Cloth: ProductTypeV2          = Value(3)
          val Electronic: ProductTypeV2        = Value(4)
        }
    ";

    const EMPTY_ENUM_INPUT: &str = "
        object ProductType extends Enumeration {
          type ProductType = Value
        }
    ";

    const NO_ENUM_INPUT: &str = "
        def foo(): Unit = {
          println(\"Hello World!\")
        }
    ";

    #[test]
    fn parse_single_enum() {
        let result = EnumerationParser::parse_enums(SINGLE_ENUM_INPUT);
        assert_eq!(result.len(), 1);

        let first_enum = result.get(0);
        assert_eq!(first_enum.is_some(), true);

        let first_enum = first_enum.unwrap();
        assert_eq!(first_enum.name, "ProductType");
        assert_eq!(first_enum.key_value_pairs.len(), 3);
        assert_eq!(
            first_enum.key_value_pairs,
            vec![
                KeyValuePair {
                    key: "Food".to_string(),
                    value: "1".to_string(),
                },
                KeyValuePair {
                    key: "Cloth".to_string(),
                    value: "2".to_string(),
                },
                KeyValuePair {
                    key: "Electronic".to_string(),
                    value: "3".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parse_multiple_enums() {
        let result = EnumerationParser::parse_enums(MULTIPLE_ENUMS_INPUT);
        assert_eq!(result.len(), 2);

        let first_enum = result.get(0);
        assert_eq!(first_enum.is_some(), true);

        let first_enum = first_enum.unwrap();
        assert_eq!(first_enum.name, "ProductType");
        assert_eq!(first_enum.key_value_pairs.len(), 3);
        assert_eq!(
            first_enum.key_value_pairs,
            vec![
                KeyValuePair {
                    key: "Food".to_string(),
                    value: "1".to_string(),
                },
                KeyValuePair {
                    key: "Cloth".to_string(),
                    value: "2".to_string(),
                },
                KeyValuePair {
                    key: "Electronic".to_string(),
                    value: "3".to_string(),
                },
            ]
        );

        let second_enum = result.get(1);
        assert_eq!(second_enum.is_some(), true);

        let second_enum = second_enum.unwrap();
        assert_eq!(second_enum.name, "ProductTypeV2");
        assert_eq!(second_enum.key_value_pairs.len(), 3);
        assert_eq!(
            second_enum.key_value_pairs,
            vec![
                KeyValuePair {
                    key: "Food".to_string(),
                    value: "2".to_string(),
                },
                KeyValuePair {
                    key: "Cloth".to_string(),
                    value: "3".to_string(),
                },
                KeyValuePair {
                    key: "Electronic".to_string(),
                    value: "4".to_string(),
                },
            ]
        );
    }

    #[test]
    fn parse_empty_enum() {
        let result = EnumerationParser::parse_enums(EMPTY_ENUM_INPUT);
        assert_eq!(result.len(), 1);

        let first_enum = result.get(0);
        assert_eq!(first_enum.is_some(), true);

        let first_enum = first_enum.unwrap();
        assert_eq!(first_enum.name, "ProductType");
        assert_eq!(first_enum.key_value_pairs.len(), 0);
    }

    #[test]
    fn parse_no_enum() {
        let result = EnumerationParser::parse_enums(NO_ENUM_INPUT);
        assert_eq!(result.len(), 0);
    }
}
