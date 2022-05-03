///! Insurance contract representing the proof
///! that a user has insurance

use anchor_lang::prelude::*;

/// Contract
/// Seed: [
/// sure-insurance-contract,
/// pool,
/// token,
/// liquidity_position
/// ]
#[account]
pub struct InsuranceContract {
    /// The bump identity of the PDA
    pub bump: u8, // 1 byte

    /// Amount insured
    pub amount: u64, // 8 bytes

    /// Liquidity position
    /// The liquidity position insuring the user
    pub liquidity_position: Pubkey,
}

impl InsuranceContract {
    pub const SPACE: usize = 0;
}