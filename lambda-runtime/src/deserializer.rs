use std::{error::Error, fmt};

use serde::Deserialize;

use crate::{Context, LambdaEvent};

const ERROR_CONTEXT: &str = "failed to deserialize the incoming data into the function's payload type";

use bytes::Bytes;

#[cfg(not(feature = "simd_json"))]
mod deser_error {
    use super::*;

    /// Event payload deserialization error.
    /// Returned when the data sent to the function cannot be deserialized
    /// into the type that the function receives.
    #[derive(Debug)]
    #[cfg(not(feature = "simd_json"))]
    pub(crate) struct DeserializeError {
        pub(super) inner: serde_path_to_error::Error<aws_lambda_json_impl::JsonError>,
    }
    impl fmt::Display for DeserializeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let path = self.inner.to_string();
            if path == "." {
                writeln!(f, "{ERROR_CONTEXT}: {}", self.inner)
            } else {
                writeln!(f, "{ERROR_CONTEXT}: [{path}] {}", self.inner)
            }
        }
    }

    impl Error for DeserializeError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.inner)
        }
    }
}

#[cfg(feature = "simd_json")]
mod deser_error {
    use super::*;

    /// Event payload deserialization error.
    /// Returned when the data sent to the function cannot be deserialized
    /// into the type that the function receives.
    /// For simd_json, we can't get serde_path_to_error to work at the moment

    #[derive(Debug)]
    pub(crate) struct DeserializeError {
        pub(super) inner: aws_lambda_json_impl::JsonError
    }

    impl fmt::Display for DeserializeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let path = self.inner.to_string();
            if path == "." {
                writeln!(f, "{ERROR_CONTEXT}: {}", self.inner)
            } else {
                writeln!(f, "{ERROR_CONTEXT}: [{path}] {}", self.inner)
            }
        }
    }

    impl Error for DeserializeError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.inner)
        }
    }
}

pub(crate) use deser_error::*;

/// Deserialize the data sent to the function into the type that the function receives.
#[cfg(not(feature = "simd_json"))]
pub(crate) fn deserialize<T>(body: Bytes, context: Context) -> Result<LambdaEvent<T>, DeserializeError>
where
    T: for<'de> Deserialize<'de>,
{
    use aws_lambda_json_impl::JsonDeserializer;
    let jd = &mut JsonDeserializer::from_slice(&body);
    serde_path_to_error::deserialize(jd)
        .map(|payload| LambdaEvent::new(payload, context))
        .map_err(|inner| DeserializeError { inner })
}

#[cfg(feature = "simd_json")]
pub(crate) fn deserialize<T>(body: Bytes, context: Context) -> Result<LambdaEvent<T>, DeserializeError>
where
    T: for<'de> Deserialize<'de>,
{
    //TODO: Find a way to make serde_path_to_json work
    aws_lambda_json_impl::from_bytes(body)
        .map(|payload| LambdaEvent::new(payload, context))
        .map_err(|inner| DeserializeError { inner })
}


