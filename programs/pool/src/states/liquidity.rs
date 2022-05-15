///! Liquidity positions
///!
use anchor_lang::prelude::*;
// Liquidity Position holds information about a given
/// token position
/// Each token position references an NFT mint
///
/// At each tick liquidity
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

    // TokenMint representing the position
    pub token_mint: Pubkey,

    /// Mint of token provided
    pub nft_account: Pubkey, // 32 bytes

    /// NFT mint. The mint representing the position
    /// The NFT is the owner of the position.
    pub nft_mint: Pubkey, // 32 bytes

    /// Time at liquidity position creation
    pub created_at: i64, // 8 bytes,

    /// Id in the tick pool
    pub tick_id: u8,

    /// The tick that the liquidity is at
    pub tick: u16, // 8 bytes

    /// Outstanding Rewards
    pub outstanding_rewards: u32, // 4 bytes
}

impl LiquidityPosition {
    pub const SPACE: usize = 1 + 8 + 8 + 32 + 32 + 32 + 32 + 8 + 1 + 8 + 4;
}

#[event]
pub struct NewLiquidityPosition {
    pub tick: u16,
    pub liquidity: u64,
}
