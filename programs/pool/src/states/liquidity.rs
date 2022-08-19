///! Liquidity positions
///!
use anchor_lang::prelude::*;

use crate::states::{
    bitmap::BitMap,
    owner::ProtocolOwner,
    seeds::{SURE_LIQUIDITY_POSITION, SURE_NFT_MINT_SEED, SURE_TOKEN_ACCOUNT_SEED},
    tick::Tick,
};

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use vipers::{assert_is_ata, prelude::*};

/// -- Liquidity Position --
///
/// Holds information about liquidity at a given tick
///
#[account]
#[derive(Default)]
pub struct LiquidityPosition {
    /// Bump Identity
    pub bump: u8, // 1byte

    /// The amount of liquidity provided in lamports
    pub liquidity: u128, // 8 bytes

    /// the amount of liquidity used
    pub used_liquidity: u128, // 8 bytes

    /// Liquidity Pool
    pub pool: Pubkey, // 32 bytes

    /// NFT mint. The mint representing the position
    /// The NFT is the owner of the position.
    pub position_mint: Pubkey, // 32 bytes

    /// Id in the tick pool
    pub tick_index_lower: i32,

    /// The tick that the liquidity is at
    pub tick_index_upper: i32, // 8 bytes

    /// Outstanding Rewards
    pub owed_fees: u32, // 4 bytes
    pub owed_premium: u32,
}

impl LiquidityPosition {
    pub const SPACE: usize = 1 + 8 + 8 + 32 + 32 + 32 + 32 + 8 + 1 + 8 + 4;

    pub fn initialize(
        &mut self,
        bump: u8,
        liquidity_delta: u128,
        pool: Pubkey,
        position_mint: Pubkey,
        tick_index_lower: i32,
        tick_index_upper: i32,
    ) {
        self.bump = bump;
        self.liquidity = liquidity_delta;
        self.pool = pool;
        self.position_mint = position_mint;
        self.tick_index_lower = tick_index_lower;
        self.tick_index_upper = tick_index_upper;
    }
}
