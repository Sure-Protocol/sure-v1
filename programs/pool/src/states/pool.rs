use std::thread::AccessError;

use anchor_lang::{prelude::*, solana_program::account_info::Account};
use anchor_spl::token::TokenAccount;

/// Account describing the pool manager
///
#[account]
#[derive(Default)]
pub struct PoolManager {
    // the current pool manager
    pub owner: Pubkey, // 32 bytes
    // bump to identify the PDA
    pub bump: u8, // 1 byte
}

impl PoolManager {
    pub const POOL_MANAGER_SIZE: usize = 32 + 1;
}

/// Pool Account (PDA) contains information describing the
/// insurance pool
#[account]
#[derive(Default)]
pub struct PoolAccount {
    /// Bump to identify the PDA
    pub bump: u8, // 1 byte

    /// Token held in the pool.
    /// In the beginning this is just USDC
    pub token: Pubkey, // 32 bytes

    /// Fee paid when buying insurance.
    /// in 10^-6
    pub insurance_fee: u16, // 4 bytes

    /// Spacing of Ticks
    /// Measured in basis points. Standard is 1 (basis point, 0.01%)
    pub tick_spacing: u16, // 2 bytes

    /// The total liquidity in the pool
    pub liquidity: u64, // 8 bytes

    /// Available Liquidity in the pool
    pub active_liquidity: u64, // 8 bytes

    /// Bitmap that contains tick information
    pub bitmap: Pubkey, // 32 bytes

    /// Current premium rate in basis points (0.01%).
    pub premium_rate: u64, // 8 bytes

    /// Name of pool visible to the user
    pub name: String, // 4 + 200 bytes

    /// The public key of the smart contract that is
    /// insured
    pub smart_contract: Pubkey, // 32 bytes

    /// Vault that holds the liquidity (tokens)
    pub vault: Pubkey, // 32 bytes

    /// Whether the insurance pool is locked
    pub locked: bool, // 1 byte
}

impl PoolAccount {
    pub const SPACE: usize = 1 + 32 + 4 + 4 + 4 + 8 + 8 + 8 + 4 + 200 + 32 + 32 + 1;
}
