
use std::ops::{Mul, Div};

use anchor_lang::prelude::*;
use locked_voter::{Locker, Escrow};

use crate::utils::{SURE, SureError};
use crate::states::{Proposal,VoteAccount};

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        constraint = locker.token_mint.key() == SURE
    )]
    pub locker: Account<'info,Locker>,

    #[account(
        constraint = user_escrow.owner == voter.key(),
        constraint = user_escrow.amount > 0 
    )]
    pub user_escrow: Account<'info,Escrow>,

    #[account(
        constraint = proposal.has_ended()? == false
    )]
    pub proposal: Account<'info,Proposal>,

    #[account(
        init,
        payer = voter,
        seeds = [
            &"sure-oracle-vote".as_ref(),
            proposal.key().as_ref(),
            voter.key().as_ref(),
        ],
        bump,
        space = 8 + VoteAccount::SPACE
    )]
    pub vote_account: AccountLoader<'info,VoteAccount>,

    pub system_program: Program<'info,System>
}


pub fn handler(ctx:Context<VoteOnProposal>,vote_hash: String) -> Result<()>{
    let proposal = &ctx.accounts.proposal;
    if proposal.has_ended()? {
        return Err(SureError::VotingPeriodEnded.into())
    }
    let locker =&ctx.accounts.locker;
    let voting_power = ctx.accounts.user_escrow.voting_power(&locker.params)?;
    let decimals = 6;
    //initialize vote account
    let mut vote_account = ctx.accounts.vote_account.load_mut()?;
    let vote_account_bump = *ctx.bumps.get("vote_account").unwrap();
    let vote_hash_bytes:&[u8;32] = vote_hash.as_bytes().try_into().unwrap();
    vote_account.initialize(vote_account_bump, &ctx.accounts.voter.key(), vote_hash_bytes, voting_power,decimals)?;
    Ok(())
}

