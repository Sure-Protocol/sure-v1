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

    #[msg("Invalid salt resulted in invalid vote_hash")]
    InvalidSalt,
}

impl From<TryFromIntError> for SureError {
    fn from(_: TryFromIntError) -> Self {
        SureError::StakeTooLittle
    }
}