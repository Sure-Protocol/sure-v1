use super::errors::SureError;
use crate::{helpers::tick::*, states::liquidity::LiquidityPosition};
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

/// Calculate/update new liquidity based on delta
///
/// Formula = delta > 0 : liquidity + delta ? liquidity - abs(delta)
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

/// Calculate the delta in token 0
///
/// This is based on formula 6.29 in Uniswap v3
/// white paper
pub fn calculate_token_0_delta(
    liquidity_delta: i64,
    liquidity_position: &LiquidityPosition,
    current_tick_index: i32,
) -> Result<u64> {
    let tick_index_lower = liquidity_position.tick_index_lower;
    let tick_index_upper = liquidity_position.tick_index_upper;
    let sqrt_price_0 = get_sqrt_ratio_at_tick(liquidity_position.tick_index_lower)?;
    let sqrt_price_1 = get_sqrt_ratio_at_tick(liquidity_position.tick_index_upper)?;
    let liquidity_delta_u = liquidity_delta as u64;

    let current_sqrt_price = get_sqrt_ratio_at_tick(current_tick_index)?;

    let (sqrt_price_lower, sqrt_price_upper) = if sqrt_price_1 > sqrt_price_0 {
        (sqrt_price_0, sqrt_price_1)
    } else {
        (sqrt_price_1, sqrt_price_0)
    };

    if current_tick_index < tick_index_lower {
        Ok(0)
    } else if current_tick_index >= tick_index_upper {
        let sqrt_diff = sqrt_price_upper - sqrt_price_lower;
        // TODO: calculations should be done in U64.64 and the result should be 64
        let token_change = liquidity_delta_u
            .checked_mul(sqrt_diff)
            .ok_or(SureError::MultiplictationQ3232Overflow)?;
        Ok(token_change)
    } else {
        let sqrt_diff = current_sqrt_price - sqrt_price_lower;
        let token_change = liquidity_delta_u
            .checked_mul(sqrt_diff)
            .ok_or(SureError::MultiplictationQ3232Overflow)?;
        Ok(token_change)
    }
}

/// Calculate token 1 change
///
/// TODO: upgrade U32.32 math
pub fn calculate_token_1_delta(
    liquidity_delta: i64,
    liquidity_position: &LiquidityPosition,
    current_tick_index: i32,
) -> Result<u64> {
    let tick_index_lower = liquidity_position.tick_index_lower;
    let tick_index_upper = liquidity_position.tick_index_upper;

    let liquidity_delta_u = liquidity_delta as u64;

    let sqrt_price_0 = get_sqrt_ratio_at_tick(liquidity_position.tick_index_lower)?;
    let sqrt_price_1 = get_sqrt_ratio_at_tick(liquidity_position.tick_index_upper)?;

    let current_sqrt_price = get_sqrt_ratio_at_tick(current_tick_index)?;
    let (sqrt_price_lower, sqrt_price_upper) = if sqrt_price_1 > sqrt_price_0 {
        (sqrt_price_0, sqrt_price_1)
    } else {
        (sqrt_price_1, sqrt_price_0)
    };

    let token_delta = if current_tick_index < tick_index_lower {
        let sqrt_price_lower_reciprocal = (1 as u64)
            .checked_div(sqrt_price_lower)
            .ok_or(SureError::DivisionQ3232Error)?;
        let sqrt_price_upper_reciprocal = (1 as u64)
            .checked_div(sqrt_price_upper)
            .ok_or(SureError::DivisionQ3232Error)?;
        let sqrt_price_sub = sqrt_price_lower_reciprocal
            .checked_sub(sqrt_price_upper_reciprocal)
            .ok_or(SureError::SubtractionQ3232Error)?;
        let res = liquidity_delta_u
            .checked_mul(sqrt_price_sub)
            .ok_or(SureError::MultiplictationQ3232Overflow)?;
        Ok(res)
    } else if current_tick_index >= tick_index_upper {
        Ok(0)
    } else {
        let sqrt_price_upper_reciprocal = (1 as u64)
            .checked_div(sqrt_price_upper)
            .ok_or(SureError::DivisionQ3232Error)?;

        let sqrt_price_reciprocal = (1 as u64)
            .checked_div(current_sqrt_price)
            .ok_or(SureError::MultiplictationQ3232Overflow)?;
        let sqrt_price_sub = sqrt_price_reciprocal
            .checked_sub(sqrt_price_upper_reciprocal)
            .ok_or(SureError::SubtractionQ3232Error)?;
        let res = liquidity_delta_u
            .checked_mul(sqrt_price_sub)
            .ok_or(SureError::MultiplictationQ3232Overflow)?;
        Ok(res)
    };

    token_delta
}
