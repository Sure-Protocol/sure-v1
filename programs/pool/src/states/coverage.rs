use std::mem::size_of;

///! Insurance contract representing the proof
///! that a user has insurance
use anchor_lang::prelude::*;
use anchor_lang::solana_program;

use crate::common::errors::SureError;
use crate::common::tick_math::get_sqrt_ratio_at_tick;
use crate::common::token_tx::deposit_into_vault;

use super::pool::BuyCoverageResult;
use super::tick_v2::get_tick_location;
use anchor_spl::token::{Mint, Token, TokenAccount};

const SURE_TIME_LOCK_IN_SECONDS: u64 = solana_program::clock::SECONDS_PER_DAY;
/// Max number of ticks to buy over
const NUM_TICKS_IN_COVERAGE_POSITION_usize: usize = 64 * 3;
const NUM_TICKS_IN_COVERAGE_POSITION: i32 = 64;

/// TickCoveragePosition keeps information
/// about the coverage position at a given tick
#[account(zero_copy)]
#[repr(packed)]
#[derive(Default, Debug, PartialEq)]
pub struct TickCoveragePosition {
    /// Amount of coverage bought at tick
    pub coverage_amount: u64, // 8 bytes
}

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
#[account(zero_copy)]
#[repr(packed)]
pub struct CoveragePosition {
    /// Pool insured against
    pub pool: Pubkey,

    /// token mint
    pub position_mint: Pubkey,

    /// Contract expiry
    pub expiry_ts: i64, // 8 byte

    /// Contract Amount
    pub covered_amount: u64, // 8 byte

    /// Owner of contract
    pub owner: Pubkey, // 32 byte

    ///start tick index
    pub start_tick_index: i32, // 4 bytes

    /// last tick index with liquidity
    pub last_covered_tick_index: i32,

    /// Coverage amount at Ticks
    pub coverage_amount_ticks: [u64; NUM_TICKS_IN_COVERAGE_POSITION_usize], // 8*64*3 = 1_536 bytes
}

impl Default for CoveragePosition {
    #[inline]
    fn default() -> Self {
        CoveragePosition {
            pool: Pubkey::default(),
            position_mint: Pubkey::default(),
            expiry_ts: 0,
            covered_amount: 0,
            owner: Pubkey::default(),
            start_tick_index: 0,
            last_covered_tick_index: 0,
            coverage_amount_ticks: [0; NUM_TICKS_IN_COVERAGE_POSITION_usize],
        }
    }
}

impl CoveragePosition {
    pub const SPACE: usize = 32
        + 32
        + 8
        + 8
        + 32
        + NUM_TICKS_IN_COVERAGE_POSITION_usize * size_of::<TickCoveragePosition>();

    pub fn initialize(
        &mut self,
        position_owner: &Signer,
        position_mint: Pubkey,
        start_tick_index: i32,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let timestamp_now = clock.unix_timestamp;

        self.expiry_ts = timestamp_now;
        self.covered_amount = 0;
        self.position_mint = position_mint;
        self.owner = position_owner.key();
        self.start_tick_index = start_tick_index;
        Ok(())
    }

    /// Update The coverage position
    ///
    /// update the position based on the coverage change
    /// result
    pub fn update_from_coverage_update(
        &mut self,
        coverage_result: BuyCoverageResult,
        expiry_ts: i64,
    ) {
        self.expiry_ts = expiry_ts;
        self.covered_amount = coverage_result.get_coverage_amount();
    }
    pub fn get_max_tick_index(&self, tick_spacing: u16) -> i32 {
        self.start_tick_index + NUM_TICKS_IN_COVERAGE_POSITION * tick_spacing as i32
    }

    pub fn get_lowest_sqrt_price_x32(&self) -> Result<u64> {
        get_sqrt_ratio_at_tick(self.start_tick_index)
    }

    /// Calculate the highest tick index
    pub fn get_current_coverage_position(&self, tick_spacing: u16) -> Result<i32> {
        get_tick_location(
            self.start_tick_index,
            self.last_covered_tick_index,
            tick_spacing,
        )
    }

    pub fn get_coverage_at_tick_index(&self, tick_index: i32, tick_spacing: u16) -> Result<u64> {
        let loc = get_tick_location(self.start_tick_index, tick_index, tick_spacing)?;
        match self.coverage_amount_ticks.get(loc as usize) {
            Some(amount) => Ok(*amount),
            None => return Err(SureError::TickOutOfRange.into()),
        }
    }

    /// Update coverage position
    ///
    /// update based on change at tick
    ///
    /// * Arguments
    /// - amount_in: coverage amount at the given tick
    pub fn update_coverage_at_tick(
        &mut self,
        tick_index: i32,
        tick_spacing: u16,
        amount_in: u64,
        a_to_b: bool,
    ) -> Result<()> {
        if self.is_tick_index_out_of_bounds(tick_index, tick_spacing, a_to_b) {
            return Err(SureError::TickOutOfRange.into());
        }

        let max_tick_index = self.get_max_tick_index(tick_spacing);
        if tick_index > max_tick_index || tick_index < self.start_tick_index {
            return Err(SureError::TickOutOfRange.into());
        }

        let tick_location = get_tick_location(self.start_tick_index, tick_index, tick_spacing)?;
        self.coverage_amount_ticks[tick_location as usize] += amount_in;

        Ok(())
    }

    /// Check if the provided tick_index is beyond the bounds
    /// if a_to_b we assume that we are buying insurance
    pub fn is_tick_index_out_of_bounds(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> bool {
        if tick_index < self.start_tick_index || tick_index > self.get_max_tick_index(tick_spacing)
        {
            return false;
        } else {
            return true;
        }
    }
}
