use std::ops::{Shl, Shr};

use super::errors::SureError;
use super::tick_math::get_sqrt_ratio_at_tick;
use super::uint::U256;
use super::*;
use crate::states::{
    liquidity::LiquidityPosition, tick_v2::TickArrayPool, Pool, TickArray, TickUpdate,
};
use anchor_lang::prelude::*;

pub fn get_liquidity_delta(liquidity_amount: u128, increase: bool) -> Result<i128> {
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
    pub liquidity_delta: i128, // Q64.64
    pub next_liquidity: u128,
    pub tick_lower_update: TickUpdate,
    pub tick_upper_update: TickUpdate,
    pub token_0_delta: u64,
    pub token_1_delta: u64,
    pub fee_growth_inside_0_x64: u128,
    pub fee_growth_inside_1_x64: u128,
}

/// Build liquidity state based on which
/// product is being used
pub fn build_liquidity_state<'info>(
    position: &Account<'info, LiquidityPosition>,
    pool: &Account<'info, Pool>,
    tick_array_lower: &AccountLoader<'info, TickArray>,
    tick_array_upper: &AccountLoader<'info, TickArray>,
    amount_delta: u128,
    productType: &ProductType,
    is_increasing: bool,
) -> Result<UpdatedLiquidityState> {
    if productType.is_smart_contract_coverage() {
        build_liquidity_coverage_state(
            position,
            pool,
            tick_array_lower,
            tick_array_upper,
            amount_delta,
            is_increasing,
        )
    } else if productType.is_smart_AMM() {
        build_new_liquidity_AMM_state(
            position,
            pool,
            tick_array_lower,
            tick_array_upper,
            amount_delta,
            productType,
            is_increasing,
        )
    } else {
        return Err(SureError::InvalidProductTypeId.into());
    }
}

/// Build new liquidity coverage state
///
/// For insurance, vault 0 is used for the deposited liquidity
/// while vault 1 is for the premiums paid by the users.
pub fn build_liquidity_coverage_state<'info>(
    position: &Account<'info, LiquidityPosition>,
    pool: &Account<'info, Pool>,
    tick_array_lower: &AccountLoader<'info, TickArray>,
    tick_array_upper: &AccountLoader<'info, TickArray>,
    amount_delta: u128,
    is_increasing: bool,
) -> Result<UpdatedLiquidityState> {
    if amount_delta == 0 {
        return Err(SureError::LiquidityHaveToBeGreaterThan0.into());
    }

    // Validate the amount delta and return the +/- delta
    let liquidity_delta = get_liquidity_delta(amount_delta, is_increasing)?;

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
        pool.fee_growth_0_x64,
        pool.fee_growth_1_x64,
        liquidity_delta,
        &ProductType::Coverage,
        false,
    )?;

    // Update upper tick
    let tick_upper_update = tick_upper.calculate_next_liquidity_update(
        position.tick_index_upper,
        pool.current_tick_index,
        pool.fee_growth_0_x64,
        pool.fee_growth_1_x64,
        liquidity_delta,
        &ProductType::Coverage,
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

    // Calculate amount of deposit into vault 0
    let token_0_delta = calculate_token_0_delta(
        liquidity_delta,
        position.tick_index_lower,
        position.tick_index_upper,
        pool.current_tick_index,
        &ProductType::Coverage,
    )?;

    // Vault 1 is for premiums
    let token_1_delta = 0;

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

/// Build new liquidity state for AMM
///
/// the new state is used to update all the states
/// involved in the transaction
///
/// TODO: Implement tickArray: TickArrayPool in order to accept only one tick array
/// TODO: Make it work for productType=1 (coverage)
pub fn build_new_liquidity_AMM_state<'info>(
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

    // Validate the amount delta and return the +/- delta
    let liquidity_delta = get_liquidity_delta(amount_delta, is_increasing)?;

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

    let token_0_delta = calculate_token_0_delta(
        liquidity_delta,
        position.tick_index_lower,
        position.tick_index_upper,
        pool.current_tick_index,
        productType,
    )?;

    let token_1_delta = calculate_token_1_delta(
        liquidity_delta,
        position.tick_index_lower,
        position.tick_index_upper,
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
/// TODO: delta calculations is a bit off expected liquidity
pub fn calculate_token_0_delta(
    liquidity_delta: i128,
    tick_index_lower: i32,
    tick_index_upper: i32,
    current_tick_index: i32,
    productType: &ProductType,
) -> Result<u64> {
    msg!("Calculate token 0 delta");
    msg!(&format!("tick_index_lower: {}", tick_index_lower));
    let sqrt_price_0 = get_sqrt_ratio_at_tick(tick_index_lower);
    msg!(&format!(
        "sqrt_price_0: {} sqrt_price_0 >> 64: {}",
        sqrt_price_0,
        sqrt_price_0.shr(64)
    ));
    let sqrt_price_1 = get_sqrt_ratio_at_tick(tick_index_upper);
    msg!(&format!("liquidity_delta: {}", liquidity_delta));
    // Q64.64 -> Q192.64
    let liquidity_delta_u = U256::from(liquidity_delta.abs() as u128).shl(64);
    msg!(&format!("liquidity_delta_u: {}", liquidity_delta_u));
    let current_sqrt_price = get_sqrt_ratio_at_tick(current_tick_index);
    msg!(&format!("current_sqrt_price: {}", current_sqrt_price));
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
        // Q.192.64
        msg!(&format!("sqrt_price_upper Q 64 {}", sqrt_price_0.shr(64)));
        let sqrt_diff = U256::from(sqrt_price_upper - sqrt_price_lower);
        // TODO: calculations should be done in U64.64 and the result should be 64
        msg!(&format!(
            "sqrt_diff x liquidity_delta_u = {} x {}",
            sqrt_diff, liquidity_delta_u
        ));
        let token_change = liquidity_delta_u
            .checked_mul(sqrt_diff)
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .shr(64 as u8) // -> Q.192.64
            .as_u128(); // Q64.64
        msg!(&format!("= token change 0 {}", token_change));
        if token_change.shr(64 as u8) > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }
        Ok(token_change.shr(64 as u8) as u64)
    } else {
        // Q192.64
        let sqrt_diff = U256::from(current_sqrt_price - sqrt_price_lower);
        // Q128.128
        // 192.64 x 192.64 = x.128
        let token_change = liquidity_delta_u
            .checked_mul(sqrt_diff)
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .shr(64 as u8) // -> Q.192.64
            .as_u128(); // -> 64.64
        println!("token_change: {}", token_change);
        // Convert to Q32.32
        if token_change.shr(64 as u8) > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }

        // return q64.0
        Ok(token_change.shr(64 as u8) as u64)
    }
}

/// Calculate token 1 change
///
/// TODO: Simply calculations and check if U256 is necessary
pub fn calculate_token_1_delta(
    liquidity_delta: i128,
    tick_index_lower: i32,
    tick_index_upper: i32,
    current_tick_index: i32,
    product_type: &ProductType,
) -> Result<u64> {
    let liquidity_delta_u = U256::from(liquidity_delta.abs() as u128).shl(64);

    let sqrt_price_0 = get_sqrt_ratio_at_tick(tick_index_lower);
    // Q64.64
    let sqrt_price_1 = get_sqrt_ratio_at_tick(tick_index_upper);

    let current_sqrt_price = get_sqrt_ratio_at_tick(current_tick_index);
    // Q192.64
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
        let sqrt_diff = U256::from(sqrt_price_upper - sqrt_price_lower);
        // Q192.64 x Q 192.64
        let nominator = liquidity_delta_u
            .checked_mul(sqrt_diff)
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .shr(64 as u8); // -> Q.192.64

        let denominator = U256::from(sqrt_price_upper)
            .checked_mul(U256::from(sqrt_price_lower))
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .shr(64 as u8); // -> Q.192.6
        println!("nominator / denominator =  {} / {}", nominator, denominator);
        let ratio = nominator
            .checked_div(denominator)
            .ok_or(SureError::DivisionQ3232Error)?;
        println!("ratio: {}", ratio);
        let token_change = ratio.as_u128(); // -> 64.64
        println!("token_change: {}", token_change);
        if token_change.shr(64 as u8) > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }

        // return q64.0
        Ok(token_change.shr(64 as u8) as u64)
    } else if current_tick_index >= tick_index_upper {
        Ok(0)
    } else {
        // TODO: simply calculations and correct for shifts
        println!("current tick gt tick_index_lower");
        // Q192.64
        let sqrt_diff = U256::from(sqrt_price_upper - current_sqrt_price);
        println!(
            "sqrt_diff x liquidity delta  = {} x {}",
            sqrt_diff, liquidity_delta_u
        );
        // Q192.64 x Q 192.64
        let nominator = liquidity_delta_u
            .checked_mul(sqrt_diff)
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .shr(64 as u8); // -> Q.192.64

        let denominator = U256::from(sqrt_price_upper)
            .checked_mul(U256::from(current_sqrt_price))
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .shr(64 as u8); // -> Q.192.6
        println!("nominator / denominator =  {} / {}", nominator, denominator);
        let ratio = nominator
            .checked_div(denominator)
            .ok_or(SureError::DivisionQ3232Error)?;
        println!("ratio: {}", ratio);
        let token_change = ratio.as_u128(); // -> 64.64
        println!("token_change: {}", token_change);
        if token_change.shr(64 as u8) > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }

        // return q64.0
        Ok(token_change.shr(64 as u8) as u64)
    };

    token_delta
}

#[cfg(test)]
mod liquidity_test {
    use crate::common::tick_math::get_tick_at_sqrt_ratio;

    use super::*;

    // TODO double check that casting is correct
    #[test]
    fn test_calculate_token_0_delta() {
        //Q64.0
        let liquidity_delta: i128 = (1_000_000 as i128) << 64;
        let sqrt_price_lower = (5 as u128) << 64;
        let tick_index_lower = get_tick_at_sqrt_ratio(sqrt_price_lower).unwrap();

        let sqrt_price_upper = (7 as u128) << 64;
        let tick_index_upper = get_tick_at_sqrt_ratio(sqrt_price_upper).unwrap();

        let sqrt_price_current = (6 as u128) << 64;
        let tick_index_current = get_tick_at_sqrt_ratio(sqrt_price_current).unwrap();

        let token_delta_0 = calculate_token_0_delta(
            liquidity_delta,
            tick_index_lower,
            tick_index_upper,
            tick_index_current,
            &ProductType::AMM,
        )
        .unwrap();
        println!(
            "token delta 0: {} , liquidity delta : {}",
            token_delta_0, liquidity_delta
        );
        let token_delta_1 = calculate_token_1_delta(
            liquidity_delta,
            tick_index_lower,
            tick_index_upper,
            tick_index_current,
            &ProductType::AMM,
        )
        .unwrap();
        println!("token delta 1: {}", token_delta_1);
        assert_ne!(token_delta_1, 0);

        assert!(
            (liquidity_delta.shr(64) as u64) - token_delta_0 < 100_000,
            "token delta equal to calculation"
        );
    }
}
