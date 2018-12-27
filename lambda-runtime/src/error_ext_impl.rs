// Generated code, DO NOT MODIFY!

use error::LambdaErrorExt;
use std::{
    alloc::LayoutErr,
    cell::{BorrowError, BorrowMutError},
    char::{DecodeUtf16Error, ParseCharError},
    env::{JoinPathsError, VarError},
    ffi::{FromBytesWithNulError, IntoStringError, NulError},
    net::AddrParseError,
    num::{ParseFloatError, ParseIntError},
    path::StripPrefixError,
    str::{ParseBoolError, Utf8Error},
    string::{FromUtf16Error, FromUtf8Error, ParseError},
    sync::mpsc::{RecvError, RecvTimeoutError, TryRecvError},
    time::SystemTimeError,
};

impl LambdaErrorExt for VarError {
    fn error_type(&self) -> &str {
        "VarError"
    }
}
impl LambdaErrorExt for ParseError {
    fn error_type(&self) -> &str {
        "ParseError"
    }
}
impl LambdaErrorExt for RecvTimeoutError {
    fn error_type(&self) -> &str {
        "RecvTimeoutError"
    }
}
impl LambdaErrorExt for TryRecvError {
    fn error_type(&self) -> &str {
        "TryRecvError"
    }
}
impl LambdaErrorExt for LayoutErr {
    fn error_type(&self) -> &str {
        "LayoutErr"
    }
}
impl LambdaErrorExt for BorrowError {
    fn error_type(&self) -> &str {
        "BorrowError"
    }
}
impl LambdaErrorExt for BorrowMutError {
    fn error_type(&self) -> &str {
        "BorrowMutError"
    }
}
impl LambdaErrorExt for DecodeUtf16Error {
    fn error_type(&self) -> &str {
        "DecodeUtf16Error"
    }
}
impl LambdaErrorExt for ParseCharError {
    fn error_type(&self) -> &str {
        "ParseCharError"
    }
}
impl LambdaErrorExt for JoinPathsError {
    fn error_type(&self) -> &str {
        "JoinPathsError"
    }
}
impl LambdaErrorExt for FromBytesWithNulError {
    fn error_type(&self) -> &str {
        "FromBytesWithNulError"
    }
}
impl LambdaErrorExt for IntoStringError {
    fn error_type(&self) -> &str {
        "IntoStringError"
    }
}
impl LambdaErrorExt for NulError {
    fn error_type(&self) -> &str {
        "NulError"
    }
}
impl LambdaErrorExt for AddrParseError {
    fn error_type(&self) -> &str {
        "AddrParseError"
    }
}
impl LambdaErrorExt for ParseFloatError {
    fn error_type(&self) -> &str {
        "ParseFloatError"
    }
}
impl LambdaErrorExt for ParseIntError {
    fn error_type(&self) -> &str {
        "ParseIntError"
    }
}
impl LambdaErrorExt for StripPrefixError {
    fn error_type(&self) -> &str {
        "StripPrefixError"
    }
}
impl LambdaErrorExt for ParseBoolError {
    fn error_type(&self) -> &str {
        "ParseBoolError"
    }
}
impl LambdaErrorExt for Utf8Error {
    fn error_type(&self) -> &str {
        "Utf8Error"
    }
}
impl LambdaErrorExt for FromUtf16Error {
    fn error_type(&self) -> &str {
        "FromUtf16Error"
    }
}
impl LambdaErrorExt for FromUtf8Error {
    fn error_type(&self) -> &str {
        "FromUtf8Error"
    }
}
impl LambdaErrorExt for RecvError {
    fn error_type(&self) -> &str {
        "RecvError"
    }
}
impl LambdaErrorExt for SystemTimeError {
    fn error_type(&self) -> &str {
        "SystemTimeError"
    }
}
