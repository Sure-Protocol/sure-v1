use std::future::ready;

use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    states::{Config, Proposal, VoteAccount},
    utils::{tokenTx, SureError},
};

#[derive(Accounts)]
pub struct CollectVoteReward<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        constraint = voter_account.mint == proposal_vault.mint,
        constraint = voter_account.mint == proposal_vault_mint.key(),
        constraint = voter_account.owner ==  voter.key()
    )]
    pub voter_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub vote_account: AccountLoader<'info, VoteAccount>,

    #[account(
        mut,
        has_one = config
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        constraint = proposal_vault_mint.key() == proposal_vault.mint @ SureError::ProposalVaultMintKeyDoesNotMatchVaultMint,
        constraint = proposal_vault_mint.key() == config.token_mint @ SureError::ProposalVaultMintKeyDoesNotMatchProposalStateVaultMint,

    )]
    pub proposal_vault_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = proposal_vault.owner == proposal.key()
    )]
    pub proposal_vault: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CollectVoteReward>) -> Result<()> {
    let mut vote_account = ctx.accounts.vote_account.load_mut()?;
    let proposal = ctx.accounts.proposal.as_ref();
    let time = clock::Clock::get()?.unix_timestamp;
    let mint_decimals = ctx.accounts.proposal_vault_mint.decimals;

    // check if it is possible to collect rewards
    proposal.can_collect_voter_reward(time)?;

    // get the user vote reward
    let reward = vote_account.calculate_token_reward_at_time(proposal, mint_decimals, time)?;

    tokenTx::withdraw_from_vault(
        proposal,
        &ctx.accounts.proposal_vault,
        &ctx.accounts.voter_account,
        &ctx.accounts.token_program,
        reward,
    )?;

    emit!(CollectVoteRewardEvent {
        vote: ctx.accounts.vote_account.key(),
        proposal: proposal.key(),
        time: time,
        reward: reward,
    });
    Ok(())
}

#[event]
pub struct CollectVoteRewardEvent {
    pub vote: Pubkey,
    pub proposal: Pubkey,
    pub time: i64,
    pub reward: u64,
}
