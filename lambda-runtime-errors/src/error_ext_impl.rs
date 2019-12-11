// Generated code, DO NOT MODIFY!
// This file contains the implementation of the LambdaErrorExt
// trait for most of the standard library errors as well as the
// implementation of the From trait for the HandlerError struct
// to support the same standard library errors.

use crate::{HandlerError, LambdaErrorExt};
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
        "std::env::VarError"
    }
}
impl LambdaErrorExt for ParseError {
    fn error_type(&self) -> &str {
        "std::string::ParseError"
    }
}
impl LambdaErrorExt for RecvTimeoutError {
    fn error_type(&self) -> &str {
        "std::sync::mpsc::RecvTimeoutError"
    }
}
impl LambdaErrorExt for TryRecvError {
    fn error_type(&self) -> &str {
        "std::sync::mpsc::TryRecvError"
    }
}
impl LambdaErrorExt for LayoutErr {
    fn error_type(&self) -> &str {
        "std::alloc::LayoutErr"
    }
}
impl LambdaErrorExt for BorrowError {
    fn error_type(&self) -> &str {
        "std::cell::BorrowError"
    }
}
impl LambdaErrorExt for BorrowMutError {
    fn error_type(&self) -> &str {
        "std::cell::BorrowMutError"
    }
}
impl LambdaErrorExt for DecodeUtf16Error {
    fn error_type(&self) -> &str {
        "std::char::DecodeUtf16Error"
    }
}
impl LambdaErrorExt for ParseCharError {
    fn error_type(&self) -> &str {
        "std::char::ParseCharError"
    }
}
impl LambdaErrorExt for JoinPathsError {
    fn error_type(&self) -> &str {
        "std::env::JoinPathsError"
    }
}
impl LambdaErrorExt for FromBytesWithNulError {
    fn error_type(&self) -> &str {
        "std::ffi::FromBytesWithNulError"
    }
}
impl LambdaErrorExt for IntoStringError {
    fn error_type(&self) -> &str {
        "std::ffi::IntoStringError"
    }
}
impl LambdaErrorExt for NulError {
    fn error_type(&self) -> &str {
        "std::ffi::NulError"
    }
}
impl LambdaErrorExt for AddrParseError {
    fn error_type(&self) -> &str {
        "std::net::AddrParseError"
    }
}
impl LambdaErrorExt for ParseFloatError {
    fn error_type(&self) -> &str {
        "std::num::ParseFloatError"
    }
}
impl LambdaErrorExt for ParseIntError {
    fn error_type(&self) -> &str {
        "std::num::ParseIntError"
    }
}
impl LambdaErrorExt for StripPrefixError {
    fn error_type(&self) -> &str {
        "std::path::StripPrefixError"
    }
}
impl LambdaErrorExt for ParseBoolError {
    fn error_type(&self) -> &str {
        "std::str::ParseBoolError"
    }
}
impl LambdaErrorExt for Utf8Error {
    fn error_type(&self) -> &str {
        "std::str::Utf8Error"
    }
}
impl LambdaErrorExt for FromUtf16Error {
    fn error_type(&self) -> &str {
        "std::string::FromUtf16Error"
    }
}
impl LambdaErrorExt for FromUtf8Error {
    fn error_type(&self) -> &str {
        "std::string::FromUtf8Error"
    }
}
impl LambdaErrorExt for RecvError {
    fn error_type(&self) -> &str {
        "std::sync::mpsc::RecvError"
    }
}
impl LambdaErrorExt for SystemTimeError {
    fn error_type(&self) -> &str {
        "std::time::SystemTimeError"
    }
}
impl From<VarError> for HandlerError {
    fn from(e: VarError) -> Self {
        HandlerError::new(e)
    }
}
impl From<RecvTimeoutError> for HandlerError {
    fn from(e: RecvTimeoutError) -> Self {
        HandlerError::new(e)
    }
}
impl From<TryRecvError> for HandlerError {
    fn from(e: TryRecvError) -> Self {
        HandlerError::new(e)
    }
}
impl From<LayoutErr> for HandlerError {
    fn from(e: LayoutErr) -> Self {
        HandlerError::new(e)
    }
}
impl From<BorrowError> for HandlerError {
    fn from(e: BorrowError) -> Self {
        HandlerError::new(e)
    }
}
impl From<BorrowMutError> for HandlerError {
    fn from(e: BorrowMutError) -> Self {
        HandlerError::new(e)
    }
}
impl From<DecodeUtf16Error> for HandlerError {
    fn from(e: DecodeUtf16Error) -> Self {
        HandlerError::new(e)
    }
}
impl From<ParseCharError> for HandlerError {
    fn from(e: ParseCharError) -> Self {
        HandlerError::new(e)
    }
}
impl From<JoinPathsError> for HandlerError {
    fn from(e: JoinPathsError) -> Self {
        HandlerError::new(e)
    }
}
impl From<FromBytesWithNulError> for HandlerError {
    fn from(e: FromBytesWithNulError) -> Self {
        HandlerError::new(e)
    }
}
impl From<IntoStringError> for HandlerError {
    fn from(e: IntoStringError) -> Self {
        HandlerError::new(e)
    }
}
impl From<NulError> for HandlerError {
    fn from(e: NulError) -> Self {
        HandlerError::new(e)
    }
}
impl From<AddrParseError> for HandlerError {
    fn from(e: AddrParseError) -> Self {
        HandlerError::new(e)
    }
}
impl From<ParseFloatError> for HandlerError {
    fn from(e: ParseFloatError) -> Self {
        HandlerError::new(e)
    }
}
impl From<ParseIntError> for HandlerError {
    fn from(e: ParseIntError) -> Self {
        HandlerError::new(e)
    }
}
impl From<StripPrefixError> for HandlerError {
    fn from(e: StripPrefixError) -> Self {
        HandlerError::new(e)
    }
}
impl From<ParseBoolError> for HandlerError {
    fn from(e: ParseBoolError) -> Self {
        HandlerError::new(e)
    }
}
impl From<Utf8Error> for HandlerError {
    fn from(e: Utf8Error) -> Self {
        HandlerError::new(e)
    }
}
impl From<FromUtf16Error> for HandlerError {
    fn from(e: FromUtf16Error) -> Self {
        HandlerError::new(e)
    }
}
impl From<FromUtf8Error> for HandlerError {
    fn from(e: FromUtf8Error) -> Self {
        HandlerError::new(e)
    }
}
impl From<RecvError> for HandlerError {
    fn from(e: RecvError) -> Self {
        HandlerError::new(e)
    }
}
impl From<SystemTimeError> for HandlerError {
    fn from(e: SystemTimeError) -> Self {
        HandlerError::new(e)
    }
}
