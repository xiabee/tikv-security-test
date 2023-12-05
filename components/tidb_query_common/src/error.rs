// Copyright 2019 TiKV Project Authors. Licensed under Apache-2.0.

use std::convert::Infallible;

use error_code::{self, ErrorCode, ErrorCodeExt};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EvaluateError {
    #[error("Execution terminated due to exceeding the deadline")]
    DeadlineExceeded,

    #[error("Invalid {charset} character string")]
    InvalidCharacterString { charset: String },

    /// This variant is only a compatible layer for existing CodecError.
    /// Ideally each error kind should occupy an enum variant.
    #[error("{msg}")]
    Custom { code: i32, msg: String },

    #[error("{0}")]
    Other(String),
}

impl EvaluateError {
    /// Returns the error code.
    pub fn code(&self) -> i32 {
        match self {
            EvaluateError::InvalidCharacterString { .. } => 1300,
            EvaluateError::DeadlineExceeded => 9007,
            EvaluateError::Custom { code, .. } => *code,
            EvaluateError::Other(_) => 10000,
        }
    }
}

// Compatible shortcut for existing errors generated by `box_err!`.
impl From<Box<dyn std::error::Error + Send + Sync>> for EvaluateError {
    #[inline]
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        EvaluateError::Other(err.to_string())
    }
}

impl From<tikv_util::deadline::DeadlineError> for EvaluateError {
    #[inline]
    fn from(_: tikv_util::deadline::DeadlineError) -> Self {
        EvaluateError::DeadlineExceeded
    }
}

impl From<Infallible> for EvaluateError {
    fn from(e: Infallible) -> Self {
        match e {}
    }
}

impl From<std::str::Utf8Error> for EvaluateError {
    fn from(err: std::str::Utf8Error) -> Self {
        EvaluateError::Other(format!("invalid input value: {:?}", err))
    }
}

impl From<std::string::FromUtf8Error> for EvaluateError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        EvaluateError::Other(format!("invalid input value: {:?}", err))
    }
}

impl ErrorCodeExt for EvaluateError {
    fn error_code(&self) -> ErrorCode {
        match self {
            EvaluateError::DeadlineExceeded => error_code::coprocessor::DEADLINE_EXCEEDED,
            EvaluateError::InvalidCharacterString { .. } => {
                error_code::coprocessor::INVALID_CHARACTER_STRING
            }
            EvaluateError::Custom { .. } => error_code::coprocessor::EVAL,
            EvaluateError::Other(_) => error_code::UNKNOWN,
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct StorageError(#[from] pub anyhow::Error);

/// We want to restrict the type of errors to be either a `StorageError` or `EvaluateError`, thus
/// `failure::Error` is not used. Instead, we introduce our own error enum.
#[derive(Debug, Error)]
pub enum ErrorInner {
    #[error("Storage error: {0}")]
    Storage(#[source] StorageError),

    #[error("Evaluate error: {0}")]
    Evaluate(#[source] EvaluateError),
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error(#[from] pub Box<ErrorInner>);

impl From<ErrorInner> for Error {
    #[inline]
    fn from(e: ErrorInner) -> Self {
        Error(Box::new(e))
    }
}

impl From<StorageError> for Error {
    #[inline]
    fn from(e: StorageError) -> Self {
        Error(Box::new(ErrorInner::Storage(e)))
    }
}

impl From<EvaluateError> for Error {
    #[inline]
    fn from(e: EvaluateError) -> Self {
        Error(Box::new(ErrorInner::Evaluate(e)))
    }
}

// Any error that turns to `EvaluateError` can be turned to `Error` as well.
impl<T: Into<EvaluateError>> From<T> for Error {
    #[inline]
    default fn from(err: T) -> Self {
        let eval_error = err.into();
        eval_error.into()
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl ErrorCodeExt for Error {
    fn error_code(&self) -> ErrorCode {
        match self.0.as_ref() {
            ErrorInner::Storage(_) => error_code::coprocessor::STORAGE_ERROR,
            ErrorInner::Evaluate(e) => e.error_code(),
        }
    }
}
