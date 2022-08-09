use crate::common::errors::SureError;
use crate::common::fixed_point_math::mul_round_down_Q3232;
use crate::common::liquidity::calculate_new_liquidity;
use anchor_lang::prelude::*;
use vipers::{assert_is_ata, prelude::*};

use super::{pool::Pool, tick_v2::Tick};

/// -- Liquidity Position --
///
/// Holds information about liquidity at a given tick
///
#[account]
#[derive(Default)]
pub struct LiquidityPosition {
    pub bump: u8, // 1 byte

    /// The amount of liquidity provided in lamports
    pub liquidity: u128, // 16 bytes

    /// Liquidity Pool
    pub pool: Pubkey, // 32 bytes

    /// NFT mint. The mint representing the position
    /// The NFT is the owner of the position.
    pub position_mint: Pubkey, // 32 bytes

    /// Upper tick that liquidity is provided at
    pub tick_index_upper: i32, // 4

    /// Lower tick
    /// that liquidity is provided at
    pub tick_index_lower: i32, // 4

    /// Checkpoint last fee in vault a
    pub fee_checkpoint_in_0_last_x64: u128, // 16 bytes

    /// Checkpoint last fee in vault b
    pub fee_checkpoint_in_1_last_x64: u128, // 16 bytes

    /// Non collected fees from vault a
    pub fee_owed_in_0: u128, // 16 bytes

    /// Non collected fees from vault b
    pub fee_owed_in_1: u128, // 16 bytes
}

impl LiquidityPosition {
    pub const SPACE: usize = 1 + 16 + 32 + 32 + 4 + 4 + 16 + 16 + 16 + 16;

    /// Initialize Liquidity Position
    ///
    pub fn initialize(
        &mut self,
        pool: &Account<Pool>,
        tick_index_upper: i32,
        tick_index_lower: i32,
        position_mint: Pubkey,
    ) -> Result<()> {
        if !Tick::is_valid_tick(tick_index_lower, pool.tick_spacing) {
            return Err(SureError::InvalidLowerTickIndexProvided.into());
        }

        if !Tick::is_valid_tick(tick_index_upper, pool.tick_spacing) {
            return Err(SureError::InvalidUpperTickIndexProvided.into());
        }
        if tick_index_lower > tick_index_upper {
            return Err(SureError::LowerTickgtUpperTick.into());
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
        liquidity_delta: i128,
        fee_growth_inside_0: u128,
        fee_growth_inside_1: u128,
    ) -> Result<()> {
        let fee_change_per_unit_0 = fee_growth_inside_0
            .checked_sub(self.fee_checkpoint_in_0_last_x64)
            .ok_or(SureError::InvalidFeeGrowthSubtraction)?;

        let fee_change_per_unit_1 = fee_growth_inside_1
            .checked_sub(self.fee_checkpoint_in_1_last_x64)
            .ok_or(SureError::InvalidFeeGrowthSubtraction)?;

        let fee_change_total_0 = mul_round_down_Q3232(self.liquidity, fee_change_per_unit_0)?;
        let fee_change_total_1 = mul_round_down_Q3232(self.liquidity, fee_change_per_unit_1)?;

        self.fee_checkpoint_in_0_last_x64 = fee_growth_inside_0;
        self.fee_checkpoint_in_1_last_x64 = fee_growth_inside_1;

        self.fee_owed_in_0 = self.fee_owed_in_0.wrapping_add(fee_change_total_0);
        self.fee_owed_in_1 = self.fee_owed_in_1.wrapping_add(fee_change_total_1);

        self.liquidity = calculate_new_liquidity(self.liquidity, liquidity_delta)?;

        Ok(())
    }
}
