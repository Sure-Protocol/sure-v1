use std::{
    cell::RefMut,
    ops::{Add, BitAnd, BitOr, Deref, Div, Mul, Shl, Shr, Sub},
};

use crate::{
    factory::{calculate_stake, calculate_stake_x32},
    instructions::validate_stake,
    utils::{uint::U256, *},
};

use anchor_lang::{prelude::*, solana_program::clock};

use super::{Config, RevealedVoteArray, VoteAccount};

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ProposalStatus {
    /// A vote has been proposed
    Failed = 0,
    Proposed = 1,
    Voting = 2,
    ReachedQuorum = 3,
    RevealVote = 4,
    VoteRevealFinished = 5,
    RewardCalculation = 6,
    RewardPayout = 7,
}

impl ProposalStatus {
    pub fn get_id(self) -> u8 {
        self as u8
    }
}

impl Default for ProposalStatus {
    #[inline]
    fn default() -> Self {
        Self::Proposed
    }
}

#[account]
pub struct Proposal {
    pub config: Pubkey,
    /// bump for verification
    pub bump: u8, // 1 byte
    pub bump_array: [u8; 1], // 1 byte

    /// when the vote is finished and
    /// users can reap rewards
    pub locked: bool, // 1

    /// Optimistic
    pub optimistic: bool, // 1

    pub status: u8, // 1

    /// name of vote
    pub name: String, // 4 + 4*64 bytes

    /// id - hashed name
    pub id: [u8; 32], // 32 bytes

    /// description of vote
    pub description: String, // 4 + 4*140 (140chars) bytes

    /// Proposed result
    pub proposed_result: i64, // 8

    /// user who proposed the vote
    pub proposer: Pubkey, // 32 bytes

    /// 1/x of vote power that must be staked
    pub stake_rate: u32, // 4

    /// amount staked by propose Q32.32
    pub staked: u64, // 8 bytes

    /// vault for storing stake and votes
    pub vault: Pubkey, // 32 bytes

    /// % of ve tokens needed to conclude
    /// represented as basis points 1% = 100bp
    pub required_votes: u64, // 8

    /// as 1/x of revealed vote staked
    pub protocol_fee_rate: u32, // 4

    /// Current votes given in basis points
    /// 1 vote = 1 veToken@
    /// Q64.0
    pub votes: u64, // 8
    pub revealed_votes: u64, // 8

    // Q64.0
    pub running_sum_weighted_vote: i64, // 8
    // Q64.0
    pub running_weight: u64, // 8

    /// Start of vote
    pub vote_start_at: i64, // 8
    /// Blind vote deadline
    pub vote_end_at: i64, // 8
    /// start reveal
    pub vote_end_reveal_at: i64, // 8

    /// reward earned by propsing vote
    /// Q64.64
    pub earned_rewards: u128, // 16

    /// protocol fees
    pub protocol_fees: u128, // 16

    /// Scale parameter in exp(L)
    /// Q16.16
    pub scale_parameter: u32, // 4

    pub scale_parameter_calculated: bool, // 1

    pub vote_factor_sum: u64,   // 8
    pub distribution_sum: u128, // 16

    pub consensus: i64, // 8
}

impl Default for Proposal {
    #[inline]
    fn default() -> Proposal {
        Proposal {
            config: Pubkey::default(),
            bump: 0,
            bump_array: [0; 1],
            name: "test proposal".to_string(),
            id: [0; 32],
            optimistic: false,
            description: "test descr".to_string(),
            proposed_result: 0,
            proposer: Pubkey::default(),
            stake_rate: 10,
            staked: 0,
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
            protocol_fees: 0,
            scale_parameter: 0,
            protocol_fee_rate: 50,
            scale_parameter_calculated: false,
            locked: false,
            status: ProposalStatus::Proposed.get_id(),
            distribution_sum: 0,
            vote_factor_sum: 0,
            consensus: 0,
        }
    }
}

pub struct FinalizeVoteResult {}

impl Proposal {
    pub const SPACE: usize = 1 * 6 + 4 * 3 + 8 * 12 + 32 * 4 + 32 * 3 + 4 + 4 * 64 + 4 + 4 * 140;

    pub fn seeds(&self) -> [&[u8]; 3] {
        [
            SURE_ORACLE_SEED.as_bytes().as_ref() as &[u8],
            self.id.as_ref() as &[u8],
            self.bump_array.as_ref(),
        ]
    }

    pub fn initialize(
        &mut self,
        config: &Account<Config>,
        bump: u8,
        name: String,
        id: &[u8; 32],
        description: &str,
        proposer: &Pubkey,
        proposed_staked: u64,
        vault: &Pubkey,
        end_time_ts: Option<i64>,
    ) -> Result<()> {
        validate_stake(proposed_staked)?;

        // initialize account
        self.bump = bump;
        self.bump_array = [bump; 1];
        self.name = name;
        self.id = *id;
        self.description = String::from(description);
        self.proposer = *proposer;
        self.status = ProposalStatus::Proposed.get_id();
        if proposed_staked < config.minimum_proposal_stake {
            return Err(SureError::NotEnoughProposalStake.into());
        }
        self.staked = proposed_staked; // Q32.32
        self.vault = *vault;
        self.stake_rate = config.vote_stake_rate;
        self.protocol_fee_rate = config.protocol_fee_rate;
        self.optimistic = false;
        self.config = config.key();
        // set end of
        let current_time = Clock::get()?.unix_timestamp;
        self.vote_start_at = current_time;
        msg!(
            "[config_init] current_time: {}, voting length: {}",
            current_time,
            config.voting_length_seconds
        );
        self.vote_end_at = match end_time_ts {
            Some(t) => t,
            None => current_time
                .checked_add(config.voting_length_seconds)
                .ok_or(SureError::InvalidVoteEndTime)?,
        };
        msg!("[config_init] vote_end_at: {}", self.vote_end_at);

        self.vote_end_reveal_at = match end_time_ts {
            Some(t) => t,
            None => self
                .vote_end_at
                .checked_add(config.reveal_length_seconds)
                .ok_or(SureError::InvalidVoteEndTime)?,
        };

        msg!(
            "[config_init] vote_end_reveal_at: {}",
            self.vote_end_reveal_at
        );
        self.required_votes = config.default_required_votes;
        msg!("[config_init] Required votes: {}", self.required_votes);
        self.votes = 0;
        self.protocol_fees = 0;
        Ok(())
    }

    /// cast a vote if vote is active
    pub fn cast_vote_at_time(&mut self, vote: RefMut<VoteAccount>, time: i64) -> Result<()> {
        if self.get_status(time) == ProposalStatus::Voting {
            self.votes += vote.vote_power as u64;
        }
        Ok(())
    }

    pub fn cancel_vote_at_time(&mut self, vote: RefMut<VoteAccount>, time: i64) -> Result<()> {
        if self.get_status(time) == ProposalStatus::Voting {
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
    pub fn estimate_scale_parameter(
        &self,
        consensus: i64,
        revealed_votes: &RevealedVoteArray,
    ) -> u32 {
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
        let consensus = self.calculate_consensus();
        self.scale_parameter = self.estimate_scale_parameter(consensus, revealed_votes);
        self.consensus = consensus;
        Ok(())
    }

    /// try to finalize the vote after reveal
    pub fn try_finalize_vote_after_reveal(
        &mut self,
        revealed_votes: &RevealedVoteArray,
        time: i64,
    ) -> Result<()> {
        if self.get_status(time) == ProposalStatus::VoteRevealFinished {
            // distribute reward to proposer
            let rewards = self.calculate_proposer_reward();
            self.earned_rewards = rewards;

            // calculate scale parameter
            self.update_scale_parameter(revealed_votes)?;
            self.scale_parameter_calculated = true;
        } else {
            return Err(SureError::RevealPeriodNotActive.into());
        }
        Ok(())
    }

    /// try finalize vote
    /// Only finalize vote if either the
    /// quorum is reached or if it timed out
    pub fn try_finalize_blind_vote(&mut self, time: i64) -> Result<u128> {
        if self.get_status(time) == ProposalStatus::RevealVote {
            // distribute rewards
            let rewards = self.calculate_proposer_reward();
            self.earned_rewards = rewards;

            // Initiate instruction
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
        if self.get_status(time) > ProposalStatus::RevealVote {
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

    /// update protocol fee
    ///
    /// when a user reveals the vote the protocol takes a cut
    pub fn update_protocol_fee(&mut self, amount: u64) {
        let protocol_fee = amount.div(self.protocol_fee_rate as u64);
        self.protocol_fees += protocol_fee as u128;
    }

    /// payout protocol fees
    ///
    /// get the accrued protocol fees and
    /// set it to zero
    pub fn payout_accrued_protocol_fees(&mut self) -> Result<u64> {
        let protocol_fees = (self.protocol_fees >> 64);
        if protocol_fees > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }
        self.protocol_fees = 0;
        Ok(protocol_fees as u64)
    }

    /// Calculate the reward from the votes
    ///
    /// 0.1% = 10bp of the total votes
    ///
    /// Returns Q64.64
    fn calculate_reward_from_revealed_votes(&self) -> u128 {
        (calculate_stake_x32(self.revealed_votes, self.stake_rate) as u128) << 64
    }

    /// Update status callback
    pub fn update_status(&mut self, time: i64) {
        let status = self.get_status(time);
        self.status = status.get_id();
    }

    /// Calculate reward for proposing vote
    ///
    /// if the vote has ended calculate reward
    ///
    /// Returns
    /// - proposer reward as Q64.64
    pub fn calculate_proposer_reward(&self) -> u128 {
        // if vote is successful
        return (self.staked as u128) << 64 + self.calculate_reward_from_revealed_votes();
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

    pub fn get_status(&self, time: i64) -> ProposalStatus {
        if self.is_blind_vote_ongoing_at_time(time) && !self.has_reached_quorum() {
            return ProposalStatus::Voting;
        } else if self.has_reached_quorum() && self.is_blind_vote_ongoing_at_time(time) {
            return ProposalStatus::ReachedQuorum;
        } else if self.has_reached_quorum() && self.is_vote_reveal_ongoing_at_time(time) {
            return ProposalStatus::RevealVote;
        } else if self.is_vote_revealed_over(time) {
            return ProposalStatus::VoteRevealFinished;
        } else if self.scale_parameter_calculated {
            return ProposalStatus::RewardCalculation;
        } else if self.locked {
            return ProposalStatus::RewardPayout;
        } else {
            return ProposalStatus::Failed;
        }
    }

    /// checks if a user can submit a vote
    pub fn can_submit_vote(&self, time: i64) -> Result<()> {
        if self.get_status(time) > ProposalStatus::Voting {
            return Err(SureError::VotingPeriodEnded.into());
        }
        Ok(())
    }

    /// can_reveal_vote if
    pub fn can_reveal_vote(&self, time: i64) -> Result<()> {
        if self.get_status(time) != ProposalStatus::RevealVote
            && self.get_status(time) != ProposalStatus::ReachedQuorum
        {
            return Err(SureError::RevealPeriodNotActive.into());
        }
        Ok(())
    }

    pub fn can_finalize_vote(&self, time: i64) -> Result<()> {
        if self.get_status(time) != ProposalStatus::RewardCalculation {
            return Err(SureError::FailedToFinalizeVote.into());
        }
        Ok(())
    }

    pub fn can_finalize_vote_results(&self, time: i64) -> Result<()> {
        if self.get_status(time) != ProposalStatus::VoteRevealFinished {
            return Err(SureError::FailedToFinalizeVoteResult.into());
        }
        Ok(())
    }

    /// Check if the proposer can claim reward
    ///
    /// a proposer can claim reward after the reveal period
    /// and the parameter is calculated
    pub fn can_collect_proposer_rewards(&self, time: i64) -> Result<()> {
        if self.get_status(time) >= ProposalStatus::RewardCalculation
            && self.scale_parameter_calculated
        {
            return Ok(());
        } else {
            return Err(SureError::NotPossibleToCollectProposerReward.into());
        }
    }

    pub fn can_collect_voter_reward(&self, time: i64) -> Result<()> {
        let status = self.get_status(time);
        if status == ProposalStatus::Failed {
            return Ok(());
        }
        if status != ProposalStatus::RewardPayout {
            return Err(SureError::NotPossibleToCollectVoterReward.into());
        }

        Ok(())
    }

    /// can a vote be cancelled
    ///
    /// a vote can be cancelled when in voting period
    pub fn can_cancel_vote(&self, time: i64) -> Result<()> {
        if !(self.get_status(time) == ProposalStatus::Voting) {
            return Err(SureError::FailedToCancelVote.into());
        }
        Ok(())
    }
}

// Proto for proposal, a builder
#[cfg(test)]
pub mod test_proposal_proto {

    use super::*;

    #[derive(Default)]
    pub struct ProposalProto {
        pub bump: u8, // 1 byte
        pub bump_array: [u8; 1],
        /// name of vote
        pub name: String, // 4 + 64 bytes
        pub id: [u8; 32],
        /// description of vote
        pub description: String, // 4 + 200 bytes

        /// amount staked by propose Q32.32
        pub staked: u64, // 16 bytes
        pub proposed_result: i64,

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
        pub status: u8,

        /// reward earned by propsing vote
        /// Q64.64
        pub earned_rewards: u128,
        pub protocol_fee_rate: u32,

        pub protocol_fees: u128,
        /// Scale parameter in exp(L)
        /// Q16.16
        pub scale_parameter: u32,

        pub scale_parameter_calculated: bool,

        /// when the vote is finished and
        /// users can reap rewards
        pub locked: bool,

        pub vote_factor_sum: u64,
        pub consensus: i64,

        pub distribution_sum: u128,
    }

    impl ProposalProto {
        pub fn initialize() -> Self {
            Self {
                bump: 0,
                bump_array: [0; 1],
                name: "test".to_string(),
                id: [0; 32],
                description: "test".to_string(),
                proposed_result: 0,
                staked: 1_000_000,
                required_votes: 10_000_000,
                votes: 0,
                protocol_fee_rate: 50,
                status: ProposalStatus::Proposed.get_id(),
                revealed_votes: 0,
                running_sum_weighted_vote: 0,
                running_weight: 0,
                vote_start_at: TEST_START_TIME,
                vote_end_at: TEST_START_TIME + 86400,
                vote_end_reveal_at: TEST_START_TIME + 86400 * 2,
                earned_rewards: 0,
                protocol_fees: 0,
                scale_parameter: 0,
                scale_parameter_calculated: false,
                distribution_sum: 0,
                locked: false,
                consensus: 0,
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
                config: Pubkey::default(),
                bump: self.bump,
                bump_array: self.bump_array,
                name: self.name,
                id: self.id,
                description: self.description,
                proposer: Pubkey::default(),
                proposed_result: self.proposed_result,
                staked: self.staked,
                optimistic: false,
                stake_rate: 10,
                protocol_fee_rate: self.protocol_fee_rate,
                vault: Pubkey::default(),
                required_votes: self.required_votes,
                votes: self.votes,
                revealed_votes: self.revealed_votes,
                running_sum_weighted_vote: self.running_sum_weighted_vote,
                running_weight: self.running_weight,
                vote_start_at: self.vote_start_at,
                status: self.status,
                vote_end_at: self.vote_end_at,
                vote_end_reveal_at: self.vote_end_reveal_at,
                earned_rewards: self.earned_rewards,
                protocol_fees: self.protocol_fees,
                scale_parameter: self.scale_parameter,
                scale_parameter_calculated: self.scale_parameter_calculated,
                locked: self.locked,
                consensus: self.consensus,
                distribution_sum: self.distribution_sum,
                vote_factor_sum: self.vote_factor_sum,
            }
        }
    }
}

#[cfg(test)]
pub mod test_propose_vote {
    use std::cell::RefCell;

    use crate::states::{test_proposal_proto::ProposalProto, vote_account_proto};

    use super::*;
    const START_TIME: i64 = 1660681219;

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
            let mut proposal = ProposalProto::initialize().build();
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
            let proposal = ProposalProto::initialize().build();

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
            let mut proposal = ProposalProto::initialize().build();
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
