use anchor_lang::prelude::*;

use crate::{
    states::Config,
    utils::{SureError, MIN_VOTING_LENGTH_SECONDS},
};

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(address = config.protocol_authority)]
    pub protocol_authority: Signer<'info>,

    pub config: Box<Account<'info, Config>>,

    pub system_program: Program<'info, System>,
}

pub fn update_voting_period(
    ctx: Context<UpdateConfig>,
    voting_period: i64,
    reveal_period: i64,
) -> Result<()> {
    if voting_period < MIN_VOTING_LENGTH_SECONDS || reveal_period < MIN_VOTING_LENGTH_SECONDS {
        return Err(SureError::InvalidVoteEndTime.into());
    }
    let old_voting_period = ctx.accounts.config.voting_length_seconds;
    let old_reveal_period = ctx.accounts.config.reveal_length_seconds;

    ctx.accounts
        .config
        .update_voting_lengths(voting_period, reveal_period)?;

    emit!(UpdatedVotingPeriod {
        old_voting_period,
        voting_period,
        old_reveal_period,
        reveal_period
    });
    Ok(())
}

#[event]
pub struct UpdatedVotingPeriod {
    old_voting_period: i64,
    voting_period: i64,
    old_reveal_period: i64,
    reveal_period: i64,
}

pub fn update_required_votes(ctx: Context<UpdateConfig>, required_votes: u64) -> Result<()> {
    if required_votes < 10_000 {
        return Err(SureError::InvalidRequiredVotesParam.into());
    }
    let old_required_votes = ctx.accounts.config.default_required_votes;
    ctx.accounts.config.update_required_votes(required_votes)?;

    emit!(UpdatedRequiredVotes {
        old_required_votes,
        required_votes
    });
    Ok(())
}

#[event]
pub struct UpdatedRequiredVotes {
    old_required_votes: u64,
    required_votes: u64,
}

pub fn update_proposal_minimum_stake(ctx: Context<UpdateConfig>, minimum_stake: u64) -> Result<()> {
    let old_minimum_stake = ctx.accounts.config.minimum_proposal_stake;
    ctx.accounts
        .config
        .update_proposal_minimum_stake(minimum_stake)?;
    emit!(UpdatedProposalMinumumStake {
        old_minimum_stake,
        minimum_stake
    });
    Ok(())
}

#[event]
pub struct UpdatedProposalMinumumStake {
    old_minimum_stake: u64,
    minimum_stake: u64,
}

pub fn update_vote_stake_rate(ctx: Context<UpdateConfig>, vote_stake_rate: u32) -> Result<()> {
    if vote_stake_rate <= 1 {
        return Err(SureError::InvalidVoteStakeRateParam.into());
    }
    let old_vote_stake_rate = ctx.accounts.config.vote_stake_rate;
    ctx.accounts
        .config
        .update_vote_stake_rate(vote_stake_rate)?;
    emit!(UpdatedVoteStakeRate {
        old_vote_stake_rate,
        vote_stake_rate
    });
    Ok(())
}

#[event]
pub struct UpdatedVoteStakeRate {
    old_vote_stake_rate: u32,
    vote_stake_rate: u32,
}

pub fn update_protocol_fee_rate(ctx: Context<UpdateConfig>, protocol_fee_rate: u32) -> Result<()> {
    if protocol_fee_rate <= 1 {
        return Err(SureError::InvalidProtocolFeeRateParam.into());
    }
    let old_protocol_fee_rate = ctx.accounts.config.protocol_fee_rate;
    ctx.accounts
        .config
        .update_protocol_fee_rate(protocol_fee_rate)?;
    emit!(UpdatedProtocolFeerate {
        old_protocol_fee_rate,
        protocol_fee_rate
    });
    Ok(())
}

#[event]
pub struct UpdatedProtocolFeerate {
    old_protocol_fee_rate: u32,
    protocol_fee_rate: u32,
}
