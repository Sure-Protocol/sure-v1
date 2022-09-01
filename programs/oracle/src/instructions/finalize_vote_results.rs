use crate::states::{Proposal, RevealedVoteArray};
use anchor_lang::{prelude::*, solana_program::clock};

use super::reveal_vote;

#[derive(Accounts)]
pub struct FinalizeVoteResults<'info> {
    #[account(mut)]
    pub finalizer: Signer<'info>,

    #[account(mut)]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        has_one = proposal,
    )]
    pub revealed_votes: AccountLoader<'info, RevealedVoteArray>,

    pub system_program: Program<'info, System>,
}

/// Finalize vote
///
/// when the reveal period is over it is time
/// to close the vote and calculate the necessary parameters
/// in order to distribute rewards
///
/// anyone can finalize the vote
pub fn handler(ctx: Context<FinalizeVoteResults>) -> Result<()> {
    let proposal = ctx.accounts.proposal.as_mut();

    let revealed_votes = ctx.accounts.revealed_votes.load()?;
    let time = clock::Clock::get()?.unix_timestamp;

    // check if it's possible to finalie vote result
    proposal.can_finalize_vote_results(time)?;

    proposal.try_finalize_vote_after_reveal(&revealed_votes, time)?;

    emit!(FinalizedVoteResultsEvent {
        proposal: proposal.key(),
        time,
        revealed_votes: proposal.revealed_votes,
        consensus: proposal.consensus,
        status: proposal.status
    });
    Ok(())
}

#[event]
pub struct FinalizedVoteResultsEvent {
    pub proposal: Pubkey,
    pub time: i64,
    pub revealed_votes: u64,
    pub consensus: i64,
    pub status: u8,
}
