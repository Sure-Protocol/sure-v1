use anchor_lang::prelude::*;
use std::num::TryFromIntError;

#[error_code]
#[derive(PartialEq)]
pub enum SureError {
    #[msg("Not enough staked on vote")]
    StakeTooLittle,

    #[msg("Invalid lock period")]
    InvalidLockPeriod,

    #[msg("Invalid vote end time")]
    InvalidVoteEndTime,

    #[msg("Voting period for proposal has ended")]
    VotingPeriodEnded,

    #[msg("Currently not in vote reveal period")]
    RevealPeriodNotActive,

    #[msg("Reveal period is not over")]
    RevealPeriodIsNotFinished,

    #[msg("Invalid salt resulted in invalid vote_hash")]
    InvalidSalt,

    #[msg("Revealed vote list full")]
    FullRevealList,

    #[msg("Vote hasn't been revealed")]
    VoteNotRevealed,

    #[msg("U64 overflow")]
    OverflowU64,

    #[msg("U32 overflow")]
    OverflowU32,

    #[msg("Could not calculate the vote reward at this point")]
    NotPossibleToCalculateVoteReward,

    #[msg("Cannot payout the proposer reward at this time")]
    NotPossibleToPayoutProposerReward,

    #[msg("Too late to cancel vote")]
    FailedToCancelVote,
}

impl From<TryFromIntError> for SureError {
    fn from(_: TryFromIntError) -> Self {
        SureError::StakeTooLittle
    }
}
