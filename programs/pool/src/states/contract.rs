use crate::bitmap::BitMap;
///! Insurance contract representing the proof
///! that a user has insurance
use anchor_lang::prelude::*;

/// Account to keep an overview over a users
/// insurance contract within a pool
#[account]
pub struct UserInsuranceContracts {
    /// Bump
    pub bump: u8, // 1 byte

    /// Amount insured in pool
    pub amount_insured: u64, // 8 bytes

    /// Pool
    pub pool: Pubkey, // 8 bytes

    // Insurance Positions
    //pub insurance_positions: BitMap, // 4 + 32*256
}

impl UserInsuranceContracts {
    const SIZE: usize = 1 + 8 + 8 + 4 + 32 * 256;
}
/// Contract
/// Seed: [
/// "sure-insurance-contract"
/// signer
/// pool
/// ]
#[account]
#[derive(Default)]
pub struct InsuranceContract {
    /// The bump identity of the PDA
    pub bump: u8, // 1 byte

    /// Amount insured
    pub amount: u64, // 8 bytes

    /// Premium
    pub premium: u64, // 8 bytes

    /// Remaining premium for account
    pub remaining_premium: u64, // 8 bytes

    /// The length of the contract
    pub period_ts: i64, // 8 bytes

    /// The end time of the contract
    pub end_ts: i64, // 8 bytes

    /// Insured pool
    pub pool: Pubkey, // 32 bytes

    /// Tick Account used to buy from
    pub tick_account: Pubkey, // 32 bytes

    // Token Mint
    pub token_mint: Pubkey, // 32 bytes

    /// Owner of insurance contract
    pub owner: Pubkey, // 32 bytes

    /// Is the insurance contract active
    pub active: bool, // 1 byte

    /// Created
    pub created_ts: i64, // 8 bytes
}

impl InsuranceContract {
    pub const SPACE: usize = 1 + 8 + 8 + 8 + 8 + 8 + 32 + 32 + 32 + 32 + 1 + 8;
}
