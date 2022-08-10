use crate::common::token_tx::withdraw_from_vault;
use crate::common::{
    account,
    errors::SureError,
    liquidity::{calculate_token_0_delta, calculate_token_1_delta, get_liquidity_delta},
};
use crate::common::{
    account::validate_token_account_ownership, liquidity::*, product::*, seeds::*,
};
use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{Token, TokenAccount};
use vipers::*;

use super::increase_liquidity_position::UpdateLiquidityPosition;

/// Decrease Liquidity
///
/// decrease the liquidity in the pool, upper tick and lower tick
///
///
pub fn handler(
    ctx: Context<UpdateLiquidityPosition>,
    liquidity_amount: u128,
    token_min_a: u64,
    token_min_b: u64,
) -> Result<()> {
    // Check that the liquidity provider owns the
    // the liquidity position nft account
    validate_token_account_ownership(
        &ctx.accounts.position_token_account,
        &ctx.accounts.liquidity_provider,
    )?;

    let product_type = ProductType::get_product_type(ctx.accounts.pool.product_id)?;

    let updated_liquidity_state = build_liquidity_state(
        ctx.accounts.liquidity_position.as_ref(),
        ctx.accounts.pool.as_ref(),
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        liquidity_amount,
        &product_type,
        false,
    )?;

    update_liquidity(
        &mut ctx.accounts.pool,
        &mut ctx.accounts.liquidity_position,
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        &updated_liquidity_state,
    )?;
    msg!(&format!(
        "Token 0 delta: {}, token 1 delta {} ",
        updated_liquidity_state.token_0_delta, updated_liquidity_state.token_1_delta
    ));

    // Withdraw from vault to LP
    withdraw_from_vault(
        &ctx.accounts.pool,
        &ctx.accounts.vault_a,
        &ctx.accounts.origin_account_a,
        &ctx.accounts.token_program,
        updated_liquidity_state.token_0_delta,
    )?;

    // Withdraw from vault to LP
    withdraw_from_vault(
        &ctx.accounts.pool,
        &ctx.accounts.vault_b,
        &ctx.accounts.origin_account_b,
        &ctx.accounts.token_program,
        updated_liquidity_state.token_1_delta,
    )?;

    Ok(())
}
