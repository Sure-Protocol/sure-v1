use anchor_lang::prelude::*;
pub mod factory;
pub mod instructions;
pub mod states;
pub mod utils;

use crate::instructions::ProposeVote;
use instructions::*;

declare_id!("2prR7H6LfRqwiP2iTyZG1suG4B3zU6JEpUBXWeQB66qH");
#[program]
pub mod oracle {
    use super::*;

    /// initialize config
    ///
    /// config will be used in proposals to set voting parameters
    /// and limit protocol fee collectors to the protocol_authority
    ///
    /// ### args
    /// * protocol_authority<Pubkey>: unique for vault mint. the authority can
    ///     - change config parameters
    ///     - collect protocol fees
    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        protocol_authority: Pubkey,
    ) -> Result<()> {
        instructions::initialize_config::handler(ctx, protocol_authority)
    }

    /// update config: voting period
    ///
    /// change the voting period and reveal period
    ///
    /// ### args
    /// * voting_period<i64>: period for which the voter can submit a vote hash. In seconds
    /// * reveal_period<i64>: period for which the voter can reveal the vote. In seconds
    pub fn update_voting_period(ctx: Context<UpdateConfig>, voting_period: i64) -> Result<()> {
        instructions::update_voting_period(ctx, voting_period)
    }

    /// update config: reveal period
    ///
    /// change the reveal period and reveal period
    ///
    /// ### args
    /// * voting_period<i64>: period for which the voter can submit a vote hash. In seconds
    /// * reveal_period<i64>: period for which the voter can reveal the vote. In seconds
    pub fn update_reveal_period(ctx: Context<UpdateConfig>, voting_period: i64) -> Result<()> {
        instructions::update_reveal_period(ctx, voting_period)
    }

    /// update required votes
    ///
    /// required votes to reach quorum
    ///
    /// ### args
    /// * require_votes<u64>: number of votes needed to conclude vote
    pub fn update_required_votes(ctx: Context<UpdateConfig>, required_votes: u64) -> Result<()> {
        instructions::update_required_votes(ctx, required_votes)
    }

    /// update proposal minimum stake
    ///
    /// the minimum amount that needs to be staked in order to create a
    /// proposal
    ///
    /// ### args
    /// * minimum_stake<u64>: stake needed to propose vote
    pub fn update_proposal_minimum_stake(
        ctx: Context<UpdateConfig>,
        minimum_stake: u64,
    ) -> Result<()> {
        instructions::update_proposal_minimum_stake(ctx, minimum_stake)
    }

    /// update vote stake rate
    ///
    /// the stake rate sets requirements to how much a
    /// voter needs to stake in order to vote. typically 1% of voting
    /// power
    ///
    /// ### args
    /// * vote_stake_rate<u32>: 1/x of voting power
    pub fn update_vote_stake_rate(ctx: Context<UpdateConfig>, vote_stake_rate: u32) -> Result<()> {
        instructions::update_vote_stake_rate(ctx, vote_stake_rate)
    }

    /// update protocol fee rate
    ///
    /// the amount the protocol can take in fees
    ///
    /// ### args
    /// * protocol_fee_rate<u32>: 1/x of the voting pool
    pub fn update_protocol_fee_rate(
        ctx: Context<UpdateConfig>,
        protocol_fee_rate: u32,
    ) -> Result<()> {
        instructions::update_protocol_fee_rate(ctx, protocol_fee_rate)
    }

    /// Propose vote
    ///
    /// proposes a vote or observation that the holder of veSure can
    /// vote on.
    ///
    /// ### paramters
    /// * `ctx`: Context
    /// * `name`: Name of the observation
    /// * `description`: Clear description about the event
    /// * `stake`: The amount staked on event. In BN:  x*10^{decimals}
    pub fn propose_vote(
        ctx: Context<ProposeVote>,
        id: Vec<u8>,
        name: String,
        description: String,
        stake: u64,
    ) -> Result<()> {
        instructions::propose_vote::handler(ctx, id, name, description, stake)
    }

    /// Submit vote
    ///
    /// lets user vote blindly on a proposal using a vote hash
    ///
    /// ### Parameters
    /// * `ctx` - context
    /// * `vote_hash` - hash of vote with secret salt
    pub fn submit_vote(ctx: Context<SubmitVote>, vote_hash: Vec<u8>) -> Result<()> {
        instructions::submit_vote::handler(ctx, vote_hash)
    }

    /// Updates vote
    ///
    /// updates the vote hash of the previous submitted vote
    ///
    /// ### parameters
    /// * `ctx` - context
    /// * `vote_hash` - hash of vote with secret salt
    pub fn update_vote(ctx: Context<UpdateVote>, vote_hash: Vec<u8>) -> Result<()> {
        instructions::update_vote::handler(ctx, vote_hash)
    }

    /// cancel vote
    ///
    /// a user can cancel the vote in the voting period
    ///
    /// ### parameters
    /// * `ctx` - CancelVote context
    pub fn cancel_vote(ctx: Context<CancelVote>) -> Result<()> {
        instructions::cancel_vote::handler(ctx)
    }

    /// reveal vote
    ///
    /// let the user reveal the vote when the voting period is over
    /// the user can only receive rewards if revealed
    ///
    /// ### parameters
    /// * `ctx` - RevealVote context
    /// * `salt` - the salt used to hash the vote
    /// * `vote`- the actual vote value
    pub fn reveal_vote(ctx: Context<RevealVote>, salt: String, vote: i64) -> Result<()> {
        instructions::reveal_vote::handler(ctx, salt, vote)
    }

    /// finalize vote results
    ///
    /// after the reveal period the proposal can be finalized
    /// from this point on it is not possible to reveal the vote
    ///
    /// the proposer reward and scale parameter is calculated
    ///
    /// ### parameters
    /// *  `ctx` - the Finalize Vote context
    pub fn finalize_vote_results(ctx: Context<FinalizeVoteResults>) -> Result<()> {
        instructions::finalize_vote_results::handler(ctx)
    }

    /// finalize vote
    ///
    /// after the vote results are finalized the voters can calculate
    /// their vote share and close their vote account
    ///
    /// ### parameters
    /// * `ctx` - Finalize Vote context
    pub fn finalize_vote(ctx: Context<FinalizeVote>) -> Result<()> {
        instructions::finalize_vote::handler(ctx)
    }

    /// collect proposer reward
    ///
    /// after the vote results are finalized the proposer is free to
    /// collect the reward at any time
    pub fn collect_proposer_reward(ctx: Context<CollectProposerReward>) -> Result<()> {
        instructions::collect_proposer_reward::handler(ctx)
    }

    /// collect vote reward
    ///
    /// after the vote results are finalized the voter can collect rewards
    pub fn collect_vote_reward(ctx: Context<CollectVoteReward>) -> Result<()> {
        instructions::collect_vote_reward::handler(ctx)
    }

    /// collect protocol fees
    ///
    /// the config authority can at any time collect the protocol fees
    pub fn collect_protocol_fees(ctx: Context<CollectProtocolFees>) -> Result<()> {
        instructions::collect_protocol_fees::handler(ctx)
    }
}
