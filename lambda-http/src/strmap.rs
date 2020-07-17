use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{
    collections::{hash_map::Keys, HashMap},
    fmt,
    sync::Arc,
};

/// A read-only view into a map of string data which may contain multiple values
///
/// Internally data is always represented as many valued
#[derive(Default, Debug, PartialEq)]
pub struct StrMap(pub(crate) Arc<HashMap<String, Vec<String>>>);

impl StrMap {
    /// Return a named value where available.
    /// If there is more than one value associated with this name,
    /// the first one will be returned
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0
            .get(key)
            .and_then(|values| values.first().map(|owned| owned.as_str()))
    }

    /// Return all values associated with name where available
    pub fn get_all(&self, key: &str) -> Option<Vec<&str>> {
        self.0
            .get(key)
            .map(|values| values.iter().map(|owned| owned.as_str()).collect::<Vec<_>>())
    }

    /// Return true if the underlying map is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return an iterator over keys and values
    pub fn iter(&self) -> StrMapIter<'_> {
        StrMapIter {
            data: self,
            keys: self.0.keys(),
        }
    }
}

impl Clone for StrMap {
    fn clone(&self) -> Self {
        // only clone the inner data
        StrMap(self.0.clone())
    }
}

impl From<HashMap<String, Vec<String>>> for StrMap {
    fn from(inner: HashMap<String, Vec<String>>) -> Self {
        StrMap(Arc::new(inner))
    }
}

/// A read only reference to `StrMap` key and value slice pairings
pub struct StrMapIter<'a> {
    data: &'a StrMap,
    keys: Keys<'a, String, Vec<String>>,
}

impl<'a> Iterator for StrMapIter<'a> {
    type Item = (&'a str, &'a str);

    #[inline]
    fn next(&mut self) -> Option<(&'a str, &'a str)> {
        self.keys.next().and_then(|k| self.data.get(k).map(|v| (k.as_str(), v)))
    }
}

/// internal type used when deserializing StrMaps from
/// potentially one or many valued maps
#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl<'de> Deserialize<'de> for StrMap {
    fn deserialize<D>(deserializer: D) -> Result<StrMap, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrMapVisitor;

        impl<'de> Visitor<'de> for StrMapVisitor {
            type Value = StrMap;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a StrMap")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut inner = map.size_hint().map(HashMap::with_capacity).unwrap_or_else(HashMap::new);
                // values may either be String or Vec<String>
                // to handle both single and multi value data
                while let Some((key, value)) = map.next_entry::<_, OneOrMany>()? {
                    inner.insert(
                        key,
                        match value {
                            OneOrMany::One(one) => vec![one],
                            OneOrMany::Many(many) => many,
                        },
                    );
                }
                Ok(StrMap(Arc::new(inner)))
            }
        }

        deserializer.deserialize_map(StrMapVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn str_map_default_is_empty() {
        assert!(StrMap::default().is_empty())
    }

    #[test]
    fn str_map_get() {
        let mut data = HashMap::new();
        data.insert("foo".into(), vec!["bar".into()]);
        let strmap = StrMap(data.into());
        assert_eq!(strmap.get("foo"), Some("bar"));
        assert_eq!(strmap.get("bar"), None);
    }

    #[test]
    fn str_map_get_all() {
        let mut data = HashMap::new();
        data.insert("foo".into(), vec!["bar".into(), "baz".into()]);
        let strmap = StrMap(data.into());
        assert_eq!(strmap.get_all("foo"), Some(vec!["bar", "baz"]));
        assert_eq!(strmap.get_all("bar"), None);
    }

    #[test]
    fn str_map_iter() {
        let mut data = HashMap::new();
        data.insert("foo".into(), vec!["bar".into()]);
        data.insert("baz".into(), vec!["boom".into()]);
        let strmap = StrMap(data.into());
        let mut values = strmap.iter().map(|(_, v)| v).collect::<Vec<_>>();
        values.sort();
        assert_eq!(values, vec!["bar", "boom"]);
    }
}
