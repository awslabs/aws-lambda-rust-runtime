use serde::{de, ser};
use std::fmt;

use chrono::offset::TimeZone;
use chrono::{DateTime, LocalResult, Utc};

enum SerdeError<V: fmt::Display, D: fmt::Display> {
    NonExistent { timestamp: V },
    Ambiguous { timestamp: V, min: D, max: D },
}

fn ne_timestamp<T: fmt::Display>(ts: T) -> SerdeError<T, u8> {
    SerdeError::NonExistent::<T, u8> { timestamp: ts }
}

impl<V: fmt::Display, D: fmt::Display> fmt::Debug for SerdeError<V, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ChronoSerdeError({})", self)
    }
}

impl<V: fmt::Display, D: fmt::Display> fmt::Display for SerdeError<V, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SerdeError::NonExistent { ref timestamp } => {
                write!(f, "value is not a legal timestamp: {}", timestamp)
            }
            SerdeError::Ambiguous {
                ref timestamp,
                ref min,
                ref max,
            } => write!(
                f,
                "value is an ambiguous timestamp: {}, could be either of {}, {}",
                timestamp, min, max
            ),
        }
    }
}

fn serde_from<T, E, V>(me: LocalResult<T>, ts: &V) -> Result<T, E>
where
    E: de::Error,
    V: fmt::Display,
    T: fmt::Display,
{
    match me {
        LocalResult::None => Err(E::custom(ne_timestamp(ts))),
        LocalResult::Ambiguous(min, max) => Err(E::custom(SerdeError::Ambiguous {
            timestamp: ts,
            min,
            max,
        })),
        LocalResult::Single(val) => Ok(val),
    }
}

struct SecondsFloatTimestampVisitor;

/// Serialize a UTC datetime into an float number of seconds since the epoch
/// ```
pub fn serialize<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    serializer.serialize_i64(dt.timestamp_millis() / 1000)
}

/// Deserialize a `DateTime` from a float seconds timestamp
pub fn deserialize<'de, D>(d: D) -> Result<DateTime<Utc>, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_f64(SecondsFloatTimestampVisitor)
}

impl<'de> de::Visitor<'de> for SecondsFloatTimestampVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a unix timestamp as a float")
    }

    /// Deserialize a timestamp in seconds since the epoch
    fn visit_u64<E>(self, value: u64) -> Result<DateTime<Utc>, E>
    where
        E: de::Error,
    {
        serde_from(Utc.timestamp_opt(value as i64, 0), &value)
    }

    /// Deserialize a timestamp in seconds since the epoch
    fn visit_i64<E>(self, value: i64) -> Result<DateTime<Utc>, E>
    where
        E: de::Error,
    {
        serde_from(Utc.timestamp_opt(value, 0), &value)
    }

    /// Deserialize a timestamp in seconds since the epoch
    fn visit_f64<E>(self, value: f64) -> Result<DateTime<Utc>, E>
    where
        E: de::Error,
    {
        let time_ms = (value.fract() * 1_000_000.).floor() as u32;
        let time_s = value.trunc() as i64;
        serde_from(Utc.timestamp_opt(time_s, time_ms), &value)
    }
}
