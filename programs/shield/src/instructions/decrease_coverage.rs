use std::sync::Arc;

use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use sure_common::token::burn_tokens;

use crate::state::*;

#[derive(Accounts)]
pub struct DecreaseCoverage<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        constraint = coverage_position.mint.key() == position_mint.key() // check that it is the correct mint
    )]
    pub coverage_position: Box<Account<'info, CoveragePosition>>,

    #[account()]
    pub position_mint: Account<'info, Mint>,

    #[account(
        constraint = position_nft.owner == provider.key(),
        constraint = position_nft.mint == position_mint.key()
    )]
    pub position_nft: Account<'info, TokenAccount>,

    pub orderbook: OrderBook<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

/// decrease coverage position
///
///
pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, DecreaseCoverage>,
    amount: u64,
) -> Result<()> {
    // state update
    let coverage_position = ctx.accounts.coverage_position.as_mut();
    let coverage_change = coverage_position.decrease_coverage(amount)?;
    let ob = &ctx.accounts.orderbook;

    if coverage_change.cancel_amount > 0 {
        let order_summary = ob.cancel_order(coverage_position.order_id)?;
    }
    if coverage_change.provided_coverage_reduction > 0 {
        // push order to orderbook
        let order_summary = ob.push_order(
            u64::MAX,
            coverage_change.provided_coverage_reduction,
            10 << 32,
            Side::Ask,
            &ctx.accounts.provider.key(),
            false,
            false,
        )?;
    }

    // burn coverage tokens
    burn_tokens(
        coverage_change.provided_coverage_reduction,
        &ctx.accounts.pool,
        &ctx.accounts.position_nft,
        &ctx.accounts.position_mint,
        &ctx.accounts.token_program,
    )?;

    Ok(())
}
