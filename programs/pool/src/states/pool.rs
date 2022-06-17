
use anchor_lang::{prelude::*, solana_program::account_info::Account};

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
    pub const SPACE: usize = 1 + 4 + 32 * 256;
}

/// Pool Account (PDA) contains information describing the
/// insurance pool
#[account]
pub struct PoolAccount {
    /// Bump to identify the PDA
    pub bump: u8, // 1 byte

    /// Name of pool visible to the user
    pub name: String, // 4 + 200 bytes

    /// Fee paid when buying insurance.
    /// in basis points
    pub insurance_fee: u16, // 4 bytes

    /// The public key of the smart contract that is
    /// insured
    pub smart_contract: Pubkey, // 32 bytes

    /// Vec of token Pools
    pub token_pools: Vec<Pubkey>, // 4 + 32*64, 64 tokens for each pool

    /// Whether the insurance pool is locked
    pub locked: bool, // 1 byte
}

impl PoolAccount {
    pub const SPACE: usize = 1 + 4 + 200 + 4  + 32 + 4 + 32*64 + 1;
}

/// Pool Token Account
/// The account is used to keep an overview over the specific 
/// pool for a given token
/// 
/// This makes it easier to load data
/// 
/// Needs to 
#[account]
pub struct TokenPool {
    /// bump 
    pub bump: u8, // 1 byte 

    /// Token mint of pool
    pub token_mint: Pubkey, // 32 bytes

    /// Pool 
    pub pool: Pubkey, // 32 bytes,

    /// Liquidity in Token Pool
    pub liquidity: u64, // 8 bytes

    /// Used liquidity
    pub used_liquidity: u64, // 8 bytes
}

impl TokenPool {
    pub const SPACE: usize = 1 + 32 + 32 + 8 + 8 + 4 + 200;
}

#[event]
pub struct CreatePool {
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
    pub insurance_fee: u16,
}

#[event]
pub struct InitializeTokenPool {}
