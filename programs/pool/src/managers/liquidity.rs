use anchor_lang::prelude::*;

use crate::states::{liquidity::LiquidityPosition, tick_array::TickArrayPool};

pub struct LiquidityUpdate {}

/// Calculate the state update
///
///
pub fn get_next_liquidity_state(
    liquidity_amount: u128,
    liquidity_position: LiquidityPosition,
    tick_array_pool: TickArrayPool,
) -> Result<LiquidityUpdate> {
    Ok(LiquidityUpdate {})
}
