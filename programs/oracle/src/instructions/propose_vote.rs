use crate::{states::VoteInstruction, utils::*};

use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

pub const MINIMUM_STAKE: u128 = (3 as u128) << 64;

/// Validate that the stake is large enough
///
pub fn validate_stake(stake: u128) -> Result<()> {
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
    pub proposed_staked: u128, // 16 bytes

    /// vault for storing stake and votes
    pub vault: Pubkey, // 32 bytes

    /// % of ve tokens needed to conclude
    /// represented as basis points 1% = 100bp
    pub quorum_vote_ratio_required: u32,

    /// Current votes given in basis points
    pub current_vote_ratio: u32,
    pub number_votes: u64,

    /// deadline for vote
    pub vote_end_ts: i64,

    pub is_active: bool,

    /// Instruction to be exectued if passed
    pub instructions: [VoteInstruction; 32],
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
        proposed_staked: u128,
        vault: &Pubkey,
    ) -> Result<()> {
        // validate state change
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
        self.vote_end_ts = Clock::get()?
            .unix_timestamp
            .checked_add(duration)
            .ok_or(SureError::InvalidVoteEndTime)?;

        self.quorum_vote_ratio_required = 3000;
        self.current_vote_ratio = 0;
        self.is_active = true;
        Ok(())
    }

    pub fn has_ended(&mut self) -> Result<bool> {
        let current_time = Clock::get()?.unix_timestamp;
        let enough_votes = self.current_vote_ratio > self.quorum_vote_ratio_required;
        let timeouted = current_time > self.vote_end_ts;
        Ok(timeouted | enough_votes)
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
    stake: u128,
) -> Result<()> {
    let proposal = ctx.accounts.proposal;
    let proposal_bump = *ctx.bumps.get("proposal").unwrap();
    proposal.initialize(
        proposal_bump,
        name,
        description,
        &ctx.accounts.proposer.key(),
        stake,
        &ctx.accounts.stake_account.key(),
    )?;
    Ok(())
}
