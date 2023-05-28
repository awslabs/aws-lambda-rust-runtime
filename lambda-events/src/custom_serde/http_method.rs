use http::Method;
use serde::de::{Deserialize, Deserializer, Error as DeError, Unexpected, Visitor};
use serde::ser::Serializer;
use std::fmt;

pub fn serialize<S: Serializer>(method: &Method, ser: S) -> Result<S::Ok, S::Error> {
    ser.serialize_str(method.as_str())
}

struct MethodVisitor;
impl<'de> Visitor<'de> for MethodVisitor {
    type Value = Method;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "valid method name")
    }

    fn visit_str<E: DeError>(self, val: &str) -> Result<Self::Value, E> {
        if val.is_empty() {
            Ok(Method::GET)
        } else {
            val.parse()
                .map_err(|_| DeError::invalid_value(Unexpected::Str(val), &self))
        }
    }
}

pub fn deserialize<'de, D>(de: D) -> Result<Method, D::Error>
where
    D: Deserializer<'de>,
{
    de.deserialize_str(MethodVisitor)
}

pub fn deserialize_optional<'de, D>(deserializer: D) -> Result<Option<Method>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    if let Some(val) = s {
        let visitor = MethodVisitor {};
        return visitor.visit_str(&val).map(Some);
    }

    Ok(None)
}

pub fn serialize_optional<S: Serializer>(method: &Option<Method>, ser: S) -> Result<S::Ok, S::Error> {
    if let Some(method) = method {
        return serialize(method, ser);
    }

    ser.serialize_none()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_http_method_serializer() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            #[serde(with = "crate::custom_serde::http_method")]
            pub method: http::Method,
        }
        let data = serde_json::json!({
            "method": "DELETE"
        });
        let decoded: Test = serde_json::from_value(data.clone()).unwrap();
        assert_eq!(http::Method::DELETE, decoded.method);

        let recoded = serde_json::to_value(decoded).unwrap();
        assert_eq!(data, recoded);
    }

    #[test]
    fn test_http_optional_method_serializer() {
        #[derive(Deserialize, Serialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_optional")]
            #[serde(serialize_with = "serialize_optional")]
            #[serde(default)]
            pub method: Option<http::Method>,
        }
        let data = serde_json::json!({
            "method": "DELETE"
        });
        let decoded: Test = serde_json::from_value(data.clone()).unwrap();
        assert_eq!(Some(http::Method::DELETE), decoded.method);

        let recoded = serde_json::to_value(decoded).unwrap();
        assert_eq!(data, recoded);

        let data = serde_json::json!({ "method": null });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(None, decoded.method);

        let data = serde_json::json!({});
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(None, decoded.method);
    }
}
