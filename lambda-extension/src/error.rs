/// Error type that extensions may result in
pub type Error = lambda_runtime_api_client::Error;

/// Simple error that encapsulates human readable descriptions
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtensionError {
    err: String,
}

impl ExtensionError {
    pub(crate) fn boxed<T: Into<String>>(str: T) -> Box<ExtensionError> {
        Box::new(ExtensionError { err: str.into() })
    }
}

impl std::fmt::Display for ExtensionError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl std::error::Error for ExtensionError {}
