use std::num::TryFromIntError;

use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq)]
pub enum ShieldError {
    #[msg("Default error. Check if error handling.")]
    DefaultError,

    #[msg("The coverage position was rejected by the orderbook")]
    CoveragePositionRejected,
}

impl From<TryFromIntError> for ShieldError {
    fn from(_: TryFromIntError) -> Self {
        ShieldError::DefaultError
    }
}
