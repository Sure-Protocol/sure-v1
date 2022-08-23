use std::{
    cell::RefMut,
    ops::{Add, BitAnd, BitOr, Div, Mul, Shl, Shr, Sub},
};

use crate::{
    factory::{calculate_stake, calculate_stake_x32},
    instructions::validate_stake,
    states::VoteInstruction,
    utils::{uint::U256, *},
};

use anchor_lang::{prelude::*, solana_program::clock};

use super::{RevealedVoteArray, VoteAccount};

#[derive(Debug, Eq, PartialEq, PartialOrd)]
#[repr(C)]
pub enum ProposalStatus {
    /// A vote has been proposed
    Proposed = 1,
    Voting = 2,
    ReachedQuorum = 3,
    RevealVote = 4,
    VoteRevealFinished = 5,
    RewardCalculation = 6,
    RewardPayout = 7,
    Failed = 8,
}

impl Default for ProposalStatus {
    #[inline]
    fn default() -> Self {
        Self::Proposed
    }
}

#[account]
pub struct Proposal {
    /// bump for verification
    pub bump: u8, // 1 byte
    pub bump_array: [u8; 1],
    /// name of vote
    pub name: String, // 4 + 64 bytes
    /// description of vote
    pub description: String, // 4 + 200 bytes
    /// Proposed result
    pub proposed_result: i64,
    /// user who proposed the vote
    pub proposer: Pubkey, // 32 bytes

    /// Token mint to distribute rewards
    pub token_mint_reward: Pubkey,

    /// amount staked by propose Q32.32
    pub proposed_staked: u64, // 16 bytes

    /// vault for storing stake and votes
    pub vault: Pubkey, // 32 bytes

    /// % of ve tokens needed to conclude
    /// represented as basis points 1% = 100bp
    pub required_votes: u64,

    /// Current votes given in basis points
    /// 1 vote = 1 veToken@
    /// Q64.0
    pub votes: u64,
    pub revealed_votes: u64,

    // Q64.0
    pub running_sum_weighted_vote: i64,
    // Q64.0
    pub running_weight: u64,

    /// Start of vote
    pub vote_start_at: i64,
    /// Blind vote deadline
    pub vote_end_at: i64,
    /// start reveal
    pub vote_end_reveal_at: i64,

    /// reward earned by propsing vote
    /// Q64.64
    pub earned_rewards: u128,

    /// Scale parameter in exp(L)
    /// Q16.16
    pub scale_parameter: u32,

    pub scale_parameter_calculated: bool,

    /// when the vote is finished and
    /// users can reap rewards
    pub locked: bool,

    pub vote_factor_sum: u64,
    pub distribution_sum: u128,

    /// Instruction to be exectued if passed
    pub instructions: [VoteInstruction; 32],
}

#[event]
pub struct ProposeVoteEvent {
    pub name: String,
    pub proposer: Pubkey,
}

impl Default for Proposal {
    #[inline]
    fn default() -> Proposal {
        Proposal {
            bump: 0,
            bump_array: [0; 1],
            name: "test proposal".to_string(),
            description: "test descr".to_string(),
            proposed_result: 0,
            proposer: Pubkey::default(),
            token_mint_reward: Pubkey::default(),
            proposed_staked: 0,
            vault: Pubkey::default(),
            required_votes: 100_000,
            votes: 0,
            revealed_votes: 0,
            running_sum_weighted_vote: 0,
            running_weight: 0,
            vote_end_at: 0,
            vote_start_at: 0,
            vote_end_reveal_at: 0,
            earned_rewards: 0,
            scale_parameter: 0,
            scale_parameter_calculated: false,
            locked: false,
            distribution_sum: 0,
            vote_factor_sum: 0,
            instructions: [VoteInstruction::default(); 32],
        }
    }
}

pub struct FinalizeVoteResult {}

impl Proposal {
    pub const SPACE: usize = 1 + 1 + 4 + 64 + 4 + 200 + 32 + 16 + 32;

    pub fn seeds(&self) -> [&[u8]; 3] {
        [
            SURE_ORACLE_SEED.as_bytes().as_ref() as &[u8],
            self.name.as_bytes().as_ref() as &[u8],
            self.bump_array.as_ref(),
        ]
    }

    pub fn initialize(
        &mut self,
        bump: u8,
        name: String,
        description: String,
        proposer: &Pubkey,
        proposed_staked: u64,
        token_mint: &Pubkey,
        token_supply: u64,
        vault: &Pubkey,
        end_time_ts: Option<i64>,
        decimals: u8,
    ) -> Result<()> {
        validate_stake(proposed_staked)?;

        // initialize account
        self.bump = bump;
        self.bump_array = [bump; 1];
        self.name = name;
        self.description = description;
        self.proposer = *proposer;

        // convert to Q32.32
        let proposed_stake_proto = (proposed_staked as f64).div(10_u64.pow(decimals as u32) as f64);
        self.proposed_staked = (proposed_stake_proto.floor() as u64) << 32; // Q32.32
        self.vault = *vault;
        self.token_mint_reward = *token_mint;

        // set end of
        let current_time = Clock::get()?.unix_timestamp;
        self.vote_start_at = current_time;
        self.vote_end_at = match end_time_ts {
            Some(t) => t,
            None => current_time
                .checked_add(VOTING_LENGTH_SECONDS)
                .ok_or(SureError::InvalidVoteEndTime)?,
        };

        self.vote_end_reveal_at = match end_time_ts {
            Some(t) => t,
            None => current_time
                .checked_add(VOTING_LENGTH_SECONDS + VOTING_LENGTH_SECONDS)
                .ok_or(SureError::InvalidVoteEndTime)?,
        };

        self.required_votes = token_supply.div(VOTING_FRACTION_REQUIRED);
        self.votes = 0;
        Ok(())
    }

    /// cast a vote if vote is active
    pub fn cast_vote_at_time(&mut self, vote: RefMut<VoteAccount>, time: i64) -> Result<()> {
        if self.get_status(time).unwrap() == ProposalStatus::Voting {
            self.votes += vote.vote_power as u64;
        }
        Ok(())
    }

    pub fn cancel_vote_at_time(&mut self, vote: RefMut<VoteAccount>, time: i64) -> Result<()> {
        if self.get_status(time).unwrap() == ProposalStatus::Voting {
            self.votes -= vote.vote_power as u64;
        }
        Ok(())
    }

    pub fn has_ended_at_time(&self, current_time: i64) -> bool {
        let enough_votes = self.votes >= self.required_votes;
        let timeouted = current_time > self.vote_end_at;
        timeouted | enough_votes
    }

    pub fn has_ended(&self) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        Ok(self.has_ended_at_time(current_time))
    }

    /// Update the weighted vote sum and the weight sum
    /// - S_n = sum_i^n (w_i x V_i)
    /// - W_N = sum_i^n (w_i)
    pub fn update_running_sum_weighted_vote(&mut self, vote_account: VoteAccount) {
        if vote_account.revealed_vote {
            // u32.0 x q32.32 -> q64.32
            let update_x64 =
                (vote_account.vote_power as u64).mul(vote_account.vote.abs() as u64) as u128;
            let update = (update_x64 >> 32) as i64;
            if vote_account.vote > 0 {
                self.running_sum_weighted_vote += update;
            } else {
                self.running_sum_weighted_vote -= update;
            }
            self.running_weight += vote_account.vote_power as u64;
        }
    }

    pub fn calculate_consensus_(&self, running_sum_weighted_vote: i64, running_weight: u64) -> i64 {
        // Convert i64 -> i64.64
        let positive = running_sum_weighted_vote > 0;
        // convert u64 -> Q64.64
        let wv_x64 = (running_sum_weighted_vote.abs() as u128) << 64;
        // convert u64 -> Q64.64
        let rw_x64 = (running_weight as u128);
        // i64.64
        let consensus = (wv_x64).div(rw_x64) as i128;
        let consensus_ix32 = (consensus >> 32) as i64;
        if positive {
            return consensus_ix32;
        } else {
            return -consensus_ix32;
        }
    }
    /// Consensus
    ///
    /// X_n^bar = (1 / sum_i^n w_i ) * sum_i^n (w_i * v_i)
    ///       = (1/W_n) x S_n
    ///       = S_n / W_n
    ///
    /// ## Returns:
    ///  * consensus: I32.32
    pub fn calculate_consensus(&self) -> i64 {
        self.calculate_consensus_(self.running_sum_weighted_vote, self.running_weight)
    }

    /// Estimate the scale parameter used in the
    /// exponential model
    /// Estimate:
    ///     L_n = W_N / sum_i^n (w_i x v_i - X_n)^2
    pub fn estimate_scale_parameter(&self, revealed_votes: &RevealedVoteArray) -> u32 {
        // i32.32
        let consensus = self.calculate_consensus();
        let sum_squared = revealed_votes.calculate_sum_squared_difference(consensus);

        // Q32.32 -> Q64.64
        let running_weight_Q64 = (self.running_weight as u128) << 32;
        // Q64.64
        let lambda = running_weight_Q64.div(sum_squared as u128);
        // get rid of precision: Q64.64 >> 48 -> Q16.16
        let lambda_q16 = (lambda >> 48) as u32;
        lambda_q16
    }

    /// Calculate
    /// |w x v - X^bar|
    ///
    /// Returns distance : I32.32
    ///
    ///
    pub fn calculate_consensus_distance(&self, vote_account: &VoteAccount) -> i64 {
        // i32.32
        let consensus = self.calculate_consensus();
        println!("consensus: {}", convert_ix32_f64(consensus));
        // i32.32 - i32.32 => i32.32
        println!("vote: {}", convert_ix32_f64(vote_account.vote));
        vote_account.vote - consensus
    }

    /// Calculate the remainder of the
    /// reward pool after a reward is subtracted
    /// # Input
    /// - reward: Q64.64
    /// # Output
    /// - remaining_reward: Q64.64
    pub fn calculate_reward_pool_remainder(&self, reward: u128) -> u128 {
        println!("votes: {}", (self.votes as u128) << 64);
        ((self.votes as u128) << 64) - reward
    }

    /// Calculate: X = l * exp(-l*(x-x^bar)^2)
    ///
    /// ### Output
    /// - X : Q32.32
    ///
    /// TODO: scale down variables earlier
    /// TODO: move out to manager
    pub fn calculate_vote_factor(&self, vote_account: &VoteAccount) -> Result<u64> {
        // validate that voting is done
        // Q64.0
        let proposer_reward = self.calculate_proposer_reward();
        println!("proposer_reward: {}", proposer_reward);
        // Q64.64
        let reward_pool = self.calculate_reward_pool_remainder(proposer_reward);
        println!("reward_pool: {}", reward_pool);

        // calculate distance
        if !vote_account.revealed_vote {
            return Err(SureError::VoteNotRevealed.into());
        }
        // I32.32
        let distance = self.calculate_consensus_distance(vote_account);
        println!("distance f64: {}", convert_ix32_f64(distance));
        // I32.32 x I32.32 -> Q64.64
        let sqrd_distance = (distance as i128).mul(distance as i128) as u128;
        println!("sqrd_distance u128: {}", sqrd_distance);
        // Q64.64 >> 48 -> Q48.16
        if (sqrd_distance >> 48) > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }
        let sqrd_distance_q48_16 = (sqrd_distance >> 48) as u64;

        // Q16.16 x Q48.16  -> Q64.32 => Q128.0
        println!("scale parameter: {}", convert_q16_f16(self.scale_parameter));
        let exponent = (self.scale_parameter as u128).mul(sqrd_distance_q48_16 as u128);
        if exponent > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }
        let exponent_q64 = exponent as u64;
        println!("exponent: {}", exponent_q64);
        println!("exponent f64: {}", convert_ix32_f64(exponent_q64 as i64));
        let pt2_x64 = calculate_exp(exponent_q64, true);
        println!("pt2_x64: {}", pt2_x64);
        println!("pt2_x64 as f64: {}", convert_q64_to_f64(pt2_x64));
        let pt2_x32 = (pt2_x64 >> 48) as u64;
        // CAST DOWN: Q64.64 >> 32 -> Q32.32
        if pt2_x32 > u32::MAX as u64 {
            return Err(SureError::OverflowU64.into());
        }
        // Q64.64 << 48 -> Q48.16 -> Q16.16
        let pt2_q16 = pt2_x32 as u32;
        println!("pt2_q16 as u32: {}", convert_q16_f16(pt2_q16));
        // Q16.16 x Q16.16 -> Q32.32
        let exp_factor = self.scale_parameter.mul(pt2_q16) as u64;
        println!("exp_factor as u32: {}", convert_ix32_f64(exp_factor as i64));
        // Q64.64 >> 64 => u64
        Ok(exp_factor)
    }

    /// Updates the vote factor sum
    ///
    pub fn update_vote_factor_sum(&mut self, vote_factor: u64) {
        self.vote_factor_sum += vote_factor;
    }

    /// Calculate and update the scale parameter
    pub fn update_scale_parameter(&mut self, revealed_votes: &RevealedVoteArray) -> Result<()> {
        self.scale_parameter = self.estimate_scale_parameter(revealed_votes);
        Ok(())
    }

    /// try to finalize the vote after reveal
    pub fn try_finalize_vote_after_reveal(
        &mut self,
        revealed_votes: &RevealedVoteArray,
        time: i64,
    ) -> Result<()> {
        if self.get_status(time).unwrap() == ProposalStatus::VoteRevealFinished {
            // distribute reward to proposer
            let rewards = self.calculate_proposer_reward();
            self.earned_rewards = rewards;

            // calculate scale parameter
            self.update_scale_parameter(revealed_votes)?;
            self.scale_parameter_calculated = true;

            let vote_instruction = self.instructions[0];
            vote_instruction.invoke_proposal()?;
        } else {
            return Err(SureError::RevealPeriodNotActive.into());
        }
        Ok(())
    }

    /// try finalize vote
    /// Only finalize vote if either the
    /// quorum is reached or if it timed out
    pub fn try_finalize_blind_vote(&mut self, time: i64) -> Result<u128> {
        if self.get_status(time).unwrap() == ProposalStatus::RevealVote {
            // distribute rewards
            let rewards = self.calculate_proposer_reward();
            self.earned_rewards = rewards;

            // Initiate instruction
            let vote_instruction = self.instructions[0];
            vote_instruction.invoke_proposal()?;
            return Ok(rewards);
        }

        Ok(0)
    }

    /// get the payout earned by the proposer
    ///
    /// ### Arguments
    /// * decimals: number of decimals in the mint
    /// * time: current time used to check if it's possible to payout
    pub fn payout_earned_rewards_at_time(&mut self, decimals: u8, time: i64) -> Result<u64> {
        if self.get_status(time).unwrap() > ProposalStatus::RevealVote {
            // Q64.64 -> Q64.32
            let rewards_x32 = self.earned_rewards >> 32;
            if rewards_x32 > u64::MAX as u128 {
                return Err(SureError::OverflowU64.into());
            }

            self.earned_rewards = 0;

            return Ok(convert_x32_to_u64(rewards_x32 as u64, decimals));
        } else {
            return Err(SureError::RevealPeriodIsNotFinished.into());
        }
    }

    /// Calculate the reward from the votes
    ///
    /// 0.1% = 10bp of the total votes
    ///
    /// Returns Q64.64
    fn calculate_reward_from_revealed_votes(&self) -> u128 {
        (calculate_stake_x32(self.revealed_votes) as u128) << 64
    }

    /// Calculate reward for proposing vote
    ///
    /// if the vote has ended calculate reward
    ///
    /// Returns
    /// - proposer reward as Q64.64
    pub fn calculate_proposer_reward(&self) -> u128 {
        // if vote is successful
        return (self.proposed_staked as u128) << 64 + self.calculate_reward_from_revealed_votes();
    }

    pub fn is_blind_vote_ongoing_at_time(&self, time: i64) -> bool {
        time >= self.vote_start_at && time < self.vote_end_at
    }

    pub fn is_blind_vote_finished_at_time(&self, time: i64) -> bool {
        time >= self.vote_end_at
    }

    // Is reveal windo
    pub fn is_vote_reveal_ongoing_at_time(&self, time: i64) -> bool {
        time >= self.vote_end_at && time < self.vote_end_reveal_at
    }

    pub fn is_vote_revealed_over(&self, time: i64) -> bool {
        time >= self.vote_end_reveal_at
    }

    pub fn has_reached_quorum(&self) -> bool {
        self.votes >= self.required_votes
    }

    pub fn get_status(&self, time: i64) -> Option<ProposalStatus> {
        if self.is_blind_vote_ongoing_at_time(time) && !self.has_reached_quorum() {
            return Some(ProposalStatus::Voting);
        } else if self.is_blind_vote_finished_at_time(time) && self.has_reached_quorum() {
            return Some(ProposalStatus::ReachedQuorum);
        } else if self.has_reached_quorum() && self.is_vote_reveal_ongoing_at_time(time) {
            return Some(ProposalStatus::RevealVote);
        } else if self.is_vote_revealed_over(time) {
            return Some(ProposalStatus::VoteRevealFinished);
        } else if self.scale_parameter_calculated {
            return Some(ProposalStatus::RewardCalculation);
        } else if self.locked {
            return Some(ProposalStatus::RewardPayout);
        } else {
            return Some(ProposalStatus::Failed);
        }
    }

    /// Check if the proposer can claim reward
    ///
    /// a proposer can claim reward after the reveal period
    /// and the parameter is calculated
    pub fn can_payout_proposer_rewards(&self, time: i64) -> Result<()> {
        if self.get_status(time).unwrap() > ProposalStatus::VoteRevealFinished
            && self.scale_parameter_calculated
        {
            return Ok(());
        } else {
            return Err(SureError::NotPossibleToPayoutProposerReward.into());
        }
    }

    /// can a vote be cancelled
    ///
    /// a vote can be cancelled when in voting period
    pub fn can_cancel_vote(&self, time: i64) -> Result<()> {
        if !(self.get_status(time).unwrap() == ProposalStatus::Voting) {
            return Err(SureError::FailedToCancelVote.into());
        }
        Ok(())
    }
}

// Proto for proposal, a builder
#[cfg(test)]
pub mod test_proposal_proto {
    use anchor_lang::solana_program::vote;

    use super::*;

    #[derive(Default)]
    pub struct ProposalProto {
        pub bump: u8, // 1 byte
        pub bump_array: [u8; 1],
        /// name of vote
        pub name: String, // 4 + 64 bytes
        /// description of vote
        pub description: String, // 4 + 200 bytes

        /// amount staked by propose Q32.32
        pub proposed_staked: u64, // 16 bytes
        proposed_result: i64,

        /// % of ve tokens needed to conclude
        /// represented as basis points 1% = 100bp
        pub required_votes: u64,

        /// Current votes given in basis points
        /// 1 vote = 1 veToken@
        /// Q64.0
        pub votes: u64,
        pub revealed_votes: u64,

        // Q64.0
        pub running_sum_weighted_vote: i64,
        // Q64.0
        pub running_weight: u64,

        /// Start of vote
        pub vote_start_at: i64,
        /// Blind vote deadline
        pub vote_end_at: i64,
        /// start reveal
        pub vote_end_reveal_at: i64,

        /// reward earned by propsing vote
        /// Q64.64
        pub earned_rewards: u128,

        /// Scale parameter in exp(L)
        /// Q16.16
        pub scale_parameter: u32,

        pub scale_parameter_calculated: bool,

        /// when the vote is finished and
        /// users can reap rewards
        pub locked: bool,

        pub vote_factor_sum: u64,

        pub distribution_sum: u128,
    }

    impl ProposalProto {
        pub fn initialize() -> Self {
            Self {
                bump: 0,
                bump_array: [0; 1],
                name: "test".to_string(),
                description: "test".to_string(),
                proposed_result: 0,
                proposed_staked: 1_000_000,
                required_votes: 10_000_000,
                votes: 0,
                revealed_votes: 0,
                running_sum_weighted_vote: 0,
                running_weight: 0,
                vote_start_at: TEST_START_TIME,
                vote_end_at: TEST_START_TIME + 86400,
                vote_end_reveal_at: TEST_START_TIME + 86400 * 2,
                earned_rewards: 0,
                scale_parameter: 0,
                scale_parameter_calculated: false,
                distribution_sum: 0,
                locked: false,
                vote_factor_sum: 0,
            }
        }

        pub fn set_scale_parameter(mut self, scale_parameter: f32) -> Self {
            self.scale_parameter = convert_f32_x16(scale_parameter) as u32;
            self
        }

        pub fn set_required_voted(mut self, required_votes: u64) -> Self {
            self.required_votes = required_votes;
            self
        }

        pub fn set_votes(mut self, votes: u64) -> Self {
            self.votes = votes;
            self
        }

        // get the start of the reveal time
        pub fn get_reveal_time(&self) -> i64 {
            self.vote_end_at + 1
        }

        pub fn set_in_reveal_state(mut self) -> Self {
            // set enough required votes
            self.votes = self.required_votes + 1;
            self
        }

        pub fn build(self) -> Proposal {
            // checkpoint: fill in ny state variables
            Proposal {
                bump: self.bump,
                bump_array: self.bump_array,
                name: self.name,
                description: self.description,
                proposer: Pubkey::default(),
                proposed_result: self.proposed_result,
                token_mint_reward: Pubkey::default(),
                proposed_staked: self.proposed_staked,
                vault: Pubkey::default(),
                required_votes: self.required_votes,
                votes: self.votes,
                revealed_votes: self.revealed_votes,
                running_sum_weighted_vote: self.running_sum_weighted_vote,
                running_weight: self.running_weight,
                vote_start_at: self.vote_start_at,
                vote_end_at: self.vote_end_at,
                vote_end_reveal_at: self.vote_end_reveal_at,
                earned_rewards: self.earned_rewards,
                scale_parameter: self.scale_parameter,
                scale_parameter_calculated: self.scale_parameter_calculated,
                locked: self.locked,
                distribution_sum: self.distribution_sum,
                vote_factor_sum: self.vote_factor_sum,
                instructions: [VoteInstruction::default(); 32],
            }
        }
    }
}

#[cfg(test)]
pub mod test_propose_vote {
    use std::cell::RefCell;

    use crate::states::vote_account_proto;

    use super::*;
    const START_TIME: i64 = 1660681219;

    pub fn create_test_proposal() -> Result<Proposal> {
        let mut proposal = Proposal::default();
        let end_time_ts = 1692182416;
        let proposer = Pubkey::default();
        let stake = 100_000_000;
        let decimals = 6;
        proposal.initialize(
            245,
            "My first proposal".to_string(),
            "protocol lost 25%".to_string(),
            &proposer,
            stake,
            &Pubkey::default(),
            100_000_000_000,
            &Pubkey::default(),
            Some(end_time_ts),
            decimals,
        )?;
        proposal.scale_parameter = convert_f32_x16(0.2) as u32;
        Ok(proposal)
    }

    #[test]
    pub fn test_initialize() {
        create_test_proposal().unwrap();
    }

    #[test]
    pub fn calculate_rewards() {
        pub struct ExpectedResult {
            reward: f64,
        }
        pub struct Test {
            name: String,
            votes: Vec<VoteAccount>,
            expected_result: ExpectedResult,
        }
        let tests = [Test {
            name: "Calculate_rewards. 1. vanilla".to_string(),
            votes: [
                vote_account_proto::VoteAccountProto::initialize()
                    .set_vote_power(3_000_000, 6)
                    .build(),
                vote_account_proto::VoteAccountProto::initialize()
                    .set_vote_power(4_000_000, 6)
                    .build(),
            ]
            .to_vec(),
            expected_result: ExpectedResult { reward: 0.007 },
        }];
        for test in tests {
            let mut proposal = create_test_proposal().unwrap();
            let mut current_time = START_TIME;
            for vote in test.votes {
                proposal
                    .cast_vote_at_time(RefCell::new(vote).borrow_mut(), current_time)
                    .unwrap();

                current_time += 1; // tick
            }

            // test calculations
            let proposal_rewards = proposal.calculate_proposer_reward();
            assert_eq!(
                convert_q64_to_f64(proposal_rewards),
                test.expected_result.reward,
                "{}: get_proposer_reward",
                test.name
            );
        }
    }

    #[test]
    pub fn test_calculate_consensus() {
        pub struct Test {
            name: String,
            weighted_votes: i64,
            weight: u64,
            expected_consensus: f64,
        }
        let tests = [
            Test {
                name: "1. test expected consensus ".to_string(),
                weighted_votes: 20,
                weight: 10,
                expected_consensus: 2.0,
            },
            Test {
                name: "1. test expected consensus ".to_string(),
                weighted_votes: 20,
                weight: 7,
                expected_consensus: 2.8571428570430726,
            },
        ];

        for test in tests {
            let proposal = create_test_proposal().unwrap();

            let consensus = proposal.calculate_consensus_(test.weighted_votes, test.weight);
            println!("cons: {}", convert_ix32_f64(consensus));
            assert_eq!(
                convert_ix32_f64(consensus),
                test.expected_consensus,
                "{}",
                test.name
            )
        }
    }

    /// Calculate the vote reward received
    /// based on consensus and the vote
    #[test]
    pub fn test_calculate_vote_reward() {
        pub struct ExpectedResult {
            reward: f64,
        }
        pub struct Test {
            name: String,
            votes: Vec<VoteAccount>,
            expected_result: ExpectedResult,
        }
        let tests = [Test {
            name: "Calculate_rewards. 1. vanilla".to_string(),
            votes: [
                vote_account_proto::VoteAccountProto::initialize()
                    .set_vote_power(4_000_000, 6)
                    .set_vote_raw(4.0)
                    .build(),
                vote_account_proto::VoteAccountProto::initialize()
                    .set_vote_power(3_000_000, 6)
                    .set_vote_raw(3.0)
                    .build(),
            ]
            .to_vec(),
            expected_result: ExpectedResult {
                reward: 0.19278270285576582,
            },
        }];
        for test in tests {
            let mut proposal = create_test_proposal().unwrap();
            let mut vote_array = RevealedVoteArray::default();
            let mut current_time = START_TIME;
            for vote in test.votes.clone() {
                proposal
                    .cast_vote_at_time(RefCell::new(vote).borrow_mut(), current_time)
                    .unwrap();
                proposal.update_running_sum_weighted_vote(vote);
                vote_array.reveal_vote(&vote).unwrap();
                current_time += 1; // tick
            }

            // test calculations
            let proposal_rewards = proposal
                .calculate_vote_factor(&test.votes[0].clone())
                .unwrap();
            assert_eq!(
                convert_ix32_f64(proposal_rewards as i64),
                test.expected_result.reward,
                "{}: get_proposer_reward",
                test.name
            );
        }
    }
}
