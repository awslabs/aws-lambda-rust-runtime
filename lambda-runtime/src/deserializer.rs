use std::{error::Error, fmt};

use serde::Deserialize;

use crate::{Context, LambdaEvent};

/// Event payload deserialization error.
/// Returned when the data sent to the function cannot be deserialized
/// into the type that the function receives.
#[derive(Debug)]
pub(crate) struct DeserializeError {
    inner: serde_path_to_error::Error<serde_json::Error>,
}

impl fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.inner.path().to_string();
        if path == "." {
            writeln!(
                f,
                "failed to deserialize the incoming data into the function payload's type: {}",
                self.inner
            )
        } else {
            writeln!(
                f,
                "failed to deserialize the incoming data into the function payload's type: [{}] {}",
                path, self.inner
            )
        }
    }
}

impl Error for DeserializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.inner)
    }
}

/// Deserialize the data sent to the function into the type that the function receives.
pub(crate) fn deserialize<T>(body: &[u8], context: Context) -> Result<LambdaEvent<T>, DeserializeError>
where
    T: for<'de> Deserialize<'de>,
{
    let jd = &mut serde_json::Deserializer::from_slice(body);
    serde_path_to_error::deserialize(jd)
        .map(|payload| LambdaEvent::new(payload, context))
        .map_err(|inner| DeserializeError { inner })
}
