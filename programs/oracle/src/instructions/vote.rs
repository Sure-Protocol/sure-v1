
use anchor_lang::prelude::*;
use locked_voter::{Locker,Escrow};

use crate::utils::{SURE, SureError};
use sha3::{Digest,Keccak256FullCore, Sha3_256Core, Sha3_256};
use hex_literal::hex;
use super::{Proposal};

#[account]
pub struct VoteAccount {
    pub bump: u8,

    // owner of vote
    pub owner: Pubkey,

    // hash of vote "vote"+"salt"
    pub vote_hash: Vec<u8>,

    // real vote: 
    pub vote: i64,

    // how many votes put on the vote_hash
    pub votes: u64,
}

impl VoteAccount {
    pub const SPACE: usize = 1 + 32 + 4;

    pub fn initialize(
        &mut self,
        bump: u8,
        owner: &Pubkey,
        vote_hash: String,
        votes: u64,
    ) -> Result<()> {
        // validate that the user has enough votes

        self.bump = bump;
        self.owner = *owner;
        let res  = vote_hash.into_bytes();
        self.votes = votes;
        self.vote = 0;
        Ok(())
    }

    pub fn update_vote(&mut self,proposal: Proposal, new_vote_hash: String) -> Result<()>{
        if proposal.has_ended()?{
            return Err(SureError::VotingPeriodEnded.into())
        }
        self.vote_hash = new_vote_hash.into_bytes();
        Ok(())
    }

    /// Reveal vote by decrypting vote hash 
    pub fn reveal_vote(&mut self,salt: u32,vote: i64) -> Result<()>{
        let mut hasher = Sha3_256::new();
        let message = format!("{}{}",vote,salt);
        hasher.update(message.as_bytes());
        let expected_hash = hasher.finalize();
        let vec = expected_hash.to_vec();
        if !vec.eq(&self.vote_hash){
            return Err(SureError::InvalidSalt.into())
        }

        self.vote = vote;

        Ok(())
    }
}

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
    pub vote_account: Box<Account<'info,VoteAccount>>,

    pub system_program: Program<'info,System>
}


pub fn handler(ctx: Context<VoteOnProposal>,vote_hash: String) -> Result<()>{
    let proposal = ctx.accounts.proposal;
    if proposal.has_ended()? {
        return Err(SureError::VotingPeriodEnded.into())
    }
    let locker = ctx.accounts.locker;
    let voting_power = ctx.accounts.user_escrow.voting_power(&locker.params)?;

    // initialize vote account
    let vote_account = ctx.accounts.vote_account;
    let vote_account_bump = *ctx.bumps.get("vote_account").unwrap();
    vote_account.initialize(vote_account_bump, &ctx.accounts.voter.key(), vote_hash, voting_power)?;
    Ok(())
}

