///! Insurance contract representing the proof
///! that a user has insurance
use anchor_lang::prelude::*;

/// Contract
/// Seed: [
/// "sure-insurance-contract"
/// signer
/// pool
/// ]
#[account]
pub struct InsuranceContract {
    /// The bump identity of the PDA
    pub bump: u8, // 1 byte

    /// Amount insured
    pub amount: u64, // 8 bytes

    /// Insured pool
    pub pool: Pubkey, // 32 bytes

    /// Owner of insurance contract 
    pub owner: Pubkey, // 32 bytes

    /// Is the insurance contract active
    pub active: bool, // 1 byte
}

impl InsuranceContract {
    pub const SPACE: usize = 1 + 8 + 32 + 32 + 1;
}
