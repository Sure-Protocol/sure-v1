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
pub fn calculate_token_0_change(
    liquidity_delta: u64,
    current_sqrt_price: u64,
    current_tick_index: u64,
    tick_index_upper: u64,
    tick_index_lower: u64,
    sqrt_price_0: u64,
    sqrt_price_1: u64,
) -> Result<u64> {
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
        let token_change = liquidity_delta
            .checked_mul(sqrt_diff)
            .ok_or(SureError::MultiplictationQ3232Overflow)?;
        Ok(token_change)
    } else {
        let sqrt_diff = current_sqrt_price - sqrt_price_lower;
        let token_change = liquidity_delta
            .checked_mul(sqrt_diff)
            .ok_or(SureError::MultiplictationQ3232Overflow)?;
        Ok(token_change)
    }
}

pub fn calculate_token_1_change(
    liquidity_delta: u64,
    current_sqrt_price: u64,
    current_tick_index: u64,
    tick_index_upper: u64,
    tick_index_lower: u64,
    sqrt_price_0: u64,
    sqrt_price_1: u64,
) -> Result<()> {
    let (sqrt_price_lower, sqrt_price_upper) = if sqrt_price_1 > sqrt_price_0 {
        (sqrt_price_0, sqrt_price_1)
    } else {
        (sqrt_price_1, sqrt_price_0)
    };

    let sqrt_price_lower_reciprocal = (1 as u64)
        .checked_div(sqrt_price_lower)
        .ok_or(SureError::DivisionQ3232Error)?;

    let sqrt_price_upper_reciprocal = (1 as u64)
        .checked_div(sqrt_price_upper)
        .ok_or(SureError::DivisionQ3232Error)?;

    // if current_tick_index < tick_index_lower {
    //     let sqrt_price_lower_reciprocal = (1 as u64)
    //         .checked_div(sqrt_price_lower)
    //         .ok_or(SureError::DivisionQ3232Error)?;
    //     let sqrt_price_
    // }
    Ok(())
}
