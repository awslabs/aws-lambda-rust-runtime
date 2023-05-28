use chrono::{DateTime, TimeZone, Utc};
use serde::ser::Serializer;
use serde::{
    de::{Deserializer, Error as DeError, Visitor},
    Deserialize,
};
use std::fmt;

// Jan 2, 2006 3:04:05 PM
const CODEBUILD_TIME_FORMAT: &str = "%b %e, %Y %l:%M:%S %p";

struct TimeVisitor;
impl<'de> Visitor<'de> for TimeVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "valid codebuild time: {}", CODEBUILD_TIME_FORMAT)
    }

    fn visit_str<E: DeError>(self, val: &str) -> Result<Self::Value, E> {
        Utc.datetime_from_str(val, CODEBUILD_TIME_FORMAT)
            .map_err(|e| DeError::custom(format!("Parse error {} for {}", e, val)))
    }
}

pub(crate) mod str_time {
    use super::*;

    pub(crate) fn deserialize<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        d.deserialize_str(TimeVisitor)
    }

    pub fn serialize<S: Serializer>(date: &DateTime<Utc>, ser: S) -> Result<S::Ok, S::Error> {
        let s = format!("{}", date.format(CODEBUILD_TIME_FORMAT));
        ser.serialize_str(&s)
    }
}

pub(crate) mod optional_time {
    use super::*;

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(val) = s {
            let visitor = TimeVisitor {};
            return visitor.visit_str(&val).map(Some);
        }

        Ok(None)
    }

    pub fn serialize<S: Serializer>(date: &Option<DateTime<Utc>>, ser: S) -> Result<S::Ok, S::Error> {
        if let Some(date) = date {
            return str_time::serialize(date, ser);
        }

        ser.serialize_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestTime = DateTime<Utc>;

    #[test]
    fn test_deserialize_codebuild_time() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(with = "str_time")]
            pub date: TestTime,
        }
        let data = serde_json::json!({
            "date": "Sep 1, 2017 4:12:29 PM"
        });

        let expected = Utc
            .datetime_from_str("Sep 1, 2017 4:12:29 PM", CODEBUILD_TIME_FORMAT)
            .unwrap();
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.date);
    }

    #[test]
    fn test_deserialize_codebuild_optional_time() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(with = "optional_time")]
            pub date: Option<TestTime>,
        }
        let data = serde_json::json!({
            "date": "Sep 1, 2017 4:12:29 PM"
        });

        let expected = Utc
            .datetime_from_str("Sep 1, 2017 4:12:29 PM", CODEBUILD_TIME_FORMAT)
            .unwrap();
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(Some(expected), decoded.date);
    }
}
