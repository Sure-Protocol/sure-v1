use crate::states::{Proposal, VoteAccount};
use crate::utils::SURE_ORACLE_VOTE_SEED;
use anchor_lang::{prelude::*, solana_program::clock};
#[derive(Accounts)]
pub struct UpdateVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        constraint = proposal.key() == vote_account.load()?.proposal
    )]
    pub proposal: Box<Account<'info, Proposal>>,

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

/// Update the submitted vote before vote is over
///
/// allows users to change the vote hash
///
/// ## Arguments
/// * ctx: UpdateVote
/// * vote_hash: hash as string
///
pub fn handler(ctx: Context<UpdateVote>, vote_hash: String) -> Result<()> {
    let mut vote_account = ctx.accounts.vote_account.load_mut()?;
    let proposal = ctx.accounts.proposal.as_ref();
    let current_time = clock::Clock::get()?.unix_timestamp;
    let vote_hash_bytes: &[u8; 32] = vote_hash.as_bytes().try_into().unwrap();

    vote_account.update_vote_at_time(proposal, vote_hash_bytes, current_time)?;
    Ok(())
}
