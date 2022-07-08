use crate::common::{
    account::validate_token_account_ownership,
    errors::SureError,
    liquidity::{calculate_token_0_delta, calculate_token_1_delta, validate_liquidity_amount},
};
use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

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
/// create
pub fn build_new_liquidity_state<'info>(
    position: &Account<'info, LiquidityPosition>,
    pool: &Account<'info, Pool>,
    position_token_account: &Account<'info, TokenAccount>,
    tick_array_lower: &AccountLoader<'info, TickArray>,
    tick_array_upper: &AccountLoader<'info, TickArray>,
    amount: u64,
    is_increasing: bool,
) -> Result<UpdatedLiquidityState> {
    if amount == 0 {
        return Err(SureError::LiquidityHaveToBeGreaterThan0.into());
    }
    let productId = pool.productId;

    let liquidity_delta = validate_liquidity_amount(amount, is_increasing)?;

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
        false,
    )?;

    // Update upper tick
    let tick_upper_update = tick_lower.calculate_next_liquidity_update(
        position.tick_index_upper,
        pool.current_tick_index,
        pool.fee_growth_0_x32,
        pool.fee_growth_1_x32,
        liquidity_delta,
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

    let token_0_delta =
        calculate_token_0_delta(liquidity_delta, &position, pool.current_tick_index)?;

    let token_1_delta =
        calculate_token_1_delta(liquidity_delta, &position, pool.current_tick_index)?;

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
