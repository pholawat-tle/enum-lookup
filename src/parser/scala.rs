use crate::{Enum, Parser, RegularExpressionParser};

pub struct EnumerationParser;

const ENUMERATION_BLOCK_REGEX: &str = r"object\s+(\w+)\s+extends\s+Enumeration\s*\{([\s\S]*?)\}";
const ENUMERATION_VALUES_REGEX: &str = r"val\s+(\w+)[^\r\n]*?=\s*[^\r\n]*?\(([\d]+)([^(\r\n)]*)?\)";

impl RegularExpressionParser for EnumerationParser {
    fn block_regex(&self) -> &'static str {
        ENUMERATION_BLOCK_REGEX
    }

    fn values_regex(&self) -> &'static str {
        ENUMERATION_VALUES_REGEX
    }
}

impl Parser for EnumerationParser {
    fn parse_enums(input: &str) -> Vec<Enum> {
        EnumerationParser.parse(input)
    }
}

pub struct CaseObjectParser;

const CASE_OBJECT_BLOCK_REGEX: &str = r"object\s+(\w+)\s+extends\s+Enum\[(?:.*)\]\s*\{([\s\S]*?)\}";
const CASE_OBJECT_VALUES_REGEX: &str =
    r"case object\s+(\w+)[^\r\n]*?extends\s*[^\r\n]*?\(([\d]+)([^(\r\n)]*)?\)";

impl RegularExpressionParser for CaseObjectParser {
    fn block_regex(&self) -> &'static str {
        CASE_OBJECT_BLOCK_REGEX
    }

    fn values_regex(&self) -> &'static str {
        CASE_OBJECT_VALUES_REGEX
    }
}

impl Parser for CaseObjectParser {
    fn parse_enums(input: &str) -> Vec<Enum> {
        CaseObjectParser.parse(input)
    }
}

#[cfg(test)]
mod tests {
    use crate::KeyValuePair;

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

    const CASE_OBJECT_ENUM_INPUT: &str = "
        object FoodEntries extends Enum[FoodEntry] {
          val values                          = findValues.toIndexedSeq
          val fields: Map[Int, FoodEntry]     = values.map(v => (v.i, v)).toMap
          lazy val getValue: Int => FoodEntry = fields.getOrElse(_, FoodEntries.Unknown)

          case object Unknown extends FoodEntry(0)
          case object Single  extends FoodEntry(1)
          case object Bundle  extends FoodEntry(2)
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
    fn enumeration_parser_parse_single_enum() {
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
    fn enumeration_parser_parse_multiple_enums() {
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
    fn enumeration_parser_parse_case_object_enum() {
        let result = EnumerationParser::parse_enums(CASE_OBJECT_ENUM_INPUT);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn enumeration_parser_parse_empty_enum() {
        let result = EnumerationParser::parse_enums(EMPTY_ENUM_INPUT);
        assert_eq!(result.len(), 1);

        let first_enum = result.get(0);
        assert_eq!(first_enum.is_some(), true);

        let first_enum = first_enum.unwrap();
        assert_eq!(first_enum.name, "ProductType");
        assert_eq!(first_enum.key_value_pairs.len(), 0);
    }

    #[test]
    fn enumeration_parser_parse_no_enum() {
        let result = EnumerationParser::parse_enums(NO_ENUM_INPUT);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn case_object_parser_parse_single_enum() {
        let result = CaseObjectParser::parse_enums(SINGLE_ENUM_INPUT);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn case_object_parser_parse_multiple_enums() {
        let result = CaseObjectParser::parse_enums(MULTIPLE_ENUMS_INPUT);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn case_object_parser_parse_case_object_enum() {
        let result = CaseObjectParser::parse_enums(CASE_OBJECT_ENUM_INPUT);
        assert_eq!(result.len(), 1);

        let first_enum = result.get(0);
        assert_eq!(first_enum.is_some(), true);

        let first_enum = first_enum.unwrap();
        assert_eq!(first_enum.name, "FoodEntries");
        assert_eq!(first_enum.key_value_pairs.len(), 3);
        assert_eq!(
            first_enum.key_value_pairs,
            vec![
                KeyValuePair {
                    key: "Unknown".to_string(),
                    value: "0".to_string(),
                },
                KeyValuePair {
                    key: "Single".to_string(),
                    value: "1".to_string(),
                },
                KeyValuePair {
                    key: "Bundle".to_string(),
                    value: "2".to_string(),
                },
            ]
        );
    }

    #[test]
    fn case_object_parser_parse_empty_enum() {
        let result = CaseObjectParser::parse_enums(EMPTY_ENUM_INPUT);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn case_object_parser_parse_no_enum() {
        let result = CaseObjectParser::parse_enums(NO_ENUM_INPUT);
        assert_eq!(result.len(), 0);
    }
}
