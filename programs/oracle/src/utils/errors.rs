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

    #[msg("Could not calculate the vote reward at this time")]
    NotPossibleToCalculateVoteReward,

    #[msg("Cannot payout the proposer reward at this time")]
    NotPossibleToCollectProposerReward,

    #[msg("Cannot payout the voter reward at this time")]
    NotPossibleToCollectVoterReward,

    #[msg("Cannot finalize user vote at this time")]
    FailedToFinalizeVote,

    #[msg("Cannot finalize vote result at this time")]
    FailedToFinalizeVoteResult,

    #[msg("Too late to cancel vote")]
    FailedToCancelVote,

    // account constraints
    #[msg("The owner of the vote account is not the signer")]
    InvalidOwnerOfVoteAccount,

    #[msg("Proposal.vault_mint does not match the input proposal_vault_mint key")]
    ProposalVaultMintKeyDoesNotMatchProposalStateVaultMint,

    #[msg("Proposal.vault_mint does not match the vault mint key")]
    ProposalVaultMintKeyDoesNotMatchVaultMint,

    #[msg("Not enough stake to propose a vote ")]
    NotEnoughProposalStake,

    #[msg("Quorum requirements are too low")]
    InvalidRequiredVotesParam,

    #[msg("Invalid minimum staked on proposal")]
    InvalidMinimumStakedParam,

    #[msg("Invalid vote stake rate param. Probably less than 1")]
    InvalidVoteStakeRateParam,

    #[msg("Invalid protocol fee rate param. Probably less than 1")]
    InvalidProtocolFeeRateParam,

    #[msg("Unauthorized signer")]
    UnauthorizedSigner,

    #[msg("No voting power")]
    NoVotingPower,

    #[msg("Failed to divide")]
    DivideOperationFailure,
}

impl From<TryFromIntError> for SureError {
    fn from(_: TryFromIntError) -> Self {
        SureError::StakeTooLittle
    }
}
