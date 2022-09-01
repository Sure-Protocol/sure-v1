use anchor_lang::{prelude::*, solana_program::clock};

use crate::states::{Proposal, VoteAccount};
use crate::utils::SURE_ORACLE_SEED;
#[derive(Accounts)]
pub struct FinalizeVote<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = vote_account.load()?.proposal == proposal.key(),
        constraint = vote_account.load()?.owner == signer.key()
    )]
    pub vote_account: AccountLoader<'info, VoteAccount>,

    #[account(
        seeds = [
            SURE_ORACLE_SEED.as_bytes().as_ref(),
            proposal.id.as_ref(), // checkpoint - don't use name as seed 
        ],
        bump,
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    pub system_program: Program<'info, System>,
}

/// prepare vote reward
///
/// when the reveal period is over and the scale parameter is calculated
/// the user can calculate the vote factor F = l*exp(-l*(x-X))
pub fn handler(ctx: Context<FinalizeVote>) -> Result<()> {
    let mut vote_account = ctx.accounts.vote_account.load_mut()?;
    let proposal = ctx.accounts.proposal.as_ref();
    let time = clock::Clock::get()?.unix_timestamp;

    // check if vote can be finalized
    proposal.can_finalize_vote(time)?;

    vote_account.calculate_vote_factor(proposal)?;

    emit!(FinalizedVoteEvent {
        proposal: proposal.key(),
        time,
    });
    Ok(())
}

#[event]
pub struct FinalizedVoteEvent {
    pub proposal: Pubkey,
    pub time: i64,
}
