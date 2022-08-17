use std::ops::{BitAnd, Div, Mul, Shl, Shr};

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

/// Validate that the stake is large enough
///
pub fn validate_stake(stake: u64) -> Result<()> {
    if stake < MINIMUM_STAKE {
        return Err(SureError::StakeTooLittle.into());
    }

    Ok(())
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

    /// amount staked by propose
    pub proposed_staked: u64, // 16 bytes

    /// vault for storing stake and votes
    pub vault: Pubkey, // 32 bytes

    /// % of ve tokens needed to conclude
    /// represented as basis points 1% = 100bp
    pub quorum_votes: u64,

    /// Current votes given in basis points
    pub votes: u64,

    pub running_sum_weighted_vote: u128,
    pub running_weight: u128,

    /// deadline for vote
    pub vote_end_ts: i64,

    pub is_active: bool,

    pub is_successful: bool,

    /// reward earned by propsing vote
    pub earned_rewards: u64,

    /// Scale parameter in exp(L)
    pub scale_parameter: u128,

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
    ) -> Result<()> {
        validate_stake(proposed_staked)?;

        // initialize account
        self.bump = bump;
        self.bump_array = [bump; 1];
        self.name = name;
        self.description = description;
        self.proposer = *proposer;
        self.proposed_staked = proposed_staked;
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
            self.votes += vote.vote_power;
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
    pub fn update_running_sum_weighted_vote(&mut self, vote: VoteAccount) {
        if vote.revealed_vote {
            if vote.vote > 0 {
                self.running_sum_weighted_vote += (vote.vote_power as u128).mul(vote.vote as u128);
            } else {
                self.running_sum_weighted_vote -= (vote.vote_power as u128).mul(-vote.vote as u128);
            }
            self.running_weight += vote.vote_power as u128;
        }
    }

    /// Consensus
    ///
    /// X_n^bar = (1 / sum_i^n w_i ) * sum_i^n (w_i * v_i)
    ///       = (1/W_n) x S_n
    ///       = S_n / W_n
    pub fn calculate_consensus(&self) -> u64 {
        self.running_sum_weighted_vote.div(self.running_weight) as u64
    }

    /// Estimate the scale parameter used in the
    /// exponential model
    /// Estimate:
    ///     L_n = W_N / sum_i^n (w_i x v_i - X_n)^2
    pub fn estimate_scale_parameter(&self, revealed_votes: RevealedVoteArray) -> Result<u128> {
        let consensus = self.calculate_consensus();
        let sum_squared = revealed_votes.calculate_sum_squared_difference(consensus);
        let running_weight_x128 = U256::from(self.running_weight).shr(128 as u8);
        let sum_squared_x128 = U256::from(sum_squared).shr(128 as u8);
        let res = running_weight_x128.div(sum_squared_x128);
        let frac_part = res.bitand(U256::from(u128::MAX)).as_u128().shr(64 as u8);
        let real_part = res.shr(128 as u8).as_u128();
        if real_part as u64 > u64::MAX {
            return Err(SureError::OverflowU64.into());
        }
        // Q64.64
        Ok(real_part + frac_part)
    }

    /// Calculate and update the scale parameter
    pub fn update_scale_parameter(&mut self, revealed_votes: RevealedVoteArray) -> Result<()> {
        self.scale_parameter = self.estimate_scale_parameter(revealed_votes)?;
        Ok(())
    }

    /// try finalize vote
    /// Only finalize vote if either the
    /// quorum is reached or if it timed out
    pub fn try_finalize_vote_at_time(&mut self, current_time: i64) -> Result<u64> {
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
    fn get_proposer_reward(&self) -> u64 {
        let votes_x64 = (self.votes as u128) << 64;
        let reward_x64 = votes_x64.div((1_000 as u128) << 64);
        reward_x64 as u64
    }

    /// Calculate reward for proposing vote
    ///
    /// if the vote has ended calculate reward
    pub fn calculate_proposer_reward(&mut self) -> u64 {
        // if vote is successful
        return self.proposed_staked + self.get_proposer_reward();
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

    proposal.initialize(
        proposal_bump,
        name,
        description,
        &ctx.accounts.proposer.key(),
        stake,
        &ctx.accounts.stake_account.key(),
        None,
    )?;
    Ok(())
}

#[cfg(test)]
pub mod test_propose_vote {
    use super::*;
    use crate::instructions::{test_vote, vote_account_proto};
    const START_TIME: i64 = 1660681219;

    pub fn create_test_proposal() -> Result<Proposal> {
        let mut proposal = Proposal::default();
        let end_time_ts = 1692182416;
        let proposer = Pubkey::default();
        let stake = 100_000_000;
        proposal.initialize(
            245,
            "My first proposal".to_string(),
            "protocol lost 25%".to_string(),
            &proposer,
            stake,
            &Pubkey::default(),
            Some(end_time_ts),
        )?;
        Ok(proposal)
    }

    #[test]
    pub fn test_initialize() {
        create_test_proposal().unwrap();
    }

    #[test]
    pub fn calculate_rewards() {
        let mut proposal = create_test_proposal().unwrap();
        let vote_1 = vote_account_proto::VoteAccountProto::initialize()
            .set_vote_power(3_000_000)
            .build();
        let vote_2 = vote_account_proto::VoteAccountProto::initialize()
            .set_vote_power(4_000_000)
            .build();
        proposal.cast_vote_at_time(vote_1, START_TIME).unwrap();
        proposal.cast_vote_at_time(vote_2, START_TIME + 1).unwrap();

        let proposal_rewards = proposal.get_proposer_reward();
        assert_eq!(proposal_rewards, 7_000);
    }
}
