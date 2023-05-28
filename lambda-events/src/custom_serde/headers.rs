use http::header::HeaderName;
use http::{HeaderMap, HeaderValue};
use serde::de::{self, Deserializer, Error as DeError, MapAccess, Unexpected, Visitor};
use serde::ser::{Error as SerError, SerializeMap, Serializer};
use std::{borrow::Cow, fmt};

/// Serialize a http::HeaderMap into a serde str => Vec<str> map
pub(crate) fn serialize_multi_value_headers<S>(headers: &HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(headers.keys_len()))?;
    for key in headers.keys() {
        let mut map_values = Vec::new();
        for value in headers.get_all(key) {
            map_values.push(value.to_str().map_err(S::Error::custom)?)
        }
        map.serialize_entry(key.as_str(), &map_values)?;
    }
    map.end()
}

/// Serialize a http::HeaderMap into a serde str => str map
pub(crate) fn serialize_headers<S>(headers: &HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(headers.keys_len()))?;
    for key in headers.keys() {
        let map_value = headers[key].to_str().map_err(S::Error::custom)?;
        map.serialize_entry(key.as_str(), map_value)?;
    }
    map.end()
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum OneOrMore<'a> {
    One(Cow<'a, str>),
    Strings(Vec<Cow<'a, str>>),
    Bytes(Vec<Cow<'a, [u8]>>),
}

struct HeaderMapVisitor {
    is_human_readable: bool,
}

impl<'de> Visitor<'de> for HeaderMapVisitor {
    type Value = HeaderMap;

    // Format a message stating what data this Visitor expects to receive.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("lots of things can go wrong with HeaderMap")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(HeaderMap::default())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(HeaderMap::default())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HeaderMap::with_capacity(access.size_hint().unwrap_or(0));

        if !self.is_human_readable {
            while let Some((key, arr)) = access.next_entry::<Cow<'_, str>, Vec<Cow<'_, [u8]>>>()? {
                let key = HeaderName::from_bytes(key.as_bytes())
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(&key), &self))?;
                for val in arr {
                    let val = HeaderValue::from_bytes(&val)
                        .map_err(|_| de::Error::invalid_value(Unexpected::Bytes(&val), &self))?;
                    map.append(&key, val);
                }
            }
        } else {
            while let Some((key, val)) = access.next_entry::<Cow<'_, str>, OneOrMore<'_>>()? {
                let key = HeaderName::from_bytes(key.as_bytes())
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(&key), &self))?;
                match val {
                    OneOrMore::One(val) => {
                        let val = val
                            .parse()
                            .map_err(|_| de::Error::invalid_value(Unexpected::Str(&val), &self))?;
                        map.insert(key, val);
                    }
                    OneOrMore::Strings(arr) => {
                        for val in arr {
                            let val = val
                                .parse()
                                .map_err(|_| de::Error::invalid_value(Unexpected::Str(&val), &self))?;
                            map.append(&key, val);
                        }
                    }
                    OneOrMore::Bytes(arr) => {
                        for val in arr {
                            let val = HeaderValue::from_bytes(&val)
                                .map_err(|_| de::Error::invalid_value(Unexpected::Bytes(&val), &self))?;
                            map.append(&key, val);
                        }
                    }
                };
            }
        }
        Ok(map)
    }
}

/// Implementation detail.
pub(crate) fn deserialize_headers<'de, D>(de: D) -> Result<HeaderMap, D::Error>
where
    D: Deserializer<'de>,
{
    let is_human_readable = de.is_human_readable();
    de.deserialize_option(HeaderMapVisitor { is_human_readable })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_deserialize_missing_http_headers() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_headers", default)]
            pub headers: HeaderMap,
        }
        let data = serde_json::json!({
            "not_headers": {}
        });

        let expected = HeaderMap::new();

        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.headers);
    }

    #[test]
    fn test_serialize_headers() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_headers", default)]
            #[serde(serialize_with = "serialize_multi_value_headers")]
            headers: HeaderMap,
        }
        let data = serde_json::json!({
            "headers": {
                "Accept": ["*/*"]
            }
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(&"*/*", decoded.headers.get("Accept").unwrap());

        let recoded = serde_json::to_value(decoded).unwrap();
        let decoded: Test = serde_json::from_value(recoded).unwrap();
        assert_eq!(&"*/*", decoded.headers.get("Accept").unwrap());
    }

    #[test]
    fn test_null_headers() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_headers")]
            headers: HeaderMap,
        }
        let data = serde_json::json!({ "headers": null });

        let decoded: Test = serde_json::from_value(data).unwrap();
        assert!(decoded.headers.is_empty());
    }
}
