use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    states::{Config, Proposal},
    utils::tokenTx,
};

#[derive(Accounts)]
pub struct CollectProposerReward<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        constraint = proposer_token_account.mint == proposal_vault_mint.key()
    )]
    pub proposer_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        has_one = proposer,
        has_one = config,
        constraint = proposal.vault == proposal_vault.key()
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        mut,
        constraint = proposal_vault.mint == proposal_vault_mint.key()
    )]
    pub proposal_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = proposal_vault_mint.key() == config.token_mint
    )]
    pub proposal_vault_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CollectProposerReward>) -> Result<()> {
    let proposal = ctx.accounts.proposal.as_mut();
    let decimals = ctx.accounts.proposal_vault_mint.decimals;
    let time = clock::Clock::get()?.unix_timestamp;

    // check if the proposer can claim payout
    proposal.can_collect_proposer_rewards(time)?;

    // get reward
    let reward = proposal.payout_earned_rewards_at_time(decimals, time)?;

    // payout
    tokenTx::withdraw_from_vault(
        &proposal,
        &ctx.accounts.proposal_vault,
        &ctx.accounts.proposer_token_account,
        &ctx.accounts.token_program,
        reward,
    )?;

    emit!(CollectProposerRewardEvent {
        proposal: proposal.key(),
        time,
        reward
    });
    Ok(())
}

#[event]
pub struct CollectProposerRewardEvent {
    pub proposal: Pubkey,
    pub time: i64,
    pub reward: u64,
}
