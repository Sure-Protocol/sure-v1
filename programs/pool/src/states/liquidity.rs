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
    pub liquidity: u64, // 8 bytes

    /// the amount of liquidity used
    pub used_liquidity: u64, // 8 bytes

    /// Liquidity Pool
    pub pool: Pubkey, // 32 bytes

    /// Mint of token provided
    pub nft_account: Pubkey, // 32 bytes

    /// NFT mint. The mint representing the position
    /// The NFT is the owner of the position.
    pub nft_mint: Pubkey, // 32 bytes

    /// Id in the tick pool
    pub tick_id: u8,

    /// The tick that the liquidity is at
    pub tick: u16, // 8 bytes

    /// Outstanding Rewards
    pub outstanding_rewards: u32, // 4 bytes
}

impl LiquidityPosition {
    pub const SPACE: usize = 1 + 8 + 8 + 32 + 32 + 32 + 32 + 8 + 1 + 8 + 4;

    pub fn initialize(&mut self, bump: u8) {
        self.bump = bump;
    }
}
