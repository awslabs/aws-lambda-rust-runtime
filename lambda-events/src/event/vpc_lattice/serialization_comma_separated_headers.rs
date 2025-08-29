use http::{header::HeaderName, HeaderMap, HeaderValue};
use serde::{
    de::{self, Deserializer, Error as DeError, MapAccess, Unexpected, Visitor},
    ser::{Error as SerError, SerializeMap, Serializer},
};
use std::{borrow::Cow, fmt};

/// Implementation detail.
pub(crate) fn deserialize_comma_separated_headers<'de, D>(de: D) -> Result<HeaderMap, D::Error>
where
    D: Deserializer<'de>,
{
    let is_human_readable = de.is_human_readable();
    de.deserialize_option(HeaderMapVisitor { is_human_readable })
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum OneOrMore<'a> {
    One(Cow<'a, str>),
    Strings(Vec<Cow<'a, str>>),
    Bytes(Vec<Cow<'a, [u8]>>),
    CommaSeparated(Cow<'a, str>),
}

struct HeaderMapVisitor {
    is_human_readable: bool,
}

impl<'de> Visitor<'de> for HeaderMapVisitor {
    type Value = HeaderMap;

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
                        // Check if the single value contains commas and split if needed
                        if val.contains(',') {
                            split_and_append_header(&mut map, &key, &val, &self)?;
                        } else {
                            let header_val = val
                                .parse()
                                .map_err(|_| de::Error::invalid_value(Unexpected::Str(&val), &self))?;
                            map.insert(key, header_val);
                        }
                    }
                    OneOrMore::Strings(arr) => {
                        for val in arr {
                            // Each string in the array might also be comma-separated
                            if val.contains(',') {
                                split_and_append_header(&mut map, &key, &val, &self)?;
                            } else {
                                let header_val = val
                                    .parse()
                                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(&val), &self))?;
                                map.append(&key, header_val);
                            }
                        }
                    }
                    OneOrMore::Bytes(arr) => {
                        for val in arr {
                            let header_val = HeaderValue::from_bytes(&val)
                                .map_err(|_| de::Error::invalid_value(Unexpected::Bytes(&val), &self))?;
                            map.append(&key, header_val);
                        }
                    }
                    OneOrMore::CommaSeparated(val) => {
                        // Explicitly handle comma-separated values
                        split_and_append_header(&mut map, &key, &val, &self)?;
                    }
                };
            }
        }
        Ok(map)
    }
}

fn split_and_append_header<E>(
    map: &mut HeaderMap,
    key: &HeaderName,
    value: &str,
    visitor: &HeaderMapVisitor
) -> Result<(), E>
where
    E: DeError,
{
    for split_val in value.split(',') {
        let trimmed_val = split_val.trim();
        if !trimmed_val.is_empty() { // Skip empty values from trailing commas
            let header_val = trimmed_val
                .parse()
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(trimmed_val), visitor))?;
            map.append(key, header_val);
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct FixedString(String);

impl<'de> Deserialize<'de> for FixedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_string(FixedStringVisitor)
    }
}

struct FixedStringVisitor;

impl<'de> Visitor<'de> for FixedStringVisitor {
    type Value = FixedString;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a fixed string \"2.0\"")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if value == "2.0" {
            Ok(FixedString(value.to_owned()))
        } else {
            Err(E::custom(format!("unexpected string value: {}", value)))
        }
    }
}