use std::borrow::Borrow;

use super::increase_coverage_position::*;
use crate::common::{account, token_tx::deposit_into_vault};
use crate::states::tick_v2::TickArrayPool;
use anchor_lang::prelude::*;

/// Decrease Coverage Position handler
///
/// Decrease the amount coveraged by moving from upper to
/// the lower part of coverage position
///
/// Assume that current price is at the first available tick array
///
/// Premium is paid into seperate premium vault.  
/// The premium can be collected at any time
///
/// * parameters
/// - coverage_amount: the amount to decrease the position with
/// - expiry_ts: the expiration of the contract
/// - is_target_amount: is coverage_amount the targetted coverage amount
///                     or the amount to reduce the position with
pub fn handler(
    ctx: Context<UpdateCoveragePosition>,
    coverage_amount: u128,
    expiry_ts: i64,
    is_target_amount: bool,
) -> Result<()> {
    let pool = ctx.accounts.pool.as_mut();
    let coverage_buyer = &ctx.accounts.owner;
    let premium_vault = &ctx.accounts.token_vault_1;
    let coverage_buyer_account = &ctx.accounts.token_account_0;
    let coverage_position = ctx.accounts.coverage_position.load_mut()?;

    // Validate the coverage position
    // account::validate_token_account_ownership(
    //     &ctx.accounts.position_token_account,
    //     &ctx.accounts.owner,
    // )?;

    // let tick_array_pool = TickArrayPool::new(
    //     ctx.accounts.tick_array_0.load_mut().unwrap(),
    //     ctx.accounts.tick_array_1.load_mut().ok(),
    //     ctx.accounts.tick_array_2.load_mut().ok(),
    // );

    // Calculate the coverage
    // let coverage_result = pool.update_coverage(
    //     tick_array_pool,
    //     coverage_position,
    //     coverage_amount,
    //     expiry_ts,
    //     false,
    //     false,
    // )?;

    // // update pool
    // pool.update_after_coverage_change(coverage_result.borrow())?;

    // // ---
    // // deposit premium and fees into vault
    // let premium_plus_cost = coverage_result.get_total_cost_of_coverage()?;
    // deposit_into_vault(
    //     coverage_buyer,
    //     premium_vault,
    //     coverage_buyer_account,
    //     &ctx.accounts.token_program,
    //     premium_plus_cost,
    // )?;

    Ok(())
}
