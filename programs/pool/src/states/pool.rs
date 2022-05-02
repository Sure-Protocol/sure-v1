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
    pub insurance_fee: i32, // 4 bytes

    /// Size of range to provide liquidity in
    /// Measured in basis points. Standard is 1 (basis point, 0.01%)
    pub range_size: i32, // 4 bytes 

    /// Number of ranges 
    pub ranges: i32, //4 bytes,

    /// The total liquidity in the pool 
    pub liquidity: u64, // 8 bytes

    /// Available Liquidity in the pool
    pub active_liquidity: u64, // 8 bytes 

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

impl PoolAccount{
    pub const SPACE:usize = 1+32+4+4+4+8+8+8+4+200+32+32+1;
}


/// Tick acount (PDA) is used to hold information about 
/// the liquidity at a current tick
#[account]
#[derive(Default)]
pub struct Tick{
    /// The bump identity of the PDA
    pub bump: u8, // 1 byte

    /// The active liquidity at the tick
    pub liquidity: u64, // 8bytes

    /// The tick in basis points
    pub tick: u32, // 8 bytes 

    /// Boolean representing whether the liquidity is active
    pub active: bool, // 1 byte 
}

/// Liquidity Position holds information about a given 
/// token position
/// Each token position references an NFT mint 
#[account]
#[derive(Default)]
pub struct LiquidityPosition {
    /// Bump Identity
    pub bump: u8, // 1byte

    /// The amount of liquidity provided in lamports 
    pub liquidity: u64, // 8 bytes

    /// the amount of liquidity used
    pub used_liquidity: u64, // 8 bytes

    /// Liquidity Pool 
    pub pool: Pubkey, // 32 bytes

    /// Mint of token provided
    pub token_mint: Pubkey, // 32 bytes
    
    /// NFT mint. The mint representing the position
    /// The NFT is the owner of the position. 
    pub nft_mint: Pubkey, // 32 bytes

    /// Time at liquidity position creation
    pub created_at: i64, // 8 bytes,

    /// The tick that the liquidity is at 
    pub tick: u32, // 4 bytes
}

impl LiquidityPosition{
    pub const SPACE:usize = 1 + 8 +8 + 32 + 32 + 32 + 8 + 4;
}

#[event]
pub struct NewLiquidityPosition {
    pub tick: u32,
    pub liquidity: u64,
}






