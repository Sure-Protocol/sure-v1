///! Insurance contract representing the proof
///! that a user has insurance
use anchor_lang::prelude::*;
use anchor_lang::solana_program;

use anchor_spl::token::{Mint, Token, TokenAccount};

use vipers::{assert_is_ata, prelude::*};

const SURE_TIME_LOCK_IN_SECONDS: u64 = solana_program::clock::SECONDS_PER_DAY;

/// --- Insurance Contracts ----
/// <POOLS>
/// Holds information about the contracts held by the
/// user
#[account]
pub struct CoveragePositions {
    /// owner of account
    pub owner: Pubkey,

    /// Vec of Pool PubKeys
    pub pools: Vec<Pubkey>, // 4 + 32*256 = 8196, 256 insured contracts
}

impl CoveragePositions {
    pub const SPACE: usize = 1 + 32 + 4 + 32 * 256;
}

/// --- Pool insurance contract ---
/// <POOL>
/// Accumulation of all insurance contracts for a user in  
/// a given pool.
#[account]
pub struct CoveragePosition {
    /// Pool insured against
    pub pool: Pubkey,

    /// token mint
    pub position_mint: Pubkey,

    /// Contract expiry
    pub expiry_ts: i64, // 8 byte

    /// Contract Amount
    pub insured_amount: u64, // 8 byte

    /// Owner of contract
    pub owner: Pubkey, // 32 byte
}

impl CoveragePosition {
    pub const SPACE: usize = 8 + 8 + 32 + 32;

    pub fn initialize(&mut self, position_owner: &Signer, position_mint: Pubkey) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp_now = clock.unix_timestamp;

        self.expiry_ts = timestamp_now;
        self.insured_amount = 0;
        self.position_mint = position_mint;
        self.owner = position_owner.key();
        Ok(())
    }
}
