
use std::ops::Mul;

use anchor_lang::prelude::*;
use locked_voter::{Locker, Escrow};

use crate::utils::{SURE, SureError};
use sha3::{Digest,Keccak256FullCore, Sha3_256Core, Sha3_256};
use hex_literal::hex;
use crate::instructions::*;

use super::Proposal;

#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, PartialEq)]
pub struct VoteAccount {
    pub bump: u8, // 1 byte

    // hash of vote "vote"+"salt"
    pub vote_hash: [u8;32], // 32 bytes

    // real vote: 
    pub vote: i64, // 8 bytes 

    // rewards earned from voting
    pub earned_rewards: u64, // 8 bytes

    // how many votes put on the vote_hash
    pub vote_power: u64, // 8  bytes

    pub revealed_vote: bool, // 1 bytes
}

impl Default for VoteAccount {
    #[inline]
    fn default() -> VoteAccount {
        VoteAccount { bump: 0, vote_hash: [0;32], vote: 0, earned_rewards: 0, vote_power: 0,revealed_vote: false }
    }
}

impl VoteAccount {
    pub const SPACE: usize = 1 + 32 + 8 + 8 + 8 +1;

    pub fn initialize(
        &mut self,
        bump: u8,
        owner: &Pubkey,
        vote_hash: &[u8;32],
        vote_power: u64,
    ) -> Result<()> {
        // validate that the user has enough votes

        self.bump = bump;
        self.vote_hash  = *vote_hash;
        self.vote_power = vote_power;
        self.vote = 0;
        self.earned_rewards = 0;
        Ok(())
    }

    pub fn update_vote(&mut self,proposal: Proposal, new_vote_hash: [u8;32]) -> Result<()>{
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
        let expected_hash: [u8;32] = hasher.finalize().try_into().unwrap();
        let vec = expected_hash.to_vec();
        if !vec.eq(&self.vote_hash){
            return Err(SureError::InvalidSalt.into())
        }
        self.vote = vote;
        self.revealed_vote = true;
        Ok(())
    }

    /// Claim ve tokens
    /// Upon an ended vote the voters should
    /// get rewarded or slashed 
    pub fn calculate_ve_token_rewards(&mut self, consensus: i64,reward_pool: u64) {
        if self.revealed_vote{
            let distance = (self.vote - consensus).abs() as u64;
            let reward = 0;
        }

    }

    /// Calculate the weigted vote
    pub fn calculate_weighted_vote(self) -> Result<i128> {
        let weigted_sum_abs = (self.vote.abs() as u128).mul(self.vote_power as u128) as i128;
        if self.vote > 0 {
            return Ok(weigted_sum_abs);
        }else{
            return Ok(-weigted_sum_abs)
        }
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

    //initialize vote account
    let mut vote_account = ctx.accounts.vote_account.load_mut()?;
    let vote_account_bump = *ctx.bumps.get("vote_account").unwrap();
    let vote_hash_bytes:&[u8;32] = vote_hash.as_bytes().try_into().unwrap();
    vote_account.initialize(vote_account_bump, &ctx.accounts.voter.key(), vote_hash_bytes, voting_power)?;
    Ok(())
}

#[cfg(test)]
pub mod vote_account_proto {
    use super::*;

    pub fn hash_vote(vote:i64,salt: &str) -> [u8;32] {
        let mut hasher = Sha3_256::new();
        let message = format!("{}{}",vote,salt);
        hasher.update(message.as_bytes());
        let expected_hash:[u8;32] = hasher.finalize().try_into().unwrap();
        expected_hash
    }
    pub struct VoteAccountProto{
        pub bump: u8, // 1 byte

        // hash of vote "vote"+"salt"
        pub vote_hash: [u8;32], // 32 bytes

        // real vote: 
        pub vote: i64, // 8 bytes 

        // rewards earned from voting
        pub earned_rewards: u64, // 8 bytes

        // how many votes put on the vote_hash
        pub vote_power: u64, // 8  bytes

        pub revealed_vote: bool, // 1 bytes
    }
    impl VoteAccountProto {
        pub fn initialize() -> Self {
            Self { bump: 0, vote_hash: [0;32], vote: 0, earned_rewards: 0, vote_power: 0,revealed_vote: false }
        }

        pub fn set_vote_power(mut self,vote_power: u64) -> Self {
            self.vote_power = vote_power;
            self
        }

        pub fn set_vote_hash(mut self, vote: i64, salt: &str) -> Self {
            self.vote_hash = hash_vote(vote, salt);
            self
        }

        pub fn set_vote(mut self, vote: i64 ) -> Self {
            self.vote = vote;
            self.revealed_vote = true;
            self
        }

        pub fn build(self) -> VoteAccount {
            VoteAccount { bump: self.bump, vote_hash: self.vote_hash, vote: self.vote, earned_rewards: self.earned_rewards, vote_power: self.vote_power, revealed_vote: self.revealed_vote }
        }
    }
}

#[cfg(test)]
pub mod test_vote {
    use super::*;

    /// Happy path tests 
    #[test]
    pub fn vote_on_proposal() {

        #[derive(Default)]
        pub struct ExpectedValue {
            vote: i64,
            weighted_vote: i128,
        }
        #[derive(Default)]
        pub struct Test {
            name: String,
            vote: i64,
            vote_power: u64,
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
                vote_power: 2_300_000,
                salt_true: "a23sw23".to_string(),
                salt_provided:  "a23sw23".to_string(),
                proposal: test_propose_vote::create_test_proposal().unwrap(),
                vote_account: VoteAccount::default(),
                expected_value: ExpectedValue{
                    vote: 400,
                    weighted_vote: 920000000,
                }
            },
            Test{
                name: "2. negative vote the right salt".to_string(),
                vote: -400,
                vote_power: 2_300_000,
                salt_true: "a23sw23".to_string(),
                salt_provided:  "a23sw23".to_string(),
                proposal: test_propose_vote::create_test_proposal().unwrap(),
                vote_account: VoteAccount::default(),
                expected_value: ExpectedValue{
                    vote: -400,
                    weighted_vote: -920000000,
                }
            },
        ];

        for test in tests {
            let vote_hash =vote_account_proto::hash_vote(test.vote, &test.salt_true);
            vote_account.initialize(0, &Pubkey::default(),&vote_hash , test.vote_power).unwrap();
            assert_eq!(vote_account.vote,0);
            vote_account.reveal_vote(&test.salt_provided, test.vote).unwrap();
            assert_eq!(vote_account.vote,test.expected_value.vote,"{}",test.name);
            let weighted_vote = vote_account.calculate_weighted_vote().unwrap();
            assert_eq!(weighted_vote,test.expected_value.weighted_vote);
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
            let vote_hash =vote_account_proto::hash_vote(test.vote, &test.salt_true);
            vote_account.initialize(0, &Pubkey::default(),&vote_hash , 0).unwrap();
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