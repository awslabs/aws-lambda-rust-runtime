use failure::{Backtrace, Context, Fail};
use lambda_runtime_errors::LambdaErrorExt;
use std::fmt;

#[derive(LambdaErrorExt)]
struct BasicCustomError;

#[derive(Fail, LambdaErrorExt, Debug)]
#[fail(display = "Input was invalid UTF-8")]
struct FailureCustomError;

#[derive(Debug, LambdaErrorExt)]
struct FailureCustomWithKind {
    inner: Context<FailureErrorKind>,
}

#[derive(Clone, Eq, PartialEq, Debug, Fail, LambdaErrorExt)]
enum FailureErrorKind {
    #[fail(display = "First contextual error message.")]
    FirstVariant,
    #[fail(display = "Second contextual error message: {}.", _0)]
    SecondVariant(String),
}

impl Fail for FailureCustomWithKind {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl fmt::Display for FailureCustomWithKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inner, f)
    }
}

impl FailureCustomWithKind {
    pub fn kind(&self) -> FailureErrorKind {
        self.inner.get_context().clone()
    }
}

impl From<FailureErrorKind> for FailureCustomWithKind {
    fn from(kind: FailureErrorKind) -> Self {
        FailureCustomWithKind {
            inner: Context::new(kind),
        }
    }
}

#[test]
fn simple_error_type() {
    let err = BasicCustomError {};
    assert_eq!(
        err.error_type(),
        "BasicCustomError",
        "Custom error not implemented correctly"
    );
}

#[test]
fn fail_custom_error() {
    let err = FailureCustomError {};
    assert_eq!(err.error_type(), "FailureCustomError", "Error type wrong")
}

#[test]
fn fail_variant_first() {
    let err = FailureCustomWithKind::from(FailureErrorKind::FirstVariant);
    //assert_eq!(err.error_type(), "FailureCustomError", "Error type wrong")
}
