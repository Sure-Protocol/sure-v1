use std::borrow::Borrow;

use crate::common::{account, token_tx::deposit_into_vault};
use crate::states::{
    tick_v2::{TickArray, TickArrayPool},
    CoveragePosition, Pool,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, ID};

/// Increase Coverage Position
///
/// Allow users to bind k times liquidity for a longer period of time
/// within
///
/// In cover
#[derive(Accounts)]
pub struct ChangeCoveragePosition<'info> {
    /// Position owner
    pub owner: Signer<'info>,

    /// Token owner account
    #[account(mut)]
    pub token_owner_account_0: Account<'info, TokenAccount>,

    /// Pool to buy insurance from
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    /// Position Mint
    pub position_mint: Account<'info, Mint>,

    /// Position Token account
    pub position_token_account: Account<'info, TokenAccount>,

    /// Coverage Position
    #[account(
        mut,
        constraint = coverage_position.load()?.position_mint == position_mint.key()
    )]
    pub coverage_position: AccountLoader<'info, CoveragePosition>,

    /// Coverage position owner token account
    #[account(
        constraint = coverage_position_owner_token_account_0.mint == pool.token_mint_0,
    )]
    pub coverage_position_owner_token_account_0: Box<Account<'info, TokenAccount>>,

    /// Token vault 0 to buy insurance from
    #[account(
        mut,
        constraint = token_vault_0.mint == coverage_position_owner_token_account_0.mint,
        constraint = token_vault_0.key() == pool.token_vault_0
    )]
    pub token_vault_0: Box<Account<'info, TokenAccount>>,

    /// Token vault 1 to deposit premium into
    /// Constraint: should be of same mint as token vault 0
    #[account(mut,
    constraint =token_vault_1.mint == token_vault_0.mint )]
    pub token_vault_1: Box<Account<'info, TokenAccount>>,

    /// Tick array 0
    /// First array to buy insurance from and
    /// where the current price is located
    #[account(mut,has_one = pool)]
    pub tick_array_0: AccountLoader<'info, TickArray>,

    /// Tick array 1
    /// Array after tick array 0 to buy from
    #[account(mut,has_one = pool)]
    pub tick_array_1: AccountLoader<'info, TickArray>,

    /// Tick array 2
    /// Array after tick array 1 to buy from
    #[account(mut,has_one = pool)]
    pub tick_array_2: AccountLoader<'info, TickArray>,

    /// Token program to transfer tokens
    #[account(address = ID)]
    pub token_program: Program<'info, Token>,
}

/// Increase Coverage Position handler
///
/// Increase the amount coveraged by moving from lower to
/// upper part of tick arrays.
///
/// Assume that current price is at the first available tick array
///
/// Premium is paid into seperate premium vault.  
/// The premium can be collected at any time
pub fn handler(
    ctx: Context<ChangeCoveragePosition>,
    coverage_amount: u128,
    expiry_ts: i64,
    is_target_amount: bool,
) -> Result<()> {
    let pool = ctx.accounts.pool.as_mut();
    let coverage_buyer = &ctx.accounts.owner;
    let premium_vault = &ctx.accounts.token_vault_1;
    let coverage_buyer_account = &ctx.accounts.token_owner_account_0;
    let coverage_position = ctx.accounts.coverage_position.load_mut()?;

    // Validate the coverage position
    account::validate_token_account_ownership(
        &ctx.accounts.position_token_account,
        &ctx.accounts.owner,
    )?;

    // Combine input tick arrays into a tick array pool to buy insurance from
    let tick_array_pool = TickArrayPool::new(
        ctx.accounts.tick_array_0.load_mut().unwrap(),
        ctx.accounts.tick_array_1.load_mut().ok(),
        ctx.accounts.tick_array_2.load_mut().ok(),
    );

    // Calculate the coverage
    let coverage_result = pool.update_coverage(
        tick_array_pool,
        coverage_position,
        coverage_amount,
        expiry_ts,
        false,
        false,
    )?;

    // update pool
    pool.update_after_coverage_change(&coverage_result.borrow())?;

    // ---
    // deposit premium and fees into vault
    let premium_plus_cost = coverage_result.get_total_cost_of_coverage()?;
    deposit_into_vault(
        coverage_buyer,
        premium_vault,
        coverage_buyer_account,
        &ctx.accounts.token_program,
        premium_plus_cost,
    )?;

    Ok(())
}
