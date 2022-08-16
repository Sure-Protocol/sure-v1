use crate::{states::VoteInstruction, utils::*};

use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use super::VoteAccount;

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

    /// deadline for vote
    pub vote_end_ts: i64,

    pub is_active: bool,

    pub is_successful: bool,

    /// reward earned by propsing vote
    pub earned_rewards: u64,

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
            vote_end_ts: 0,
            is_active: false,
            is_successful: false,
            earned_rewards: 0,
            instructions: [VoteInstruction::default(); 32],
        }
    }
}
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

    /// cast a vote
    pub fn cast_vote(&mut self, vote: VoteAccount) -> Result<()> {
        self.votes += vote.votes;
        // Try finalize
        self.try_finalize_vote()
    }

    pub fn has_ended(&self) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        let enough_votes = self.votes >= self.quorum_votes;
        let timeouted = current_time > self.vote_end_ts;
        Ok(timeouted | enough_votes)
    }

    /// try finalize vote
    /// Only finalize vote if either the
    /// quorum is reached or if it timed out
    pub fn try_finalize_vote(&mut self) -> Result<()> {
        let has_ended = self.has_ended()?;
        let successful = self.votes >= self.quorum_votes;
        if has_ended {
            self.is_active = false
        }
        if successful {
            self.is_successful = true
        }

        // Initiate instruction
        let vote_instruction = self.instructions[0];
        vote_instruction.invoke_proposal()?;
        Ok(())
    }

    /// Calculate reward for proposing vote
    ///
    /// if the vote has ended calculate reward
    pub fn calculate_rewards(&mut self) -> Result<()> {
        if self.has_ended()? {
            self.earned_rewards = self.proposed_staked;
        }
        Ok(())
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
}
