use std::ops::{Add, BitAnd, BitOr, Div, Mul, Shl, Shr, Sub};

use crate::{
    states::VoteInstruction,
    utils::{uint::U256, *},
};

use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use super::{RevealedVoteArray, VoteAccount};

pub const MINIMUM_STAKE: u64 = 3_000_000;
// 1/ln(2)
// Q16.16
pub const DIV_LN2_X64: u64 = 94548;

/// Validate that the stake is large enough
///
pub fn validate_stake(stake: u64) -> Result<()> {
    if stake < MINIMUM_STAKE {
        return Err(SureError::StakeTooLittle.into());
    }

    Ok(())
}

/// Convert from Q64.64 -> Q64
///
/// TODO: move to common lib
pub fn convert_x64_to_u64(reward: u128, decimals: u32) -> u64 {
    let reward_f = convert_q64_to_f64(reward);
    reward_f.mul(10_u64.pow(decimals) as f64).floor() as u64
}

pub fn convert_q64_to_f64(num: u128) -> f64 {
    let fractional_part = num.bitand(u64::MAX as u128) as u128;
    let integer_part = num.bitor(u64::MAX as u128) as u128;

    let fraction = (fractional_part as f64)
        .div(2_u64.pow(32) as f64)
        .div(2_u64.pow(32) as f64);
    let integer = integer_part >> 64;
    (integer as f64) + fraction
}
/// Convert a f64 to Q64.64
pub fn convert_f64_q64(float: f64) -> u128 {
    float
        .mul(2_u64.pow(32) as f64) // Q32.32
        .mul(2_u64.pow(32) as f64) //Q32.64
        .round() as u128
}

pub fn convert_f32_i64(float: f32) -> i64 {
    float
        .mul(2_u64.pow(32) as f32) // Q32.32
        .round() as i64
}

pub fn convert_f32_x16(float: f32) -> i32 {
    float
        .mul(2_u32.pow(16) as f32) // Q32.32
        .round() as i32
}

pub fn convert_q16_f16(num: u32) -> f32 {
    let fractional_part = num.bitand(u16::MAX as u32) as u32;
    let integer_part = num.bitor(u16::MAX as u32) as u32;

    let fraction = (fractional_part as f32).div(2_u32.pow(16) as f32);
    let integer = integer_part >> 16;
    (integer as f32) + fraction
}

pub fn convert_ix32_f64(num: i64) -> f64 {
    let positive = num > 0;
    let num_q32 = num.abs() as u64;

    let fractional_part = num_q32.bitand(u32::MAX as u64) as u64;
    let integer_part = num_q32.bitor(u32::MAX as u64) as u64;

    let fraction = (fractional_part as f64).div(2_u64.pow(32) as f64);
    let integer = integer_part >> 32;
    if positive {
        (integer as f64) + fraction
    } else {
        -((integer as f64) + fraction)
    }
}

/// Calculates exp(x) = 2^(x * (1/ln(2)))
///
/// ### Arguments
/// * x: Q32.32
///
/// ### Returns
/// * exp(x): Q64.64
fn calculate_exp(x: u64, negative: bool) -> u128 {
    println!("calculate_exp");
    // Q32.32 x Q16.16 -> Q48.48 >> 32 => Q16.16
    let exponent = (x.mul(DIV_LN2_X64) >> 32) as u32;
    println!("exponent: {}", exponent);
    let exponent_f = convert_q16_f16(exponent);
    println!("exponentf: {}", exponent_f);
    let exponent_sign = if negative { -exponent_f } else { exponent_f };
    let exp_x = 2_f32.powf(exponent_sign) as f64;
    convert_f64_q64(exp_x)
}

#[cfg(test)]
pub mod test_convert_binary_representation {
    use super::*;

    #[test]
    pub fn test_convert_x64_to_u64() {
        pub struct Test {
            name: String,
            reward_f64: f64,
            // Q64.64
            reward_x64: u128,
            // Q32.0
            decimals: u32,
            expected_res: u64,
        }

        let tests = [
            Test {
                name: "1. test basic".to_string(),
                reward_f64: 10.4,
                reward_x64: 191846138366579343360, //10.4
                decimals: 6,
                expected_res: 10_400_000,
            },
            Test {
                name: "2. test with different decimal".to_string(),
                reward_f64: 10.4,
                reward_x64: 191846138366579343360, //10.4
                decimals: 12,
                expected_res: 10_400_000_000_000,
            },
            Test {
                name: "3. test with wild precision".to_string(),
                reward_f64: 10.4252435424325234235235324,
                reward_x64: 192311799533345931264, //10.4
                decimals: 12,
                expected_res: 10_425_243_542_432,
            },
        ];

        for test in tests {
            let res = convert_x64_to_u64(test.reward_x64, test.decimals);
            let reward = convert_f64_q64(test.reward_f64);
            println!("res: {}, reward_x64: {}", res, reward);
            assert_eq!(
                res, test.expected_res,
                "{}: test expected result",
                test.name
            );
        }
    }

    #[test]
    pub fn test_calculate_exp() {
        pub struct Test {
            name: String,
            x: u64,
            negative: bool,
            expected_res: u128,
        }

        let tests = [
            Test {
                name: "1. test exp(1) ".to_string(),
                x: 1,
                negative: false,
                expected_res: 50143205794491924480, // exp(1)
            },
            Test {
                name: "2. test exp(1) ".to_string(),
                x: 2804833801914,
                negative: false,
                expected_res: 50143205794491924480, // exp(1)
            },
        ];

        for test in tests {
            let res = calculate_exp(test.x, test.negative);
            assert_eq!(res, test.expected_res, "{}", test.name);
        }
    }
}

pub enum VotingStatus {
    Proposed,
    Voting,
    ReachedQuorum,
    Successful,
    Failed,
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
    /// user who proposed the vote
    pub proposer: Pubkey, // 32 bytes

    /// amount staked by propose Q32.32
    pub proposed_staked: u64, // 16 bytes

    /// vault for storing stake and votes
    pub vault: Pubkey, // 32 bytes

    /// % of ve tokens needed to conclude
    /// represented as basis points 1% = 100bp
    pub quorum_votes: u64,

    /// Current votes given in basis points
    /// 1 vote = 1 veToken@
    /// Q64.0
    pub votes: u64,

    // Q64.0
    pub running_sum_weighted_vote: i64,
    // Q64.0
    pub running_weight: u64,

    /// deadline for vote
    pub vote_end_ts: i64,

    pub is_active: bool,

    pub is_successful: bool,

    /// reward earned by propsing vote
    /// Q64.64
    pub earned_rewards: u128,

    /// Scale parameter in exp(L)
    /// Q16.16
    pub scale_parameter: u32,

    /// Instruction to be exectued if passed
    pub instructions: [VoteInstruction; 32],
}

impl Default for Proposal {
    #[inline]
    fn default() -> Proposal {
        Proposal {
            bump: 0,
            bump_array: [0; 1],
            name: "test proposal".to_string(),
            description: "test descr".to_string(),
            proposer: Pubkey::default(),
            proposed_staked: 0,
            vault: Pubkey::default(),
            quorum_votes: 100_000,
            votes: 0,
            running_sum_weighted_vote: 0,
            running_weight: 0,
            vote_end_ts: 0,
            is_active: false,
            is_successful: false,
            earned_rewards: 0,
            scale_parameter: 0,
            instructions: [VoteInstruction::default(); 32],
        }
    }
}

pub struct FinalizeVoteResult {}

pub const SURE_ORACLE_SEED: &str = "sure-oracle";
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
        vault: &Pubkey,
        end_time_ts: Option<i64>,
        decimals: u32,
    ) -> Result<()> {
        validate_stake(proposed_staked)?;

        // initialize account
        self.bump = bump;
        self.bump_array = [bump; 1];
        self.name = name;
        self.description = description;
        self.proposer = *proposer;

        // convert to Q32.32
        let proposed_stake_proto = (proposed_staked as f64).div(10_u64.pow(decimals) as f64);
        self.proposed_staked = (proposed_stake_proto.floor() as u64) << 32;
        self.vault = *vault;

        let duration = clock::SECONDS_PER_DAY as i64;

        self.vote_end_ts = match end_time_ts {
            Some(t) => t,
            None => Clock::get()?
                .unix_timestamp
                .checked_add(duration)
                .ok_or(SureError::InvalidVoteEndTime)?,
        };

        self.quorum_votes = 100_000;
        self.votes = 0;
        self.is_active = true;
        Ok(())
    }

    /// cast a vote if vote is active
    pub fn cast_vote_at_time(&mut self, vote: VoteAccount, current_time: i64) -> Result<()> {
        if !self.has_ended_at_time(current_time) {
            self.votes += vote.vote_power as u64;
        }
        Ok(())
    }

    pub fn has_ended_at_time(&self, current_time: i64) -> bool {
        let enough_votes = self.votes >= self.quorum_votes;
        let timeouted = current_time > self.vote_end_ts;
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
            println!("vote power: {}", vote_account.vote_power);
            println!("vote: {}", vote_account.vote);
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
    pub fn estimate_scale_parameter(&self, revealed_votes: RevealedVoteArray) -> u32 {
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
    pub fn calculate_consensus_distance(&self, vote_account: VoteAccount) -> i64 {
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

    pub fn calculate_vote_reward(&self, vote_account: VoteAccount) -> Result<u128> {
        // validate that voting is done
        // Q64.0
        let proposer_reward = self.get_proposer_reward();
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
        // CAST DOWN: Q64.64 >> 32 -> Q32.32
        if pt2_x64 > u64::MAX as u128 {
            return Err(SureError::OverflowU64.into());
        }
        // !Checkpoint : need to convert to correct Q16.16
        let pt2_q16 = (pt2_x64 >> 32) as u32;
        // Q16.16 x Q16.16 -> Q32.32
        let exp_factor = self.scale_parameter.mul(pt2_q16) as u64;
        // Q64.64 >> 64 => u64
        let reward_pool_u64 = (reward_pool >> 64) as u64;
        // Q64.0 x Q32.32 -> Q96.32
        let reward = reward_pool_u64.mul(exp_factor) as u128;
        Ok(reward)
    }

    /// Calculate and update the scale parameter
    pub fn update_scale_parameter(&mut self, revealed_votes: RevealedVoteArray) -> Result<()> {
        self.scale_parameter = self.estimate_scale_parameter(revealed_votes);
        Ok(())
    }

    /// try finalize vote
    /// Only finalize vote if either the
    /// quorum is reached or if it timed out
    pub fn try_finalize_vote_at_time(&mut self, current_time: i64) -> Result<u128> {
        let has_ended = self.has_ended_at_time(current_time);
        let successful = self.votes >= self.quorum_votes;

        if successful & self.is_active {
            self.is_successful = true;
            self.is_active = false;

            // distribute rewards
            let rewards = self.calculate_proposer_reward();
            self.earned_rewards = rewards;

            // Initiate instruction
            let vote_instruction = self.instructions[0];
            vote_instruction.invoke_proposal()?;
            return Ok(rewards);
        }

        if has_ended {
            self.is_active = false
        }

        Ok(0)
    }

    /// Calculate the reward from the votes
    ///
    /// 0.1% = 10bp of the total votes
    ///
    /// Returns Q64.64
    fn get_proposer_reward(&self) -> u128 {
        // Convert votes to Q64.64
        let votes_x64 = (self.votes as u128) << 64;
        // Reward as Q64.64
        let reward_x64 = votes_x64.div(1_000 as u128);
        reward_x64
    }

    /// Calculate reward for proposing vote
    ///
    /// if the vote has ended calculate reward
    ///
    /// Returns
    /// - proposer reward as Q64.64
    pub fn calculate_proposer_reward(&mut self) -> u128 {
        // if vote is successful
        return (self.proposed_staked as u128) << 64 + self.get_proposer_reward();
    }
}

#[derive(Accounts)]
#[instruction(name: String)]
pub struct ProposeVote<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        init,
        payer = proposer,
        seeds = [
            SURE_ORACLE_SEED.as_bytes().as_ref(),
            name.as_bytes().as_ref(),
        ],
        bump,
        space = 8 + Proposal::SPACE
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        constraint = stake_mint.key() == SURE
    )]
    pub stake_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = proposer,
        associated_token::mint = stake_mint,
        associated_token::authority = proposal,
    )]
    pub stake_account: Box<Account<'info, TokenAccount>>,

    //
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ProposeVote>,
    name: String,
    description: String,
    stake: u64,
) -> Result<()> {
    let proposal = ctx.accounts.proposal.as_mut();
    let proposal_bump = *ctx.bumps.get("proposal").unwrap();
    let decimals = 6;
    proposal.initialize(
        proposal_bump,
        name,
        description,
        &ctx.accounts.proposer.key(),
        stake,
        &ctx.accounts.stake_account.key(),
        None,
        decimals,
    )?;
    Ok(())
}

#[cfg(test)]
pub mod test_propose_vote {
    use super::*;
    use crate::instructions::vote_account_proto;
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
                proposal.cast_vote_at_time(vote, current_time).unwrap();

                current_time += 1; // tick
            }

            // test calculations
            let proposal_rewards = proposal.get_proposer_reward();
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
                    .set_vote_power(3_000_000, 6)
                    .set_vote_raw(3.0)
                    .build(),
                vote_account_proto::VoteAccountProto::initialize()
                    .set_vote_power(4_000_000, 6)
                    .set_vote_raw(4.0)
                    .build(),
            ]
            .to_vec(),
            expected_result: ExpectedResult { reward: 7.0 },
        }];
        for test in tests {
            let mut proposal = create_test_proposal().unwrap();
            let mut vote_array = RevealedVoteArray::default();
            let mut current_time = START_TIME;
            for vote in test.votes.clone() {
                proposal.cast_vote_at_time(vote, current_time).unwrap();
                proposal.update_running_sum_weighted_vote(vote);
                vote_array.reveal_vote(vote).unwrap();
                current_time += 1; // tick
            }

            // test calculations
            let proposal_rewards = proposal
                .calculate_vote_reward(test.votes[0].clone())
                .unwrap();
            assert_eq!(
                convert_q64_to_f64(proposal_rewards),
                test.expected_result.reward,
                "{}: get_proposer_reward",
                test.name
            );
        }
    }
}
