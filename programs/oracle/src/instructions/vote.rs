
use anchor_lang::prelude::*;
use locked_voter::{Locker, Escrow};

use crate::utils::{SURE, SureError};
use sha3::{Digest,Keccak256FullCore, Sha3_256Core, Sha3_256};
use hex_literal::hex;
use crate::instructions::*;

use super::Proposal;

#[account]
pub struct VoteAccount {
    pub bump: u8,

    // owner of vote
    pub owner: Pubkey,

    // hash of vote "vote"+"salt"
    pub vote_hash: Vec<u8>,

    // real vote: 
    pub vote: i64,


    // rewards earned from voting
    pub earned_rewards: u64,

    // how many votes put on the vote_hash
    pub votes: u64,
}

impl Default for VoteAccount {
    #[inline]
    fn default() -> VoteAccount {
        VoteAccount { bump: 0, owner: Pubkey::default(), vote_hash: Vec::new(), vote: 0, earned_rewards: 0, votes: 0 }
    }
}

impl VoteAccount {
    pub const SPACE: usize = 1 + 32 + 4;

    pub fn initialize(
        &mut self,
        bump: u8,
        owner: &Pubkey,
        vote_hash: Vec<u8>,
        votes: u64,
    ) -> Result<()> {
        // validate that the user has enough votes

        self.bump = bump;
        self.owner = *owner;
        self.vote_hash  = vote_hash;
        self.votes = votes;
        self.vote = 0;
        self.earned_rewards = 0;
        Ok(())
    }

    pub fn update_vote(&mut self,proposal: Proposal, new_vote_hash: Vec<u8>) -> Result<()>{
        if proposal.has_ended()?{
            return Err(SureError::VotingPeriodEnded.into())
        }
        self.vote_hash = new_vote_hash;
        Ok(())
    }

    /// reveal vote by proving salt 
    pub fn reveal_vote(&mut self,salt: &str,vote: i64) -> Result<()>{
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

    /// Claim ve tokens
    /// Upon an ended vote the voters should
    /// get rewarded or slashed 
    pub fn calculate_ve_token_rewards() {

    }

    // 
    pub fn reset_rewards(&mut self){
        self.earned_rewards = 0;

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


pub fn handler(ctx:Context<VoteOnProposal>,vote_hash: String) -> Result<()>{
    let proposal = &ctx.accounts.proposal;
    if proposal.has_ended()? {
        return Err(SureError::VotingPeriodEnded.into())
    }
    let locker =&ctx.accounts.locker;
    let voting_power = ctx.accounts.user_escrow.voting_power(&locker.params)?;

    // initialize vote account
    let vote_account = ctx.accounts.vote_account.as_mut();
    let vote_account_bump = *ctx.bumps.get("vote_account").unwrap();
    vote_account.initialize(vote_account_bump, &ctx.accounts.voter.key(), vote_hash.as_bytes().to_vec(), voting_power)?;
    Ok(())
}

#[cfg(test)]
pub mod test_vote_on_proposal {
    use super::*;
    
    pub fn hash_vote(vote:i64,salt: &str) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        let message = format!("{}{}",vote,salt);
        hasher.update(message.as_bytes());
        let expected_hash = hasher.finalize();
        expected_hash.to_vec()
    }

    /// Happy path tests 
    #[test]
    pub fn vote_on_proposal() {

        #[derive(Default)]
        pub struct ExpectedValue {
            vote: i64
        }
        #[derive(Default)]
        pub struct Test {
            name: String,
            vote: i64,
            salt_true: String,
            salt_provided: String,
            proposal: Proposal,
            vote_account: VoteAccount,
            expected_value: ExpectedValue
        }
        let mut vote_account = VoteAccount::default();

        let tests = [
            Test{
                name: "1. provide the right salt".to_string(),
                vote: 400,
                salt_true: "a23sw23".to_string(),
                salt_provided:  "a23sw23".to_string(),
                proposal: test_propose_vote::create_test_proposal().unwrap(),
                vote_account: VoteAccount::default(),
                expected_value: ExpectedValue{
                    vote: 400
                }
            },
        ];

        for test in tests {
            let vote_hash =hash_vote(test.vote, &test.salt_true);
            vote_account.initialize(0, &Pubkey::default(),vote_hash , 0).unwrap();
            assert_eq!(vote_account.vote,0);
            vote_account.reveal_vote(&test.salt_provided, test.vote).unwrap();
            assert_eq!(vote_account.vote,test.expected_value.vote,"{}",test.name);
        }

    }

    /// Not so happy path. Triggers errors
    #[test]
    pub fn vote_on_proposal_errors(){
       
        pub struct Test {
            name: String,
            vote: i64,
            salt_true: String,
            salt_provided: String,
            proposal: Proposal,
            vote_account: VoteAccount,
            expected_error: SureError,
        }
        let mut vote_account = VoteAccount::default();

        let tests = [
            Test{
                name: "1. provide the wrong salt".to_string(),
                vote: 400,
                salt_true: "a23sw23".to_string(),
                salt_provided:  "a23sw24".to_string(),
                proposal: test_propose_vote::create_test_proposal().unwrap(),
                vote_account: VoteAccount::default(),
                expected_error:SureError::InvalidSalt,
            },
        ];

        for test in tests {
            let vote_hash =hash_vote(test.vote, &test.salt_true);
            vote_account.initialize(0, &Pubkey::default(),vote_hash , 0).unwrap();
            assert_eq!(vote_account.vote,0);
            let err = vote_account.reveal_vote(&test.salt_provided, test.vote).unwrap_err();
            let expected_err: anchor_lang::error::Error = test.expected_error.into();
            println!("err: {}",err.to_string());
            assert_eq!(err.to_string(),expected_err.to_string(),"{}",test.name);
        }
    }

    #[test]
    pub fn update_vote_test(){
        #[derive(Default)]
        pub struct ExpectedValue {
            vote: i64
        }
        #[derive(Default)]
        pub struct Test {
            name: String,
            vote: i64,
            vote_updated: i64,
            salt_true: String,
            salt_provided: String,
            proposal: Proposal,
            vote_account: VoteAccount,
            expected_value: ExpectedValue
        }
        let mut vote_account = VoteAccount::default();

        let tests = [
            Test{
                name: "1. provide the right salt".to_string(),
                vote: 400,
                vote_updated: 450,
                salt_true: "a23sw23".to_string(),
                salt_provided:  "a23sw23".to_string(),
                proposal: test_propose_vote::create_test_proposal().unwrap(),
                vote_account: VoteAccount::default(),
                expected_value: ExpectedValue{
                    vote: 450
                }
            },
        ];

        for test in tests {
            // let vote_hash =hash_vote(test.vote, &test.salt_true);
            // vote_account.initialize(0, &Pubkey::default(),vote_hash , 0).unwrap();
            // assert_eq!(vote_account.vote,0);
            // let new_vote_hash = hash_vote(test.vote_updated, &test.salt_true);
            // vote_account.update_vote(test.proposal, new_vote_hash).unwrap();
            // vote_account.reveal_vote(&test.salt_provided, test.vote_updated).unwrap();
            // assert_eq!(vote_account.vote,test.expected_value.vote,"{}",test.name);
        }
    }

}