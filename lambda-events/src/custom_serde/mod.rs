#[allow(unused)]
use base64::Engine;
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde;
use serde::de::{Deserialize, Deserializer, Error as DeError};
use serde::ser::Serializer;
use std::collections::HashMap;

#[cfg(feature = "codebuild")]
pub(crate) mod codebuild_time;
#[cfg(feature = "codebuild")]
pub type CodeBuildNumber = f32;

#[cfg(any(
    feature = "alb",
    feature = "apigw",
    feature = "s3",
    feature = "iot",
    feature = "lambda_function_urls"
))]
mod headers;
#[cfg(any(
    feature = "alb",
    feature = "apigw",
    feature = "s3",
    feature = "iot",
    feature = "lambda_function_urls"
))]
pub(crate) use self::headers::*;

#[cfg(feature = "dynamodb")]
pub(crate) mod float_unix_epoch;

#[cfg(any(feature = "alb", feature = "apigw"))]
pub(crate) mod http_method;

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

pub(crate) fn serialize_milliseconds<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ts_with_millis = date.timestamp_millis();
    serializer.serialize_str(&ts_with_millis.to_string())
}

pub(crate) fn deserialize_milliseconds<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
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

pub(crate) fn serialize_seconds<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
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

#[allow(dead_code)]
pub(crate) fn deserialize_seconds<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
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

pub(crate) fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(DeError::custom)
}

pub(crate) fn serialize_base64<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(value))
}

/// Deserializes `HashMap<_>`, mapping JSON `null` to an empty map.
pub(crate) fn deserialize_lambda_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: serde::Deserialize<'de>,
    K: std::hash::Hash,
    K: std::cmp::Eq,
    V: serde::Deserialize<'de>,
{
    // https://github.com/serde-rs/serde/issues/1098
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(feature = "dynamodb")]
/// Deserializes `Item`, mapping JSON `null` to an empty item.
pub(crate) fn deserialize_lambda_dynamodb_item<'de, D>(deserializer: D) -> Result<serde_dynamo::Item, D::Error>
where
    D: Deserializer<'de>,
{
    // https://github.com/serde-rs/serde/issues/1098
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

pub(crate) fn serialize_duration_seconds<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let seconds = duration.num_seconds();

    serializer.serialize_i64(seconds)
}

pub(crate) fn deserialize_duration_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let seconds = f64::deserialize(deserializer)?;
    Ok(Duration::seconds(seconds as i64))
}

pub(crate) fn serialize_duration_minutes<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let minutes = duration.num_minutes();

    serializer.serialize_i64(minutes)
}

pub(crate) fn deserialize_duration_minutes<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let minutes = f64::deserialize(deserializer)?;
    Ok(Duration::minutes(minutes as i64))
}

/// Deserializes `HashMap<_>`, mapping JSON `null` to an empty map.
#[cfg(any(
    feature = "alb",
    feature = "apigw",
    feature = "cloudwatch_events",
    feature = "code_commit",
    test
))]
pub(crate) fn deserialize_nullish_boolean<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    // https://github.com/serde-rs/serde/issues/1098
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use super::*;
    use chrono::TimeZone;
    use serde_json;

    #[test]
    fn test_deserialize_base64() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_base64")]
            v: Vec<u8>,
        }
        let data = json!({
            "v": "SGVsbG8gV29ybGQ=",
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(String::from_utf8(decoded.v).unwrap(), "Hello World".to_string());
    }

    #[test]
    fn test_serialize_base64() {
        #[derive(Serialize)]
        struct Test {
            #[serde(serialize_with = "serialize_base64")]
            v: Vec<u8>,
        }
        let instance = Test {
            v: "Hello World".as_bytes().to_vec(),
        };
        let encoded = serde_json::to_string(&instance).unwrap();
        assert_eq!(encoded, r#"{"v":"SGVsbG8gV29ybGQ="}"#.to_string());
    }

    #[test]
    fn test_deserialize_milliseconds() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_milliseconds")]
            v: DateTime<Utc>,
        }
        let expected = Utc.ymd(2017, 10, 5).and_hms_nano(15, 33, 44, 302_000_000);

        // Test parsing strings.
        let data = json!({
            "v": "1507217624302",
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.v,);
        // Test parsing ints.
        let decoded: Test = serde_json::from_slice(r#"{"v":1507217624302}"#.as_bytes()).unwrap();
        assert_eq!(expected, decoded.v,);
        // Test parsing floats.
        let data = json!({
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
    fn test_deserialize_map() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_lambda_map")]
            v: HashMap<String, String>,
        }
        let input = json!({
          "v": {},
        });
        let decoded: Test = serde_json::from_value(input).unwrap();
        assert_eq!(HashMap::new(), decoded.v);

        let input = json!({
          "v": null,
        });
        let decoded: Test = serde_json::from_value(input).unwrap();
        assert_eq!(HashMap::new(), decoded.v);
    }

    #[cfg(feature = "dynamodb")]
    #[test]
    fn test_deserialize_lambda_dynamodb_item() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_lambda_dynamodb_item")]
            v: serde_dynamo::Item,
        }
        let input = json!({
          "v": {},
        });
        let decoded: Test = serde_json::from_value(input).unwrap();
        assert_eq!(serde_dynamo::Item::from(HashMap::new()), decoded.v);

        let input = json!({
          "v": null,
        });
        let decoded: Test = serde_json::from_value(input).unwrap();
        assert_eq!(serde_dynamo::Item::from(HashMap::new()), decoded.v);
    }

    #[test]
    fn test_deserialize_duration_seconds() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_duration_seconds")]
            v: Duration,
        }

        let expected = Duration::seconds(36);

        let data = json!({
            "v": 36,
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.v,);

        let data = json!({
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

        let data = json!({
            "v": 36,
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(expected, decoded.v,);

        let data = json!({
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

    #[test]
    fn test_deserialize_nullish_boolean() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
            v: bool,
        }

        let test = r#"{"v": null}"#;
        let decoded: Test = serde_json::from_str(test).unwrap();
        assert_eq!(false, decoded.v);

        let test = r#"{}"#;
        let decoded: Test = serde_json::from_str(test).unwrap();
        assert_eq!(false, decoded.v);

        let test = r#"{"v": true}"#;
        let decoded: Test = serde_json::from_str(test).unwrap();
        assert_eq!(true, decoded.v);

        let test = r#"{"v": false}"#;
        let decoded: Test = serde_json::from_str(test).unwrap();
        assert_eq!(false, decoded.v);
    }
}
