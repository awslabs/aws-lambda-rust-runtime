use crate::{
    custom_serde::{
        deserialize_headers, deserialize_lambda_map, deserialize_nullish_boolean, http_method, serialize_headers,
        serialize_multi_value_headers,
    },
    encodings::Body,
    iam::IamPolicyStatement,
};
use http::{HeaderMap, Method};
use query_map::QueryMap;
use serde::{de::DeserializeOwned, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;

/// `VpcLatticeRequest` contains data coming from VPC Lattice service
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcLatticeRequest {
    /// The version of the event structure (always "2.0" for V2)
    #[serde(default)]
    pub version: Option<String>,
    /// The url path for the request
    #[serde(default)]
    pub path: Option<String>,
    /// The HTTP method of the request
    #[serde(with = "http_method")]
    pub method: Method,
    /// HTTP headers of the request (VPC Lattice V2 uses arrays for multi-values)
    #[serde(default, deserialize_with = "deserialize_headers")]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    /// HTTP query string parameters (VPC Lattice V2 uses arrays for multi-values)
    #[serde(
        default,
        deserialize_with = "query_map::serde::aws_api_gateway_v2::deserialize_empty"
    )]
    #[serde(skip_serializing_if = "QueryMap::is_empty")]
    #[serde(serialize_with = "query_map::serde::aws_api_gateway_v2::serialize_query_string_parameters")]
    pub query_string_parameters: QueryMap,
    /// The request body
    #[serde(default)]
    pub body: Option<String>,
    /// Whether the body is base64 encoded
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
    /// VPC Lattice specific request context
    #[serde(bound = "")]
    pub request_context: VpcLatticeRequestContext,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// VPC Lattice specific request context
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcLatticeRequestContext {
    /// ARN of the service network that delivers the request
    #[serde(default)]
    pub service_network_arn: Option<String>,
    /// ARN of the service that receives the request
    #[serde(default)]
    pub service_arn: Option<String>,
    /// ARN of the target group that receives the request
    #[serde(default)]
    pub target_group_arn: Option<String>,
    /// Identity information for the request
    #[serde(default)]
    pub identity: Option<VpcLatticeIdentity>,
    /// AWS region where the request is processed
    #[serde(default)]
    pub region: Option<String>,
    /// Time of the request in microseconds since epoch
    #[serde(default)]
    pub time_epoch: Option<String>,
    /// Catchall for additional context fields
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// Identity information in VPC Lattice request context
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcLatticeIdentity {
    /// ARN of the VPC where the request originated
    #[serde(default)]
    pub source_vpc_arn: Option<String>,
    /// Type of authentication (e.g., "AWS_IAM")
    #[serde(rename = "type")]
    #[serde(default)]
    pub identity_type: Option<String>,
    /// The authenticated principal
    #[serde(default)]
    pub principal: Option<String>,
    /// Organization ID of the authenticated principal
    #[serde(rename = "principalOrgID")]
    #[serde(default)]
    pub principal_org_id: Option<String>,
    /// Name of the authenticated session
    #[serde(default)]
    pub session_name: Option<String>,
    /// X.509 certificate fields (for Roles Anywhere)
    #[serde(rename = "x509IssuerOu")]
    #[serde(default)]
    pub x509_issuer_ou: Option<String>,
    #[serde(rename = "x509SanDns")]
    #[serde(default)]
    pub x509_san_dns: Option<String>,
    #[serde(rename = "x509SanNameCn")]
    #[serde(default)]
    pub x509_san_name_cn: Option<String>,
    #[serde(rename = "x509SanUri")]
    #[serde(default)]
    pub x509_san_uri: Option<String>,
    #[serde(rename = "x509SubjectCn")]
    #[serde(default)]
    pub x509_subject_cn: Option<String>,
    /// Catchall for additional identity fields
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `VpcLatticeResponse` configures the response to be returned by VPC Lattice for the request
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcLatticeResponse {
    /// The HTTP status code for the request
    pub status_code: u16,
    /// The HTTP status description (optional)
    #[serde(default)]
    pub status_description: Option<String>,
    /// The Http headers to return
    #[serde(deserialize_with = "deserialize_headers")]
    #[serde(serialize_with = "serialize_headers")]
    #[serde(skip_serializing_if = "HeaderMap::is_empty")]
    #[serde(default)]
    pub headers: HeaderMap,
    /// The response body
    #[serde(default)]
    pub body: Option<String>,
    /// Whether the body is base64 encoded
    #[serde(default)]
    pub is_base64_encoded: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/*
/// Custom deserializer for VPC Lattice headers (which use arrays for multiple values)
fn deserialize_vpc_lattice_headers<'de, D>(deserializer: D) -> Result<HeaderMap, D::Error>
where
    D: Deserializer<'de>,
{
    let headers: HashMap<String, Vec<String>> = HashMap::deserialize(deserializer)?;
    let mut header_map = HeaderMap::new();

    for (key, values) in headers {
        let header_name = HeaderName::from_str(&key).map_err(serde::de::Error::custom)?;
        for value in values {
            let header_value = HeaderValue::from_str(&value).map_err(serde::de::Error::custom)?;
            header_map.append(header_name.clone(), header_value);
        }
    }

    Ok(header_map)
}

/// Custom serializer for VPC Lattice headers
fn serialize_vpc_lattice_headers<S>(headers: &HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = HashMap::<String, Vec<String>>::new();

    for (name, value) in headers {
        let key = name.as_str().to_string();
        let value_str = value.to_str().unwrap_or("").to_string();
        map.entry(key).or_insert_with(Vec::new).push(value_str);
    }

    map.serialize(serializer)
}

/// Custom deserializer for VPC Lattice query parameters (which use arrays for multiple values)
fn deserialize_vpc_lattice_query_params<'de, D>(deserializer: D) -> Result<QueryMap, D::Error>
where
    D: Deserializer<'de>,
{
    let params: HashMap<String, Vec<String>> = HashMap::deserialize(deserializer)?;
    let mut query_map = QueryMap::new();

    for (key, values) in params {
        query_map.insert(key, values);
    }

    Ok(query_map)
}

fn serialize_vpc_lattice_query_params<S>(params: &QueryMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = HashMap::<String, Vec<String>>::new();

    for (name, value) in params {
        let key = name.as_str().to_string();
        let value_str = value.to_str().unwrap_or("").to_string();
        map.entry(key).or_insert_with(Vec::new).push(value_str);
    }

    map.serialize(serializer)
}

 */