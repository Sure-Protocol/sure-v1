use super::errors::SureError;
use super::tick_math::get_sqrt_ratio_at_tick;
use super::*;
use crate::states::{liquidity::LiquidityPosition, Pool, TickArray, TickUpdate};
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

pub struct UpdatedLiquidityState {
    liquidity_delta: i64,
    next_liquidity: u64,
    tick_lower_update: TickUpdate,
    tick_upper_update: TickUpdate,
    pub token_0_delta: u64,
    pub token_1_delta: u64,
    fee_growth_inside_0_x32: u64,
    fee_growth_inside_1_x32: u64,
}

/// Build new liquidity state
///
/// the new state is used to update all the states
/// involved in the transaction
///
pub fn build_new_liquidity_state<'info>(
    position: &Account<'info, LiquidityPosition>,
    pool: &Account<'info, Pool>,
    tick_array_lower: &AccountLoader<'info, TickArray>,
    tick_array_upper: &AccountLoader<'info, TickArray>,
    amount_delta: u64,
    productType: &ProductType,
    is_increasing: bool,
) -> Result<UpdatedLiquidityState> {
    if amount_delta == 0 {
        return Err(SureError::LiquidityHaveToBeGreaterThan0.into());
    }

    // Validate the amount delta and return the +/- delta
    let liquidity_delta = validate_liquidity_amount(amount_delta, is_increasing)?;

    // Calculate Tick changes
    // Get Tick accounts
    let tick_array_lower = tick_array_lower.load_mut()?;
    let tick_lower = tick_array_lower.get_tick(position.tick_index_lower, pool.tick_spacing)?;
    let tick_array_upper = tick_array_upper.load_mut()?;
    let tick_upper = tick_array_upper.get_tick(position.tick_index_upper, pool.tick_spacing)?;

    // Calculate the updated liquidity
    let next_liquidity = pool.get_next_liquidity(&position, liquidity_delta)?;

    // Update lower tick
    let tick_lower_update = tick_lower.calculate_next_liquidity_update(
        position.tick_index_lower,
        pool.current_tick_index,
        pool.fee_growth_0_x32,
        pool.fee_growth_1_x32,
        liquidity_delta,
        productType,
        false,
    )?;

    // Update upper tick
    let tick_upper_update = tick_upper.calculate_next_liquidity_update(
        position.tick_index_upper,
        pool.current_tick_index,
        pool.fee_growth_0_x32,
        pool.fee_growth_1_x32,
        liquidity_delta,
        productType,
        true,
    )?;

    // Calculate the growth in fees
    let (fee_growth_inside_0_x32, fee_growth_inside_1_x32) = tick_lower.calculate_next_fee_growth(
        position.tick_index_lower,
        tick_upper,
        position.tick_index_upper,
        pool.get_current_tick_index()?,
        pool.fee_growth_0_x32,
        pool.fee_growth_1_x32,
    )?;

    let token_0_delta = calculate_token_0_delta(
        liquidity_delta,
        &position,
        pool.current_tick_index,
        productType,
    )?;

    let token_1_delta = calculate_token_1_delta(
        liquidity_delta,
        &position,
        pool.current_tick_index,
        productType,
    )?;

    Ok(UpdatedLiquidityState {
        liquidity_delta,
        next_liquidity,
        tick_lower_update,
        tick_upper_update,
        token_0_delta,
        token_1_delta,
        fee_growth_inside_0_x32,
        fee_growth_inside_1_x32,
    })
}

pub fn update_liquidity<'info>(
    pool: &mut Account<'info, Pool>,
    position: &mut Account<'info, LiquidityPosition>,
    tick_array_lower: &AccountLoader<'info, TickArray>,
    tick_array_upper: &AccountLoader<'info, TickArray>,
    state: &UpdatedLiquidityState,
) -> Result<()> {
    // update liquidity position
    position.update(
        state.liquidity_delta,
        state.fee_growth_inside_0_x32,
        state.fee_growth_inside_1_x32,
    )?;
    // Update Pool liquidity
    pool.update_liquidity(state.next_liquidity)?;

    tick_array_lower.load_mut()?.update_tick(
        position.tick_index_lower,
        pool.tick_spacing,
        &state.tick_lower_update,
    )?;
    tick_array_upper.load_mut()?.update_tick(
        position.tick_index_upper,
        pool.tick_spacing,
        &state.tick_upper_update,
    )?;

    Ok(())
}

/// Calculate the delta in token 0
///
/// This is based on formula 6.29 in Uniswap v3
/// white paper
pub fn calculate_token_0_delta(
    liquidity_delta: i64,
    liquidity_position: &LiquidityPosition,
    current_tick_index: i32,
    productType: &ProductType,
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
    product_type: &ProductType,
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

    // If the product is smart contract coverage
    // vault 1 is just holding premiums
    if product_type.is_smart_contract_coverage() {
        return Ok(0);
    }

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