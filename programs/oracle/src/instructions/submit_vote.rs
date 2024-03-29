
use std::io::Read;
use std::ops::{Mul, Div};

use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;
use anchor_spl::token::{TokenAccount, Mint, self, Token};
use locked_voter::{Locker, Escrow};

use crate::utils::{SURE, SureError, deposit_into_vault,SURE_ORACLE_VOTE_SEED};
use crate::states::{Proposal,VoteAccount, ProposalStatus};

#[derive(Accounts)]
pub struct SubmitVote<'info> {
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
        //constraint = locker.token_mint.key() == SURE
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
        constraint = proposal.vault == proposal_vault.key(),
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
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}


pub fn handler(ctx:Context<SubmitVote>,vote_hash: Vec<u8>) -> Result<()>{
    let time = Clock::get()?.unix_timestamp;
    let proposal =  &mut ctx.accounts.proposal;
    let locker =&ctx.accounts.locker;
    let voting_power = ctx.accounts.user_escrow.voting_power(&locker.params)?;
    let decimals = ctx.accounts.proposal_vault_mint.decimals;
    
    // check f 
    proposal.can_submit_vote(time)?;

    //initialize vote account
    let mut vote_account = ctx.accounts.vote_account.load_init()?;
    let vote_account_bump = *ctx.bumps.get("vote_account").unwrap();
    let vote_hash_bytes: [u8;32] =vote_hash.clone().try_into().unwrap();
    msg!("vote hash bytes length: {}",vote_hash_bytes.len());
    // Initialize vote account
    let vote_update = vote_account.initialize(proposal.stake_rate,vote_account_bump, &ctx.accounts.voter.key(), &proposal.key(),&vote_hash_bytes, ctx.accounts.proposal_vault_mint.key(),voting_power,decimals)?;

    // Update proposal with vote 
    proposal.cast_vote_at_time(vote_account, time)?;

     // cb: update status of proposal
     proposal.update_status(time);

    // deposit Sure tokens into proposal vote 
    deposit_into_vault(&ctx.accounts.voter, &ctx.accounts.proposal_vault, &ctx.accounts.voter_account, &ctx.accounts.token_program, vote_update.stake_change)?;
    emit!(SubmittedVoteEvent{
        proposal: proposal.key(),
        time,
        vote_hash: vote_hash,
        vote_power: voting_power,
    });
    Ok(())
}

#[event]
pub struct SubmittedVoteEvent {
    pub proposal: Pubkey,
    pub time: i64,
    pub vote_hash: Vec<u8>,
    pub vote_power: u64,
}
