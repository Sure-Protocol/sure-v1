use anchor_lang::{
    prelude::*,
    solana_program::{clock, vote},
};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    states::{Proposal, VoteAccount},
    utils::tokenTx,
};

#[derive(Accounts)]
pub struct CancelVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        mut,
        constraint = voter_account.owner == voter.key(),
        constraint = voter_account.mint == proposal_vault_mint.key()
    )]
    pub voter_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = proposal_vault.mint == proposal_vault_mint.key(),
        constraint = proposal_vault.owner == proposal.key()
    )]
    pub proposal_vault: Box<Account<'info, TokenAccount>>,

    pub proposal_vault_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = proposal.vault == proposal_vault.key(),
        constraint = proposal.locked == false
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        mut,
        constraint = vote_account.load()?.proposal == proposal.key(),
        constraint =  vote_account.load()?.owner == voter.key(),
    )]
    pub vote_account: AccountLoader<'info, VoteAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CancelVote>) -> Result<()> {
    let mut vote_account = ctx.accounts.vote_account.load_mut()?;
    let proposal = ctx.accounts.proposal.as_mut();
    let time = clock::Clock::get()?.unix_timestamp;
    let decimals = ctx.accounts.proposal_vault_mint.decimals;
    // check if vote can be cancelled
    proposal.can_cancel_vote(time)?;

    // cancel vote
    let refund = vote_account.cancel_vote(decimals)?;

    // cancel vote in proposal
    proposal.cancel_vote_at_time(vote_account, time)?;

    // refund
    tokenTx::withdraw_from_vault(
        proposal,
        &ctx.accounts.proposal_vault,
        &ctx.accounts.voter_account,
        &ctx.accounts.token_program,
        refund,
    )?;

    emit!(CancelledVote {
        vote: ctx.accounts.vote_account.key(),
        proposal: proposal.key(),
        time: time,
        refund: refund
    });
    Ok(())
}

#[event]
pub struct CancelledVote {
    vote: Pubkey,
    proposal: Pubkey,
    time: i64,
    refund: u64,
}
