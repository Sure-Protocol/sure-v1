use anchor_lang::prelude::*;
pub mod factory;
pub mod instructions;
pub mod states;
pub mod utils;

use crate::instructions::ProposeVote;
use instructions::*;

declare_id!("G3HjAD81oEXbR867NNBfpZ2PWDhsioaCguPZhTiXunu");
#[program]
pub mod oracle {
    use super::*;

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
        name: String,
        description: String,
        stake: u64,
    ) -> Result<()> {
        instructions::propose_vote::handler(ctx, name, description, stake)
    }

    /// Submit vote
    ///
    /// lets user vote blindly on a proposal using a vote hash
    ///
    /// ### Parameters
    /// * `ctx` - context
    /// * `vote_hash` - hash of vote with secret salt
    pub fn submit_vote(ctx: Context<SubmitVote>, vote_hash: String) -> Result<()> {
        instructions::submit_vote::handler(ctx, vote_hash)
    }

    /// Updates vote
    ///
    /// updates the vote hash of the previous submitted vote
    ///
    /// ### parameters
    /// * `ctx` - context
    /// * `vote_hash` - hash of vote with secret salt
    pub fn update_vote(ctx: Context<UpdateVote>, vote_hash: String) -> Result<()> {
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
}
