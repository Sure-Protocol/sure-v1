use std::ops::{Shl, Shr};

use super::errors::SureError;
use super::tick_math::get_sqrt_ratio_at_tick;
use super::uint::U256;
use super::*;
use crate::states::{
    liquidity::LiquidityPosition, tick_v2::TickArrayPool, Pool, TickArray, TickUpdate,
};
use anchor_lang::prelude::*;

pub fn validate_liquidity_amount(liquidity_amount: u128, increase: bool) -> Result<i128> {
    if liquidity_amount > i128::MAX as u128 {
        return Err(SureError::LiquidityTooLarge.into());
    }
    Ok(if increase {
        liquidity_amount as i128
    } else {
        -(liquidity_amount as i128)
    })
}

/// Calculate/update new liquidity based on delta
///
/// Formula = delta > 0 : liquidity + delta ? liquidity - abs(delta)
pub fn calculate_new_liquidity(liquidity: u128, delta: i128) -> Result<u128> {
    if delta == 0 {
        return Ok(liquidity);
    }

    if delta > 0 {
        liquidity
            .checked_add(delta as u128)
            .ok_or(SureError::LiquidityOverflow.into())
    } else {
        liquidity
            .checked_sub(delta.abs() as u128)
            .ok_or(SureError::LiquidityUnderflow.into())
    }
}

pub struct UpdatedLiquidityState {
    liquidity_delta: i128,
    next_liquidity: u128,
    tick_lower_update: TickUpdate,
    tick_upper_update: TickUpdate,
    pub token_0_delta: u64,
    pub token_1_delta: u64,
    fee_growth_inside_0_x64: u128,
    fee_growth_inside_1_x64: u128,
}

/// Build new liquidity state
///
/// the new state is used to update all the states
/// involved in the transaction
///
/// TODO: Implement tickArray: TickArrayPool in order to accept only one tick array
pub fn build_new_liquidity_state<'info>(
    position: &Account<'info, LiquidityPosition>,
    pool: &Account<'info, Pool>,
    tick_array_lower: &AccountLoader<'info, TickArray>,
    tick_array_upper: &AccountLoader<'info, TickArray>,
    amount_delta: u128,
    productType: &ProductType,
    is_increasing: bool,
) -> Result<UpdatedLiquidityState> {
    if amount_delta == 0 {
        return Err(SureError::LiquidityHaveToBeGreaterThan0.into());
    }

    msg!("build_new_liquidity_state 1 ");
    // Validate the amount delta and return the +/- delta
    let liquidity_delta = validate_liquidity_amount(amount_delta, is_increasing)?;

    // Calculate Tick changes
    // Get Tick accounts
    msg!("build_new_liquidity_state 2 ");
    let tick_array_lower = tick_array_lower.load_mut()?;
    msg!("build_new_liquidity_state 2.1 ");
    let tick_lower = tick_array_lower.get_tick(position.tick_index_lower, pool.tick_spacing)?;

    msg!("build_new_liquidity_state 2.2 ");
    let tick_array_upper = tick_array_upper.load_mut()?;
    msg!("build_new_liquidity_state 2.3 ");
    let tick_upper = tick_array_upper.get_tick(position.tick_index_upper, pool.tick_spacing)?;

    msg!("build_new_liquidity_state 3 ");
    // Calculate the updated liquidity
    let next_liquidity = pool.get_next_liquidity(&position, liquidity_delta)?;

    msg!("build_new_liquidity_state 4 ");
    // Update lower tick
    let tick_lower_update = tick_lower.calculate_next_liquidity_update(
        position.tick_index_lower,
        pool.current_tick_index,
        pool.fee_growth_0_x64,
        pool.fee_growth_1_x64,
        liquidity_delta,
        productType,
        false,
    )?;

    // Update upper tick
    let tick_upper_update = tick_upper.calculate_next_liquidity_update(
        position.tick_index_upper,
        pool.current_tick_index,
        pool.fee_growth_0_x64,
        pool.fee_growth_1_x64,
        liquidity_delta,
        productType,
        true,
    )?;

    // Calculate the growth in fees
    let (fee_growth_inside_0_x64, fee_growth_inside_1_x64) = tick_lower.calculate_next_fee_growth(
        position.tick_index_lower,
        tick_upper,
        position.tick_index_upper,
        pool.get_current_tick_index()?,
        pool.fee_growth_0_x64,
        pool.fee_growth_1_x64,
    )?;

    msg!(&format!("Liquidity delta: {}", liquidity_delta));
    let token_0_delta = calculate_token_0_delta(
        liquidity_delta,
        position.tick_index_lower,
        position.tick_index_upper,
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
        fee_growth_inside_0_x64,
        fee_growth_inside_1_x64,
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
        state.fee_growth_inside_0_x64,
        state.fee_growth_inside_1_x64,
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
/// TODO: Simply calculations and check if U256 is necessary
pub fn calculate_token_0_delta(
    liquidity_delta: i128,
    tick_index_lower: i32,
    tick_index_upper: i32,
    current_tick_index: i32,
    productType: &ProductType,
) -> Result<u64> {
    msg!("Calculate token 0 delta");
    let tick_index_lower = tick_index_lower;
    let tick_index_upper = tick_index_upper;
    let sqrt_price_0 = get_sqrt_ratio_at_tick(tick_index_lower);
    let sqrt_price_1 = get_sqrt_ratio_at_tick(tick_index_upper);
    // Q64.64 -> Q192.64 -> Q128.128
    let liquidity_delta_u = U256::from(liquidity_delta).shl(64);

    let current_sqrt_price = get_sqrt_ratio_at_tick(current_tick_index);
    println!("1.0 current sqrt price: {}", current_sqrt_price >> 64);
    println!("1.0 sqrt_price_0 sqrt price: {}", sqrt_price_0 >> 64);
    println!("1.0 sqrt_price_1 sqrt price: {}", sqrt_price_1 >> 64);

    let (sqrt_price_lower, sqrt_price_upper) = if sqrt_price_1 > sqrt_price_0 {
        (sqrt_price_0, sqrt_price_1)
    } else {
        (sqrt_price_1, sqrt_price_0)
    };
    msg!(&format!(
        "current_tick_index: {}, tick_index_upper: {}, tick_index_lower: {}",
        current_tick_index, tick_index_upper, tick_index_lower
    ));
    if current_tick_index < tick_index_lower {
        Ok(0)
    } else if current_tick_index >= tick_index_upper {
        msg!(&format!("sqrt_price_lower: {}", sqrt_price_lower));
        let sqrt_diff = U256::from(sqrt_price_upper - sqrt_price_lower).shl(64);
        msg!(&format!("2. sqrt_diff {}", sqrt_diff));
        // TODO: calculations should be done in U64.64 and the result should be 64
        let token_change = liquidity_delta_u
            .checked_mul(sqrt_diff)
            .ok_or(SureError::MultiplictationQ3232Overflow)?;
        msg!(&format!("token change: {}", token_change));

        let token_change_shr = token_change.shr(64 as u8).as_u128();
        msg!(&format!("token change: {}", token_change.shr(64 as u8)));
        msg!(&format!("token_change_shr: {}", token_change_shr));
        if token_change_shr > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }
        Ok(token_change_shr as u64)
    } else {
        msg!(&format!(
            "3.0. sqrt_diff {}",
            current_sqrt_price - sqrt_price_lower
        ));
        // Q192.64
        let sqrt_diff = U256::from(current_sqrt_price - sqrt_price_lower);
        // Q128.128
        let sqrt_diff_shift = sqrt_diff.shl(64 as u8);
        msg!(&format!("3.1. sqrt_diff {}", sqrt_diff));
        // Q128.128 x Q128.128
        let token_change = liquidity_delta_u
            .checked_mul(sqrt_diff_shift)
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .shr(64 as u8) // -> Q.192.64
            .as_u128() // -> 64.64
            .shr(32 as u8); // -> Q64.32

        // Convert to Q32.32
        if token_change > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }

        Ok(token_change as u64)
    }
}

/// Calculate token 1 change
///
/// TODO: Simply calculations and check if U256 is necessary
pub fn calculate_token_1_delta(
    liquidity_delta: i128,
    liquidity_position: &LiquidityPosition,
    current_tick_index: i32,
    product_type: &ProductType,
) -> Result<u64> {
    let tick_index_lower = liquidity_position.tick_index_lower;
    let tick_index_upper = liquidity_position.tick_index_upper;

    let liquidity_delta_u = U256::from(liquidity_delta);

    let sqrt_price_0 = get_sqrt_ratio_at_tick(liquidity_position.tick_index_lower);
    let sqrt_price_1 = get_sqrt_ratio_at_tick(liquidity_position.tick_index_upper);

    let current_sqrt_price = U256::from(get_sqrt_ratio_at_tick(current_tick_index));
    let (sqrt_price_lower, sqrt_price_upper) = if sqrt_price_1 > sqrt_price_0 {
        (U256::from(sqrt_price_0), U256::from(sqrt_price_1))
    } else {
        (U256::from(sqrt_price_1), U256::from(sqrt_price_0))
    };

    // If the product is smart contract coverage
    // vault 1 is just holding premiums
    if product_type.is_smart_contract_coverage() {
        return Ok(0);
    }

    let token_delta = if current_tick_index < tick_index_lower {
        let sqrt_price_lower_reciprocal = U256::from(1 as u8)
            .checked_div(sqrt_price_lower)
            .ok_or(SureError::DivisionQ3232Error)?;

        let sqrt_price_upper_reciprocal = U256::from(1 as u8)
            .checked_div(sqrt_price_upper)
            .ok_or(SureError::DivisionQ3232Error)?;

        let sqrt_price_sub = sqrt_price_lower_reciprocal
            .checked_sub(sqrt_price_upper_reciprocal)
            .ok_or(SureError::SubtractionQ3232Error)?;

        let res = liquidity_delta_u
            .checked_mul(sqrt_price_sub)
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .as_u128();
        if res > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }

        Ok(res as u64)
    } else if current_tick_index >= tick_index_upper {
        Ok(0)
    } else {
        let sqrt_price_upper_reciprocal = U256::from(1 as u8)
            .checked_div(sqrt_price_upper)
            .ok_or(SureError::DivisionQ3232Error)?;

        let sqrt_price_reciprocal = U256::from(1 as u8)
            .checked_div(current_sqrt_price)
            .ok_or(SureError::MultiplictationQ3232Overflow)?;
        let sqrt_price_sub = sqrt_price_reciprocal
            .checked_sub(sqrt_price_upper_reciprocal)
            .ok_or(SureError::SubtractionQ3232Error)?;
        let res = liquidity_delta_u
            .checked_mul(sqrt_price_sub)
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .as_u128();
        if res > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }
        Ok(res as u64)
    };

    token_delta
}

#[cfg(test)]
mod liquidity_test {
    use crate::common::tick_math::get_tick_at_sqrt_ratio;

    use super::*;

    #[test]
    fn test_calculate_token_0_delta() {
        let liquidity_delta = 1_000_000;
        let sqrt_price_lower = (5 as u128) << 64;
        let tick_index_lower = get_tick_at_sqrt_ratio(sqrt_price_lower).unwrap();

        let sqrt_price_upper = (7 as u128) << 64;
        let tick_index_upper = get_tick_at_sqrt_ratio(sqrt_price_upper).unwrap();

        let sqrt_price_current = (6 as u128) << 64;
        let tick_index_current = get_tick_at_sqrt_ratio(sqrt_price_current).unwrap();

        let token_delta = calculate_token_0_delta(
            liquidity_delta,
            tick_index_lower,
            tick_index_upper,
            tick_index_current,
            &ProductType::Coverage,
        )
        .unwrap();

        println!("ok: {}", token_delta >> 32);

        assert_eq!(
            token_delta,
            (1_000_000 as u64) << 32,
            "token delta equal to calculation"
        );
    }
}
