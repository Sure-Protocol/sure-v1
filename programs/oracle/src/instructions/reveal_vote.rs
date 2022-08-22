use crate::states::{Proposal, RevealedVoteArray, VoteAccount};
use crate::utils::{SURE_ORACLE_SEED, SURE_ORACLE_VOTE_SEED};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;

#[derive(Accounts)]
pub struct RevealVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        seeds = [
            SURE_ORACLE_SEED.as_bytes().as_ref(),
            proposal.name.as_bytes().as_ref(),
        ],
        bump = proposal.bump,
        constraint = proposal.key() == vote_account.load()?.proposal
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        mut,
        seeds = [
            SURE_ORACLE_SEED.as_bytes().as_ref(),
            proposal.key().as_ref(),
        ],
        bump = reveal_vote_array.load()?.bump,
    )]
    pub reveal_vote_array: AccountLoader<'info, RevealedVoteArray>,

    #[account(
        mut,
        seeds= [
            SURE_ORACLE_VOTE_SEED.as_ref(),
            proposal.key().as_ref(),
            voter.key().as_ref(),
        ],
        bump = vote_account.load()?.bump,
        constraint = *vote_account.to_account_info().owner == voter.key()
    )]
    pub vote_account: AccountLoader<'info, VoteAccount>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RevealVote>, salt: String, vote: i64) -> Result<()> {
    let mut vote_account = ctx.accounts.vote_account.load_mut()?;
    let mut reveal_vote_array = ctx.accounts.reveal_vote_array.load_mut()?;
    let proposal = ctx.accounts.proposal.as_ref();
    let current_time = clock::Clock::get()?.unix_timestamp;

    // reveal vote in vote account
    vote_account.reveal_vote(proposal, &salt, vote, current_time)?;

    // reveal vote in reveal vote list
    reveal_vote_array.reveal_vote(&vote_account)?;
    Ok(())
}
