use crate::{
    factory::calculate_stake,
    utils::{convert_x32_to_u64, SureError, SURE_ORACLE_VOTE_SEED, VOTE_STAKE_RATE},
};

use super::{Config, Proposal, ProposalStatus};
use anchor_lang::{prelude::*, solana_program::pubkey};
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Shl, Shr, Sub};

use hex_literal::hex;
use sha3::{Digest, Keccak256FullCore, Sha3_256, Sha3_256Core};
pub const MINT_FACTOR: u32 = 1_000;

pub struct VoteAccountUpdate {
    pub stake_change: u64,
    pub increase_stake: bool,
}
#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, PartialEq)]
pub struct VoteAccount {
    pub bump: u8,            //     1 byte
    pub bump_array: [u8; 1], //     1 byte

    pub proposal: Pubkey,   //      32 bytes
    pub owner: Pubkey,      //      32 bytes
    pub stake_mint: Pubkey, //      32 bytes

    // hash of vote "vote"+"salt"
    pub vote_hash: [u8; 32], //     32 bytes

    /// real vote:
    /// I32.32
    pub vote: i64, //               8 bytes
    pub staked: u64, //             8 bytes

    /// F = V * l * exp(-l*(x-X))
    pub vote_factor: u64, //        8 bytes

    /// rewards earned from voting
    /// C * F / S_v
    pub earned_rewards: u64, //     8 bytes

    // how many votes put on the vote_hash
    // Q32.0 - assume rounded
    pub vote_power: u32, //         8  bytes

    pub revealed_vote: bool, //     1 bytes

    pub locked: bool, //            1 bytes
}

impl Default for VoteAccount {
    #[inline]
    fn default() -> VoteAccount {
        VoteAccount {
            bump: 0,
            bump_array: [0; 1],
            vote_hash: [0; 32],
            owner: Pubkey::default(),
            stake_mint: Pubkey::default(),
            proposal: Pubkey::default(),
            staked: 0,
            vote: 0,
            vote_factor: 0,
            earned_rewards: 0,
            vote_power: 0,
            revealed_vote: false,
            locked: false,
        }
    }
}

impl VoteAccount {
    pub const SPACE: usize = 1 + 1 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 1;

    pub fn seeds(&self) -> [&[u8]; 4] {
        [
            SURE_ORACLE_VOTE_SEED.as_bytes() as &[u8],
            self.proposal.as_ref(),
            self.owner.as_ref(),
            self.bump_array.as_ref(),
        ]
    }

    pub fn initialize(
        &mut self,
        stake_rate: u32,
        bump: u8,
        owner: &Pubkey,
        proposal: &Pubkey,
        vote_hash: &[u8; 32],
        stake_mint: Pubkey,
        vote_power: u64,
        decimals: u8,
    ) -> Result<VoteAccountUpdate> {
        // validate that the user has enough votes

        self.bump = bump;
        self.vote_hash = *vote_hash;

        // f64
        let vote_power_proto = (vote_power).div(10_u64.pow(decimals as u32));
        if vote_power_proto > u32::MAX as u64 {
            return Err(SureError::OverflowU32.into());
        }
        // convert to Q32.0
        let vote_power = vote_power_proto as u32;
        self.vote_power = vote_power;
        self.stake_mint = stake_mint;
        self.staked = calculate_stake((vote_power as u64) << 32, decimals, stake_rate);
        self.vote = 0;
        self.earned_rewards = 0;
        self.owner = *owner;
        self.proposal = *proposal;
        Ok(VoteAccountUpdate {
            stake_change: self.staked,
            increase_stake: true,
        })
    }

    /// Returns vote_power Q32.0
    /// NOTE: consider to be external
    pub fn calculate_vote_power_x32_from_tokens(amount: u64, decimals: u32) -> Result<u32> {
        let vote_power_proto = amount.div(10_u64.pow(decimals));
        if vote_power_proto > u32::MAX as u64 {
            return Err(SureError::OverflowU32.into());
        }
        Ok(vote_power_proto as u32)
    }

    /// Returns vote_power Q32.32
    /// NOTE: consider to be external
    pub fn calculate_vote_power_x64_from_tokens(amount: u64, decimals: u32) -> u64 {
        let vote_power_proto = (amount).div(10_u64.pow(decimals));
        vote_power_proto << 32
    }

    pub fn update_vote_at_time(
        &mut self,
        proposal: &Proposal,
        new_vote_hash: &[u8; 32],
        time: i64,
    ) -> Result<()> {
        // If blind voting is over
        if !(proposal.get_status(time) == ProposalStatus::Voting) {
            return Err(SureError::VotingPeriodEnded.into());
        }
        self.vote_hash = *new_vote_hash;
        Ok(())
    }

    /// reveal vote by proving salt
    pub fn reveal_vote(
        &mut self,
        proposal: &Proposal,
        salt: &str,
        vote: i64,
        time: i64,
    ) -> Result<()> {
        if !(proposal.get_status(time) == ProposalStatus::RevealVote) {
            return Err(SureError::RevealPeriodNotActive.into());
        }
        let mut hasher = Sha3_256::new();
        let message = format!("{}{}", vote, salt);
        hasher.update(message.as_bytes());
        let expected_hash: [u8; 32] = hasher.finalize().try_into().unwrap();
        let vec = expected_hash.to_vec();
        if !vec.eq(&self.vote_hash) {
            return Err(SureError::InvalidSalt.into());
        }
        self.vote = vote;
        self.revealed_vote = true;
        Ok(())
    }

    pub fn cancel_vote(&mut self) -> Result<u64> {
        // lock vote
        self.locked = true;

        // get refund stake
        Ok(self.staked)
    }

    /// Calculate the vote factor
    ///
    /// when votes are revealed the vote factor can be calculated
    /// calculates and sets V = l*exp(-l*x)
    pub fn calculate_vote_factor(&mut self, proposal: &Proposal) -> Result<u64> {
        let vote_factor = proposal.calculate_vote_factor(&self)?;
        self.vote_factor = vote_factor;
        Ok(vote_factor)
    }

    /// Calculate expected reward
    /// Upon an ended vote the voters should
    /// get rewarded or slashed
    ///
    /// Reward is
    /// V * X where
    /// V is the vote_power
    /// X is the exponential result
    ///
    /// ### Returns
    /// - mint reward in Q32.32
    pub fn calculate_token_reward_at_time(
        &mut self,
        proposal: &Proposal,
        mint_decimals: u8,
        time: i64,
    ) -> Result<u64> {
        let status = proposal.get_status(time);
        if self.revealed_vote && self.vote_factor > 0 && status == ProposalStatus::RewardCalculation
        {
            self.calculate_token_reward_(self.vote_factor, mint_decimals)
        } else if status == ProposalStatus::Failed {
            return self.cancel_vote();
        } else {
            return Err(SureError::NotPossibleToCalculateVoteReward.into());
        }
    }

    /// helper for the calculate_token_rewards method
    pub fn calculate_token_reward_(
        &self,
        exponential_value: u64,
        mint_decimals: u8,
    ) -> Result<u64> {
        if self.revealed_vote {
            // Q32.0 x Q32.32 -> Q64.32
            let reward_x64 = (self.vote_power as u128).mul(exponential_value as u128);
            if reward_x64 > u64::MAX as u128 {
                return Err(SureError::OverflowU64.into());
            }
            let reward_x32 = reward_x64 as u64;

            // convert to token mint
            let reward_10 = convert_x32_to_u64(reward_x32, mint_decimals);
            Ok(reward_10)
        } else {
            return Err(SureError::VoteNotRevealed.into());
        }
    }

    /// Calculate the weigted vote
    pub fn calculate_weighted_vote(self) -> Result<i64> {
        // Q32.32 x Q32.32 -> Q64.32
        let weigted_vote_abs = (self.vote_power as u64).mul(self.vote.abs() as u64) as u128;
        // convert Q64.32 -> Q64.0
        let weigted_vote_abs_x32 = (weigted_vote_abs >> 32) as u64;
        if weigted_vote_abs_x32 > i64::MAX as u64 {
            return Err(SureError::OverflowU64.into());
        }
        if self.vote > 0 {
            return Ok(weigted_vote_abs as i64);
        } else {
            return Ok(-(weigted_vote_abs as i64));
        }
    }

    //
    pub fn reset_rewards(&mut self) {
        self.earned_rewards = 0;
    }
}

#[cfg(test)]
pub mod vote_account_proto {
    use crate::utils::convert_f32_i64;

    use super::*;

    pub fn hash_vote(vote: i64, salt: &str) -> [u8; 32] {
        let mut hasher = Sha3_256::new();
        let message = format!("{}{}", vote, salt);
        hasher.update(message.as_bytes());
        let expected_hash: [u8; 32] = hasher.finalize().try_into().unwrap();
        expected_hash
    }
    pub struct VoteAccountProto {
        pub bump: u8, // 1 byte

        // hash of vote "vote"+"salt"
        pub vote_hash: [u8; 32], // 32 bytes

        // real vote:
        pub vote: i64, // 8 bytes

        pub vote_factor: u64,

        pub staked: u64,

        // rewards earned from voting
        pub earned_rewards: u64, // 8 bytes

        // how many votes put on the vote_hash
        pub vote_power: u32, // 8  bytes

        pub revealed_vote: bool, // 1 bytes

        pub locked: bool,
    }
    impl VoteAccountProto {
        pub fn initialize() -> Self {
            Self {
                bump: 0,
                vote_hash: [0; 32],
                vote: 0,
                vote_factor: 0,
                earned_rewards: 0,
                staked: 0,
                vote_power: 0,
                revealed_vote: false,
                locked: false,
            }
        }

        pub fn set_vote_power(mut self, amount: u64, decimals: u32) -> Self {
            self.vote_power =
                VoteAccount::calculate_vote_power_x32_from_tokens(amount, decimals).unwrap();
            self
        }

        pub fn set_vote_hash(mut self, vote: i64, salt: &str) -> Self {
            self.vote_hash = hash_vote(vote, salt);
            self
        }

        pub fn set_vote_raw(mut self, vote: f32) -> Self {
            self.vote = convert_f32_i64(vote);
            self.revealed_vote = true;
            self
        }
        pub fn set_vote(mut self, vote: i64) -> Self {
            self.vote = vote;
            self.revealed_vote = true;
            self
        }

        pub fn build(self) -> VoteAccount {
            VoteAccount {
                bump: self.bump,
                bump_array: [self.bump; 1],
                vote_hash: self.vote_hash,
                vote: self.vote,
                vote_factor: self.vote_factor,
                stake_mint: Pubkey::default(),
                staked: self.staked,
                earned_rewards: self.earned_rewards,
                vote_power: self.vote_power,
                revealed_vote: self.revealed_vote,
                owner: Pubkey::default(),
                proposal: Pubkey::default(),
                locked: self.locked,
            }
        }
    }
}

#[cfg(test)]
pub mod test_vote {
    use proposal::test_proposal_proto::{self, ProposalProto};

    use crate::{states::proposal, utils::convert_f32_i64};

    use super::*;
    const START_TIME: i64 = 1660681219;

    /// Happy path tests
    #[test]
    pub fn vote_on_proposal() {
        #[derive(Default)]
        pub struct ExpectedValue {
            vote: i64,
            weighted_vote: i64,
        }
        #[derive(Default)]
        pub struct Test {
            name: String,
            vote: i64,
            vote_power: u64,
            decimals: u8,
            salt_true: String,
            salt_provided: String,
            start_time: i64,
            proposal: ProposalProto,
            vote_account: VoteAccount,
            expected_value: ExpectedValue,
        }
        let mut vote_account = VoteAccount::default();

        let tests = [
            Test {
                name: "1. provide the right salt".to_string(),
                vote: 400,
                vote_power: 2_300_000,
                decimals: 6,
                salt_true: "a23sw23".to_string(),
                salt_provided: "a23sw23".to_string(),
                start_time: START_TIME,
                proposal: test_proposal_proto::ProposalProto::initialize(),
                vote_account: VoteAccount::default(),
                expected_value: ExpectedValue {
                    vote: 400,
                    weighted_vote: 800,
                },
            },
            Test {
                name: "2. negative vote the right salt".to_string(),
                vote: -400,
                vote_power: 2_300_000,
                decimals: 6,
                salt_true: "a23sw23".to_string(),
                salt_provided: "a23sw23".to_string(),
                start_time: START_TIME,
                proposal: test_proposal_proto::ProposalProto::initialize(),
                vote_account: VoteAccount::default(),
                expected_value: ExpectedValue {
                    vote: -400,
                    weighted_vote: -800,
                },
            },
        ];

        for test in tests {
            let vote_hash = vote_account_proto::hash_vote(test.vote, &test.salt_true);
            vote_account
                .initialize(
                    10,
                    0,
                    &Pubkey::default(),
                    &Pubkey::default(),
                    &vote_hash,
                    Pubkey::default(),
                    test.vote_power,
                    test.decimals,
                )
                .unwrap();
            let mut vote = vote_account.vote;
            let expected_vote = 0;
            let vote_power = vote_account.vote_power;
            assert_eq!(vote, expected_vote);
            assert_eq!(vote_power, 2, "{}: test vote power", test.name);

            let reveal_time = test.proposal.get_reveal_time();
            let proposal = test.proposal.set_in_reveal_state().build();
            vote_account
                .reveal_vote(&proposal, &test.salt_provided, test.vote, reveal_time)
                .unwrap();
            vote = vote_account.vote;
            assert_eq!(
                vote, test.expected_value.vote,
                "{}: reveal vote ",
                test.name
            );
            let weighted_vote = vote_account.calculate_weighted_vote().unwrap();
            assert_eq!(
                weighted_vote, test.expected_value.weighted_vote,
                "{}: calculate expected weight",
                test.name
            );
        }
    }

    /// Not so happy path. Triggers errors
    #[test]
    pub fn vote_on_proposal_errors() {
        pub struct Test {
            name: String,
            vote: i64,
            vote_power: u64,
            decimals: u8,
            salt_true: String,
            salt_provided: String,
            start_time: i64,
            proposal: ProposalProto,
            vote_account: VoteAccount,
            expected_error: SureError,
        }
        let mut vote_account = VoteAccount::default();

        let tests = [Test {
            name: "1. provide the wrong salt".to_string(),
            vote: 400,
            vote_power: 3_000_000,
            decimals: 6,
            salt_true: "a23sw23".to_string(),
            salt_provided: "a23sw24".to_string(),
            start_time: START_TIME,
            proposal: test_proposal_proto::ProposalProto::initialize(),
            vote_account: VoteAccount::default(),
            expected_error: SureError::InvalidSalt,
        }];

        for test in tests {
            let vote_hash = vote_account_proto::hash_vote(test.vote, &test.salt_true);
            vote_account
                .initialize(
                    10,
                    0,
                    &Pubkey::default(),
                    &Pubkey::default(),
                    &vote_hash,
                    Pubkey::default(),
                    test.vote_power,
                    test.decimals,
                )
                .unwrap();
            let vote = vote_account.vote;
            assert_eq!(vote, 0);

            let reveal_time = test.proposal.get_reveal_time();
            let proposal = test.proposal.set_in_reveal_state().build();
            let err = vote_account
                .reveal_vote(&proposal, &test.salt_provided, test.vote, reveal_time)
                .unwrap_err();
            let expected_err: anchor_lang::error::Error = test.expected_error.into();
            println!("err: {}", err.to_string());
            assert_eq!(err.to_string(), expected_err.to_string(), "{}", test.name);
        }
    }

    #[test]
    pub fn update_vote_test() {
        #[derive(Default)]
        pub struct ExpectedValue {
            vote: i64,
        }
        #[derive(Default)]
        pub struct Test {
            name: String,
            vote: i64,
            vote_power: u64,
            decimals: u8,
            vote_updated: i64,
            salt_true: String,
            salt_provided: String,
            start_time: i64,
            proposal: ProposalProto,
            vote_account: VoteAccount,
            expected_value: ExpectedValue,
        }
        let mut vote_account = VoteAccount::default();

        let tests = [Test {
            name: "1. provide the right salt".to_string(),
            vote: 400,
            vote_updated: 450,
            vote_power: 3_000_000,
            decimals: 6,
            salt_true: "a23sw23".to_string(),
            salt_provided: "a23sw23".to_string(),
            start_time: START_TIME,
            proposal: test_proposal_proto::ProposalProto::initialize(),
            vote_account: VoteAccount::default(),
            expected_value: ExpectedValue { vote: 450 },
        }];

        for test in tests {
            let current_time = START_TIME;
            let vote_hash = vote_account_proto::hash_vote(test.vote, &test.salt_true);
            vote_account
                .initialize(
                    10,
                    0,
                    &Pubkey::default(),
                    &Pubkey::default(),
                    &vote_hash,
                    Pubkey::default(),
                    test.vote_power,
                    test.decimals,
                )
                .unwrap();
            let expected_vote = 0;
            let vote = vote_account.vote;
            assert_eq!(vote, expected_vote, "{} vote equal to 0", test.name);
            let new_vote_hash = vote_account_proto::hash_vote(test.vote_updated, &test.salt_true);
            let reveal_time = test.proposal.get_reveal_time();
            let proposal = test.proposal.set_in_reveal_state().build();
            vote_account
                .update_vote_at_time(&proposal, &new_vote_hash, current_time)
                .unwrap();

            vote_account
                .reveal_vote(
                    &proposal,
                    &test.salt_provided,
                    test.vote_updated,
                    reveal_time,
                )
                .unwrap();
            let vote = vote_account.vote;
            let expected_vote = test.expected_value.vote;
            assert_eq!(vote, expected_vote, "{} updated vote", test.name);
        }
    }

    #[test]
    pub fn test_calculate_token_rewards() {
        #[derive(Default)]
        pub struct ExpectedValue {
            reward: u64,
        }
        #[derive(Default)]
        pub struct Test {
            name: String,
            vote: i64,
            vote_power: u64,
            decimals: u8,
            vote_updated: i64,
            salt_true: String,
            salt_provided: String,
            start_time: i64,
            proposal: ProposalProto,
            vote_account: VoteAccount,
            exponential_value: f32,
            expected_value: ExpectedValue,
        }
        let mut vote_account = VoteAccount::default();

        let tests = [
            Test {
                name: "1. ".to_string(),
                vote: 400,
                vote_updated: 450,
                vote_power: 3_000_000,
                decimals: 6,
                salt_true: "a23sw23".to_string(),
                salt_provided: "a23sw23".to_string(),
                start_time: START_TIME,
                proposal: test_proposal_proto::ProposalProto::initialize(),
                vote_account: VoteAccount::default(),
                exponential_value: 1.2,
                expected_value: ExpectedValue { reward: 3_600_000 },
            },
            Test {
                name: "2. ".to_string(),
                vote: 400,
                vote_updated: 450,
                vote_power: 3_000_000,
                decimals: 6,
                salt_true: "a23sw23".to_string(),
                salt_provided: "a23sw23".to_string(),
                start_time: START_TIME,
                proposal: test_proposal_proto::ProposalProto::initialize(),
                vote_account: VoteAccount::default(),
                exponential_value: 0.98989898989898,
                expected_value: ExpectedValue { reward: 2_969_696 },
            },
        ];

        for test in tests {
            let vote_hash = vote_account_proto::hash_vote(test.vote, &test.salt_true);
            vote_account
                .initialize(
                    10,
                    0,
                    &Pubkey::default(),
                    &Pubkey::default(),
                    &vote_hash,
                    Pubkey::default(),
                    test.vote_power,
                    test.decimals,
                )
                .unwrap();
            let reveal_time = test.proposal.get_reveal_time();
            let proposal = test.proposal.set_in_reveal_state().build();
            vote_account
                .reveal_vote(&proposal, &test.salt_true, test.vote, reveal_time)
                .unwrap();
            let reward = vote_account
                .calculate_token_reward_(
                    convert_f32_i64(test.exponential_value) as u64,
                    test.decimals,
                )
                .unwrap();
            assert_eq!(
                reward, test.expected_value.reward,
                "{} |  equal reward. exponential value: {}, decimals: {}",
                test.name, test.exponential_value, test.decimals
            );
        }
    }
}
