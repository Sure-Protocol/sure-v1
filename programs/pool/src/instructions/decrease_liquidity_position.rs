use anchor_lang::prelude::*;

use super::increase_liquidity_position::UpdateLiquidity;

pub fn handler(ctx: Context<UpdateLiquidity>) -> Result<()> {
    Ok(())
}

#[event]
pub struct NewLiquidityPosition {
    pub tick: u16,
    pub liquidity: u64,
}
