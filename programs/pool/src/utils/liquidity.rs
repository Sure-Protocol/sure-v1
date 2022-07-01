use super::errors::SureError;
use anchor_lang::prelude::*;

pub fn validate_liquidity_amount(liquidity_amount: u64, increase: bool) -> Result<i64> {
    if liquidity_amount > i64::MAX as u64 {
        return Err(SureError::LiquidityTooLarge.into());
    }
    Ok(if increase {
        liquidity_amount as i64
    } else {
        -(liquidity_amount as i64)
    })
}

pub fn calculate_new_liquidity(liquidity: u64, delta: i64) -> Result<u64> {
    if delta == 0 {
        return Ok(liquidity);
    }

    if delta > 0 {
        liquidity
            .checked_add(delta as u64)
            .ok_or(SureError::LiquidityOverflow.into())
    } else {
        liquidity
            .checked_sub(delta.abs() as u64)
            .ok_or(SureError::LiquidityUnderflow.into())
    }
}
