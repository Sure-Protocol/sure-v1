use anchor_lang::prelude::*;
use std::num::TryFromIntError;

#[error_code]
#[derive(PartialEq)]
pub enum SureError {
    #[msg("Invalid mint")]
    InvalidMint,

    /// =========== Pool =============
    #[msg("Invalid Range size")]
    InvalidRangeSize,

    #[msg("Invalid tick")]
    InvalidTick,

    #[msg("Invalid tick spacing. Tick spacing might be 0.")]
    InvalidTickSpacing,

    #[msg("Tick array not found in tick array pool")]
    InvalidTickArrayIndexInTickArrayPool,

    #[msg("Invalid Amount")]
    InvalidAmount,

    #[msg("Provided Liquidity is too large")]
    LiquidityTooLarge,

    #[msg("Tick index is out of range")]
    TickOutOfRange,

    #[msg("The provided liquidity have to be greater than 0")]
    LiquidityHaveToBeGreaterThan0,

    #[msg("All of the liquidity is used")]
    LiquidityFilled,

    #[msg("Invalid Pool creator provided. Are you sure you are the protocol owner?")]
    InvalidPoolCreator,

    #[msg("Could not provide liquidity")]
    CouldNotProvideLiquidity,

    #[msg("Not empty Tick account")]
    TickAccountNotEmpty,

    #[msg("Invalid timestamp")]
    InvalidTimestamp,

    // ======= Insurance contract ======
    #[msg("Insurance Contract has expired")]
    InsuranceContractExpired,

    #[msg("Insurance Contract is not active")]
    InsuranceContractIsNotActive,

    #[msg("Number of pools in product pools is exceeded")]
    PoolsInProductPoolExceeded,

    #[msg("Product Pool is empty")]
    ProductPoolIsEmpty,

    #[msg("Fee rate exceeds the max of 10 000bp = 100%")]
    MaxFeeRateExceeded,

    #[msg("The Protocol fee rate exceeded 3 200bp=33%")]
    MaxProtocolFeeRateExceeded,

    #[msg("The sum of the sub fee rates exceeds the fee_rate")]
    InvalidSubFeeRates,

    #[msg("The max founders fee is exceeded")]
    MaxFoundersFeeRateExceeded,

    #[msg("The Liquidity Provider fee rate is too low")]
    TooLowLiquidityProviderFeeRate,

    #[msg("Square root price ratio is not within ranges")]
    SqrtRatioNotWithinRange,

    #[msg("The ordering of token mint are wrong")]
    WrongTokenMintOrder,

    #[msg("The word position is too large")]
    TooLargeWordPosition,

    #[msg("The word position is too small")]
    TooSmallWordPosition,

    #[msg("The specified word does not match the given tick array")]
    InvalidTickArrayWord,

    #[msg("Invalid upper and lower tick provided")]
    InvalidTickIndexProvided,

    #[msg("Not a valid owner. The expected user does not have ownership over the account")]
    InvalidOwner,

    #[msg("Could not update the pool liquidity")]
    CouldNotUpdatePoolLiquidity,

    #[msg("Liquidity change causes total liquidity to overflow")]
    LiquidityOverflow,

    #[msg("Liquidity change causes total liquidity to underflow")]
    LiquidityUnderflow,

    #[msg("Invalid fee growth subtraction")]
    InvalidFeeGrowthSubtraction,

    #[msg("Q32.32 multiplication overflow")]
    MultiplictationQ3232Overflow,

    #[msg("Q32.32 division error")]
    DivisionQ3232Error,

    #[msg("Q32.32 Substraction error")]
    SubtractionQ3232Error,

    #[msg("Q32.32 Addition overflow")]
    AdditionQ3232OverflowError,

    #[msg("U64 overflow")]
    OverflowU64,
}

impl From<TryFromIntError> for SureError {
    fn from(_: TryFromIntError) -> Self {
        SureError::InvalidAmount
    }
}
