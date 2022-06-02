use std::thread::AccessError;

use anchor_lang::{prelude::*, solana_program::account_info::Account};
use anchor_spl::token::TokenAccount;
use crate::BitMap;

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
    pub const SIZE: usize = 32 + 1;
}

/// SurePools holds information on which programs are
/// insured by Sure
#[account]
pub struct SurePools {
    pub bump: u8, // 1 byte

    /// Vec of insured programs
    pub pools: Vec<Pubkey>, // 4 + 32*256 = 8196, 256 insured contracts
}

impl SurePools {
    pub const SIZE: usize = 1 + 4 + 32 * 256;
}

/// Pool Account (PDA) contains information describing the
/// insurance pool
#[account]


pub struct PoolAccount {
    /// Bump to identify the PDA
    pub bump: u8, // 1 byte

    /// Name of pool visible to the user
    pub name: String, // 4 + 200 bytes

    /// Token Mint 
    pub token_mint: Pubkey, // 32 bytes

    /// Fee paid when buying insurance.
    /// in basis points
    pub insurance_fee: u16, // 4 bytes

    /// The total liquidity in the pool
    pub liquidity: u64, // 8 bytes

    /// Used Liquidity in the pool
    pub used_liquidity: u64, // 8 bytes

    /// Current premium rate in basis points (0.01%).
    pub premium_rate: u64, // 8 bytes

    /// Bitmap representing tick accounts 
    /// in pool
    pub pool_liquidity_tick_bitmap: Pubkey,

    /// The public key of the smart contract that is
    /// insured
    pub smart_contract: Pubkey, // 32 bytes

    /// Whether the insurance pool is locked
    pub locked: bool, // 1 byte
}

impl PoolAccount {
    pub const SPACE: usize = 1 + 4 + 200 + 4 + 32 + 8 + 8  + 8 + 32 + 32 + 1;
}

#[event]
pub struct InitializedPool {
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
}

#[event]
pub struct CreatePoolVaults {}
