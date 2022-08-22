
use std::ops::{Mul, Div};

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use anchor_spl::token::{TokenAccount, Mint, self, Token};
use locked_voter::{Locker, Escrow};

use crate::utils::{SURE, SureError, deposit_into_vault,SURE_ORACLE_VOTE_SEED};
use crate::states::{Proposal,VoteAccount, ProposalStatus};

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(
        mut,
        constraint = voter_account.mint == proposal_vault.mint,
        constraint = voter_account.mint == proposal_vault_mint.key(),
        constraint = voter_account.owner ==  voter.key()
    )]
    pub voter_account: Box<Account<'info,TokenAccount>>,

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
        mut,
        constraint = proposal.has_ended()? == false
    )]
    pub proposal: Account<'info,Proposal>,

    #[account(
        mut,
        constraint = proposal.vault == proposal_vault.mint,
    )]
    pub proposal_vault: Box<Account<'info,TokenAccount>>,

    #[account(
        constraint = proposal_vault_mint.key() == proposal_vault.mint
    )]
    pub proposal_vault_mint: Box<Account<'info,Mint>>,

    #[account(
        init,
        payer = voter,
        seeds = [
            SURE_ORACLE_VOTE_SEED.as_ref(),
            proposal.key().as_ref(),
            voter.key().as_ref(),
        ],
        bump,
        space = 8 + VoteAccount::SPACE
    )]
    pub vote_account: AccountLoader<'info,VoteAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


pub fn handler(ctx:Context<VoteOnProposal>,vote_hash: String) -> Result<()>{
    let current_time = Clock::get()?.unix_timestamp;
    let proposal =  &mut ctx.accounts.proposal;
    if !(proposal.get_status(current_time).unwrap() == ProposalStatus::Voting) {
        return Err(SureError::VotingPeriodEnded.into())
    }
    let locker =&ctx.accounts.locker;
    let voting_power = ctx.accounts.user_escrow.voting_power(&locker.params)?;
    let decimals = ctx.accounts.proposal_vault_mint.decimals;
    
    //initialize vote account
    let mut vote_account = ctx.accounts.vote_account.load_mut()?;
    let vote_account_bump = *ctx.bumps.get("vote_account").unwrap();
    let vote_hash_bytes:&[u8;32] = vote_hash.as_bytes().try_into().unwrap();
   
    // Initialize vote account
    let vote_update = vote_account.initialize(vote_account_bump, &ctx.accounts.voter.key(), &proposal.key(),vote_hash_bytes, voting_power,decimals)?;

    // Update proposal with vote 
    proposal.cast_vote_at_time(vote_account, current_time)?;

    // deposit Sure tokens into proposal vote 
    deposit_into_vault(&ctx.accounts.voter, &ctx.accounts.proposal_vault, &ctx.accounts.voter_account, &ctx.accounts.token_program, vote_update.stake_change)?;

    Ok(())
}

