use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{states::Proposal, utils::token};

#[derive(Accounts)]
pub struct CollectProposerReward<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(mut)]
    pub proposer_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(mut)]
    pub proposal_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub proposal_vault_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CollectProposerReward>) -> Result<()> {
    let proposal = ctx.accounts.proposal.as_mut();
    let decimals = ctx.accounts.proposal_vault_mint.decimals;
    let time = clock::Clock::get()?.unix_timestamp;

    // check if the proposer can claim payout
    proposal.can_payout_proposer_rewards(time)?;

    // get reward
    let reward = proposal.payout_earned_rewards_at_time(decimals, time)?;

    // payout
    token::withdraw_from_vault(
        &proposal,
        &ctx.accounts.proposal_vault,
        &ctx.accounts.proposer_account,
        &ctx.accounts.token_program,
        reward,
    )?;

    Ok(())
}
