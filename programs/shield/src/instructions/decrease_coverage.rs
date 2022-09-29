use std::sync::Arc;

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

use crate::state::CoveragePosition;

#[derive(Accounts)]
pub struct DecreaseCoverage<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

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

    pub system_program: Program<'info, System>,
}

/// decrease coverage position
///
///
pub fn handler(ctx: Context<DecreaseCoverage>, amount: u64) -> Result<()> {
    // state update
    let coverage_position = ctx.accounts.coverage_position.as_mut();
    let coverage_change = coverage_position.decrease_coverage(amount)?;

    Ok(())
}
