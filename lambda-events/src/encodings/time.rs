use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::ser::Serializer;
use serde::{
    de::{Deserializer, Error as DeError},
    Deserialize, Serialize,
};
use std::ops::{Deref, DerefMut};

/// Timestamp with millisecond precision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MillisecondTimestamp(
    #[serde(deserialize_with = "deserialize_milliseconds")]
    #[serde(serialize_with = "serialize_milliseconds")]
    pub DateTime<Utc>,
);

impl Deref for MillisecondTimestamp {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MillisecondTimestamp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Timestamp with second precision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SecondTimestamp(
    #[serde(deserialize_with = "deserialize_seconds")]
    #[serde(serialize_with = "serialize_seconds")]
    pub DateTime<Utc>,
);

impl Deref for SecondTimestamp {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SecondTimestamp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Duration with second precision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SecondDuration(
    #[serde(deserialize_with = "deserialize_duration_seconds")]
    #[serde(serialize_with = "serialize_duration_seconds")]
    pub Duration,
);

impl Deref for SecondDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SecondDuration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Duration with minute precision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MinuteDuration(
    #[serde(deserialize_with = "deserialize_duration_minutes")]
    #[serde(serialize_with = "serialize_duration_minutes")]
    pub Duration,
);

impl Deref for MinuteDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MinuteDuration {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn serialize_milliseconds<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ts_with_millis = date.timestamp_millis();
    serializer.serialize_str(&ts_with_millis.to_string())
}

fn deserialize_milliseconds<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let (whole, frac) = normalize_timestamp(deserializer)?;
    assert_eq!(frac, 0);
    let seconds: f64 = whole as f64 / 1000.0;
    let milliseconds: u32 = (seconds.fract() * 1000f64) as u32;
    let nanos = milliseconds * 1_000_000;
    Utc.timestamp_opt(seconds as i64, nanos)
        .latest()
        .ok_or_else(|| D::Error::custom("invalid timestamp"))
}

fn serialize_seconds<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let seconds = date.timestamp();
    let milliseconds = date.timestamp_subsec_millis();
    let whole_seconds = seconds + (milliseconds as i64 / 1000);
    let subsec_millis = milliseconds % 1000;
    if milliseconds > 0 {
        let combined = format!("{}.{:03}", whole_seconds, subsec_millis);
        serializer.serialize_str(&combined)
    } else {
        serializer.serialize_str(&whole_seconds.to_string())
    }
}

fn deserialize_seconds<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let (whole, frac) = normalize_timestamp(deserializer)?;
    let seconds = whole;
    let nanos = frac * 1_000_000;
    Utc.timestamp_opt(seconds as i64, nanos as u32)
        .latest()
        .ok_or_else(|| D::Error::custom("invalid timestamp"))
}

fn serialize_duration_seconds<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let seconds = duration.num_seconds();

    serializer.serialize_i64(seconds)
}

fn deserialize_duration_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = f64::deserialize(deserializer)?;
    Ok(Duration::seconds(seconds as i64))
}

fn serialize_duration_minutes<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let minutes = duration.num_minutes();

    serializer.serialize_i64(minutes)
}

fn deserialize_duration_minutes<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let minutes = f64::deserialize(deserializer)?;
    Ok(Duration::minutes(minutes as i64))
}

fn normalize_timestamp<'de, D>(deserializer: D) -> Result<(u64, u64), D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Float(f64),
        Int(u64),
    }

    let input: f64 = match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s.parse::<f64>().map_err(DeError::custom)?,
        StringOrNumber::Float(f) => f,
        StringOrNumber::Int(i) => i as f64,
    };

    // We need to do this due to floating point issues.
    let input_as_string = format!("{}", input);
    let parts: Result<Vec<u64>, _> = input_as_string
        .split('.')
        .map(|x| x.parse::<u64>().map_err(DeError::custom))
        .collect();
    let parts = parts?;
    if parts.len() > 1 {
        Ok((parts[0], parts[1]))
    } else {
        Ok((parts[0], 0))
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use super::*;
    use chrono::TimeZone;
    use serde_json;

    #[test]
    fn test_deserialize_milliseconds() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_milliseconds")]
            v: DateTime<Utc>,
        }
        let expected = Utc.ymd(2017, 10, 5).and_hms_nano(15, 33, 44, 302_000_000);

        // Test parsing strings.
        let data = serde_json::json!({
            "v": "1507217624302",
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.v,);
        // Test parsing ints.
        let decoded: Test = serde_json::from_slice(r#"{"v":1507217624302}"#.as_bytes()).unwrap();
        assert_eq!(expected, decoded.v,);
        // Test parsing floats.
        let data = serde_json::json!({
            "v": 1507217624302.0,
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.v,);
    }

    #[test]
    fn test_serialize_milliseconds() {
        #[derive(Serialize)]
        struct Test {
            #[serde(serialize_with = "serialize_milliseconds")]
            v: DateTime<Utc>,
        }
        let instance = Test {
            v: Utc.ymd(1983, 7, 22).and_hms_nano(1, 0, 0, 99_888_777),
        };
        let encoded = serde_json::to_string(&instance).unwrap();
        assert_eq!(encoded, String::from(r#"{"v":"427683600099"}"#));
    }

    #[test]
    fn test_serialize_seconds() {
        #[derive(Serialize)]
        struct Test {
            #[serde(serialize_with = "serialize_seconds")]
            v: DateTime<Utc>,
        }

        // Make sure nanoseconds are chopped off.
        let instance = Test {
            v: Utc.ymd(1983, 7, 22).and_hms_nano(1, 0, 0, 99),
        };
        let encoded = serde_json::to_string(&instance).unwrap();
        assert_eq!(encoded, String::from(r#"{"v":"427683600"}"#));

        // Make sure milliseconds are included.
        let instance = Test {
            v: Utc.ymd(1983, 7, 22).and_hms_nano(1, 0, 0, 2_000_000),
        };
        let encoded = serde_json::to_string(&instance).unwrap();
        assert_eq!(encoded, String::from(r#"{"v":"427683600.002"}"#));

        // Make sure milliseconds are included.
        let instance = Test {
            v: Utc.ymd(1983, 7, 22).and_hms_nano(1, 0, 0, 1_234_000_000),
        };
        let encoded = serde_json::to_string(&instance).unwrap();
        assert_eq!(encoded, String::from(r#"{"v":"427683601.234"}"#));
    }

    #[test]
    fn test_deserialize_duration_seconds() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_duration_seconds")]
            v: Duration,
        }

        let expected = Duration::seconds(36);

        let data = serde_json::json!({
            "v": 36,
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.v,);

        let data = serde_json::json!({
            "v": 36.1,
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.v,);
    }

    #[test]
    fn test_serialize_duration_seconds() {
        #[derive(Serialize)]
        struct Test {
            #[serde(serialize_with = "serialize_duration_seconds")]
            v: Duration,
        }
        let instance = Test {
            v: Duration::seconds(36),
        };
        let encoded = serde_json::to_string(&instance).unwrap();
        assert_eq!(encoded, String::from(r#"{"v":36}"#));
    }

    #[test]
    fn test_deserialize_duration_minutes() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_duration_minutes")]
            v: Duration,
        }

        let expected = Duration::minutes(36);

        let data = serde_json::json!({
            "v": 36,
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.v,);

        let data = serde_json::json!({
            "v": 36.1,
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.v,);
    }

    #[test]
    fn test_serialize_duration_minutes() {
        #[derive(Serialize)]
        struct Test {
            #[serde(serialize_with = "serialize_duration_minutes")]
            v: Duration,
        }
        let instance = Test {
            v: Duration::minutes(36),
        };
        let encoded = serde_json::to_string(&instance).unwrap();
        assert_eq!(encoded, String::from(r#"{"v":36}"#));
    }
}
