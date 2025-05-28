use query_map::QueryMap;
use serde::{ser::SerializeMap, Serializer};
use std::collections::HashMap;

/// Serializes `QueryMap`, converting value from `Vec<String>` to `String` using the last value.
pub fn serialize_query_string_parameters<S>(value: &QueryMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut query_string_parameters = HashMap::new();

    if let Some((mut last_key, mut last_value)) = value.iter().next() {
        // insert the last value for each key
        value.iter().for_each(|(k, v)| {
            if k != last_key {
                query_string_parameters.insert(last_key, last_value);
                last_key = k;
            }
            last_value = v;
        });
        // insert the last pair
        query_string_parameters.insert(last_key, last_value);
    }

    let mut map = serializer.serialize_map(Some(query_string_parameters.len()))?;
    for (k, v) in &query_string_parameters {
        map.serialize_entry(k, v)?;
    }
    map.end()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;
    use serde_json::Value;

    #[test]
    fn test_serialize_query_string_parameters() {
        #[derive(Serialize)]
        struct Test {
            #[serde(serialize_with = "serialize_query_string_parameters")]
            pub v: QueryMap,
        }

        fn s(value: &str) -> String {
            value.to_string()
        }

        let query = QueryMap::from(HashMap::from([
            (s("key1"), vec![s("value1"), s("value2"), s("value3")]),
            (s("key2"), vec![s("value4")]),
            (s("key3"), vec![s("value5"), s("value6")]),
        ]));

        let serialized = serde_json::to_string(&Test { v: query }).unwrap();
        let parsed: Value = serde_json::from_str(&serialized).unwrap();

        assert_eq!(parsed["v"]["key1"], Value::String("value3".to_string()));
        assert_eq!(parsed["v"]["key2"], Value::String("value4".to_string()));
        assert_eq!(parsed["v"]["key3"], Value::String("value6".to_string()));
    }
}
