//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the TokenLending program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum ErrorCode {
    // 0
    /// Invalid instruction data passed in.
    #[error("the collateral/loan account is not empty")]
    AccountNotEmptyError,
}

impl From<ErrorCode> for ProgramError {
    fn from(e: ErrorCode) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for ErrorCode {
    fn type_of() -> &'static str {
        "Lending Error"
    }
}