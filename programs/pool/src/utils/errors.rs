use anchor_lang::prelude::*;

#[error_code]
pub enum SureError {
    #[msg("Invalid mint")]
    InvalidMint,

    /// =========== Pool =============
    #[msg("Invalid Range size")]
    InvalidRangeSize,
}