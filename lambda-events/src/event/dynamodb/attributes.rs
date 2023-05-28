use base64::Engine;
use serde_dynamo::AttributeValue;
use std::collections::HashMap;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_null_attribute() {
        let value = serde_json::json!({
            "NULL": true
        });

        let attr: AttributeValue = serde_json::from_value(value.clone()).unwrap();
        match attr {
            AttributeValue::Null(true) => {}
            other => panic!("unexpected value {:?}", other),
        }

        let reparsed = serde_json::to_value(attr).unwrap();
        assert_eq!(value, reparsed);
    }

    #[test]
    fn test_string_attribute() {
        let value = serde_json::json!({
            "S": "value"
        });

        let attr: AttributeValue = serde_json::from_value(value.clone()).unwrap();
        match attr {
            AttributeValue::S(ref s) => assert_eq!("value", s.as_str()),
            other => panic!("unexpected value {:?}", other),
        }

        let reparsed = serde_json::to_value(attr).unwrap();
        assert_eq!(value, reparsed);
    }

    #[test]
    fn test_number_attribute() {
        let value = serde_json::json!({
            "N": "123.45"
        });

        let attr: AttributeValue = serde_json::from_value(value.clone()).unwrap();
        match attr {
            AttributeValue::N(ref n) => assert_eq!("123.45", n.as_str()),
            other => panic!("unexpected value {:?}", other),
        }

        let reparsed = serde_json::to_value(attr).unwrap();
        assert_eq!(value, reparsed);
    }

    #[test]
    fn test_binary_attribute() {
        let value = serde_json::json!({
            "B": "dGhpcyB0ZXh0IGlzIGJhc2U2NC1lbmNvZGVk"
        });

        let attr: AttributeValue = serde_json::from_value(value.clone()).unwrap();
        match attr {
            AttributeValue::B(ref b) => {
                let expected = base64::engine::general_purpose::STANDARD
                    .decode("dGhpcyB0ZXh0IGlzIGJhc2U2NC1lbmNvZGVk")
                    .unwrap();
                assert_eq!(&expected, b)
            }
            other => panic!("unexpected value {:?}", other),
        }

        let reparsed = serde_json::to_value(attr).unwrap();
        assert_eq!(value, reparsed);
    }

    #[test]
    fn test_boolean_attribute() {
        let value = serde_json::json!({
            "BOOL": true
        });

        let attr: AttributeValue = serde_json::from_value(value.clone()).unwrap();
        match attr {
            AttributeValue::Bool(b) => assert_eq!(true, b),
            other => panic!("unexpected value {:?}", other),
        }

        let reparsed = serde_json::to_value(attr).unwrap();
        assert_eq!(value, reparsed);
    }

    #[test]
    fn test_string_set_attribute() {
        let value = serde_json::json!({
            "SS": ["Giraffe", "Hippo" ,"Zebra"]
        });

        let attr: AttributeValue = serde_json::from_value(value.clone()).unwrap();
        match attr {
            AttributeValue::Ss(ref s) => {
                let expected = vec!["Giraffe", "Hippo", "Zebra"];
                assert_eq!(expected, s.iter().collect::<Vec<_>>());
            }
            other => panic!("unexpected value {:?}", other),
        }

        let reparsed = serde_json::to_value(attr).unwrap();
        assert_eq!(value, reparsed);
    }

    #[test]
    fn test_number_set_attribute() {
        let value = serde_json::json!({
            "NS": ["42.2", "-19", "7.5", "3.14"]
        });

        let attr: AttributeValue = serde_json::from_value(value.clone()).unwrap();
        match attr {
            AttributeValue::Ns(ref s) => {
                let expected = vec!["42.2", "-19", "7.5", "3.14"];
                assert_eq!(expected, s.iter().collect::<Vec<_>>());
            }
            other => panic!("unexpected value {:?}", other),
        }

        let reparsed = serde_json::to_value(attr).unwrap();
        assert_eq!(value, reparsed);
    }

    #[test]
    fn test_binary_set_attribute() {
        let value = serde_json::json!({
            "BS": ["U3Vubnk=", "UmFpbnk=", "U25vd3k="]
        });

        let attr: AttributeValue = serde_json::from_value(value.clone()).unwrap();
        match attr {
            AttributeValue::Bs(ref s) => {
                let expected = vec!["U3Vubnk=", "UmFpbnk=", "U25vd3k="]
                    .into_iter()
                    .flat_map(|s| base64::engine::general_purpose::STANDARD.decode(s))
                    .collect::<Vec<_>>();
                assert_eq!(&expected, s);
            }
            other => panic!("unexpected value {:?}", other),
        }

        let reparsed = serde_json::to_value(attr).unwrap();
        assert_eq!(value, reparsed);
    }

    #[test]
    fn test_attribute_list_attribute() {
        let value = serde_json::json!({
            "L": [ {"S": "Cookies"} , {"S": "Coffee"}, {"N": "3.14159"}]
        });

        let attr: AttributeValue = serde_json::from_value(value.clone()).unwrap();
        match attr {
            AttributeValue::L(ref s) => {
                let expected = vec![
                    AttributeValue::S("Cookies".into()),
                    AttributeValue::S("Coffee".into()),
                    AttributeValue::N("3.14159".into()),
                ];
                assert_eq!(&expected, s);
            }
            other => panic!("unexpected value {:?}", other),
        }

        let reparsed = serde_json::to_value(attr).unwrap();
        assert_eq!(value, reparsed);
    }

    #[test]
    fn test_attribute_map_attribute() {
        let value = serde_json::json!({
            "M": {"Name": {"S": "Joe"}, "Age": {"N": "35"}}
        });

        let attr: AttributeValue = serde_json::from_value(value).unwrap();
        match attr {
            AttributeValue::M(s) => {
                let mut expected = HashMap::new();
                expected.insert("Name".into(), AttributeValue::S("Joe".into()));
                expected.insert("Age".into(), AttributeValue::N("35".into()));
                assert_eq!(expected, s);
            }
            other => panic!("unexpected value {:?}", other),
        }
    }
}
