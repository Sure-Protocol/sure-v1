use crate::states::{Config, Proposal, RevealedVoteArray, VoteAccount};
use crate::utils::{SURE_ORACLE_REVEAL_ARRAY_SEED, SURE_ORACLE_SEED, SURE_ORACLE_VOTE_SEED};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;

#[derive(Accounts)]
pub struct RevealVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SURE_ORACLE_SEED.as_bytes().as_ref(),
            proposal.id.as_ref(),
        ],
        bump = proposal.bump,
        constraint = proposal.key() == vote_account.load()?.proposal
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        mut,
        seeds = [
            SURE_ORACLE_REVEAL_ARRAY_SEED.as_bytes().as_ref(),
            proposal.id.as_ref(),
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
        constraint = vote_account.load()?.owner == voter.key()
    )]
    pub vote_account: AccountLoader<'info, VoteAccount>,

    pub system_program: Program<'info, System>,
}

/// reveal vote
///
/// after the voting period is over the user can reveal their vote
///
/// TODO: consider not throwing error, but instead updating state and logging
/// errors
pub fn handler(ctx: Context<RevealVote>, salt: String, vote: i64) -> Result<()> {
    let mut vote_account = ctx.accounts.vote_account.load_mut()?;
    let mut reveal_vote_array = ctx.accounts.reveal_vote_array.load_mut()?;
    let proposal = ctx.accounts.proposal.as_mut();
    let time = clock::Clock::get()?.unix_timestamp;
    msg!(
        "[reveal_vote] time {}, proposal end vote {}, status {:?}",
        time,
        proposal.vote_end_reveal_at,
        proposal.get_status(time)
    );
    // check if can reveal vote
    proposal.can_reveal_vote(time)?;

    // reveal vote in vote account
    vote_account.reveal_vote(proposal, &salt, vote, time)?;

    proposal.update_protocol_fee(vote_account.staked);

    // reveal vote in reveal vote list
    reveal_vote_array.reveal_vote(&vote_account)?;

    emit!(RevealedVoteEvent {
        proposal: proposal.key(),
        time,
        revealed_vote: vote_account.vote,
        vote_power: vote_account.vote_power
    });
    Ok(())
}

#[event]
pub struct RevealedVoteEvent {
    pub proposal: Pubkey,
    pub time: i64,
    pub revealed_vote: i64,
    pub vote_power: u64,
}
