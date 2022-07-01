use crate::utils::errors::SureError;
use anchor_lang::prelude::*;
use vipers::{assert_is_ata, prelude::*};

use super::pool::Pool;
use super::tick::Tick;

/// -- Liquidity Position --
///
/// Holds information about liquidity at a given tick
///
#[account]
#[derive(Default)]
pub struct LiquidityPosition {
    /// The amount of liquidity provided in lamports
    pub liquidity: u64, // 8 bytes

    /// Liquidity Pool
    pub pool: Pubkey, // 32 bytes

    /// NFT mint. The mint representing the position
    /// The NFT is the owner of the position.
    pub position_mint: Pubkey, // 32 bytes

    /// Upper tick that liquidity is provided at
    pub tick_index_upper: i32,

    /// Lower tick
    /// that liquidity is provided at
    pub tick_index_lower: i32,

    /// Checkpoint last fee in vault a
    pub fee_checkpoint_in_0_last_x32: u64,

    /// Checkpoint last fee in vault b
    pub fee_checkpoint_in_1_last_x32: u64,

    /// Non collected fees from vault a
    pub fee_owed_in_0: u64,

    /// Non collected fees from vault b
    pub fee_owed_in_1: u64,
}

impl LiquidityPosition {
    pub const SPACE: usize = 1 + 8 + 8 + 32 + 32 + 32 + 32 + 8 + 1 + 8 + 4;

    /// Initialize Liquidity Position
    ///
    pub fn initialize(
        &mut self,
        pool: &Account<Pool>,
        tick_index_upper: i32,
        tick_index_lower: i32,
        position_mint: Pubkey,
    ) -> Result<()> {
        if !Tick::is_valid_tick(tick_index_lower, pool.tick_spacing)
            || !Tick::is_valid_tick(tick_index_upper, pool.tick_spacing)
            || tick_index_lower > tick_index_upper
        {
            return Err(SureError::InvalidTickIndexProvided.into());
        }
        self.pool = pool.key();
        self.position_mint = position_mint;
        self.tick_index_upper = tick_index_upper;
        self.tick_index_lower = tick_index_lower;
        Ok(())
    }

    /// Update the liquidity position
    ///
    /// This happens if the liquidity position is changed
    /// or the user wants to collect the fees
    pub fn update(
        &mut self,
        liquidity_delta: u64,
        fee_growth_inside_0: u64,
        fee_growth_inside_1: u64,
    ) -> Result<()> {
        let fee_change_per_unit_0 = fee_growth_inside_0
            .checked_sub(self.fee_checkpoint_in_0_last_x32)
            .ok_or(SureError::InvalidFeeGrowthSubtraction)?;

        let fee_change_per_unit_1 = fee_growth_inside_1
            .checked_sub(self.fee_checkpoint_in_1_last_x32)
            .ok_or(SureError::InvalidFeeGrowthSubtraction)?;

        //let fee_change_1 = checked_mult

        self.fee_checkpoint_in_0_last_x32 = fee_growth_inside_0;
        self.fee_checkpoint_in_1_last_x32 = fee_growth_inside_1;
        Ok(())
    }
}
