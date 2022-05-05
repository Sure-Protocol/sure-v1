use anchor_lang::prelude::*;

#[error_code]
pub enum SureError {
    #[msg("Invalid mint")]
    InvalidMint,

    /// =========== Pool =============
    #[msg("Invalid Range size")]
    InvalidRangeSize,

    #[msg("Invalid tick to provide liquidity to")]
    InvalidTick,

    #[msg("Invalid Amount")]
    InvalidAmount,

    #[msg("All of the liquidity is used")]
    LiquidityFilled,

    #[msg("Invalid Pool creator provided. Are you sure you are the protocol owner?")]
    InvalidPoolCreator,

    #[msg("Could not provide liquidity")]
    CouldNotProvideLiquidity,
}
