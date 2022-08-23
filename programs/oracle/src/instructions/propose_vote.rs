use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{utils::{SureError, SURE, SURE_ORACLE_SEED}, states::ProposeVoteEvent};
use crate::{states::{proposal::Proposal,RevealedVoteArray}, utils::token};

pub const MINIMUM_STAKE: u64 = 3_000_000;
// 1/ln(2)
// Q16.16
pub const DIV_LN2_X64: u64 = 94548;

/// Validate that the stake is large enough
///
pub fn validate_stake(stake: u64) -> Result<()> {
    if stake < MINIMUM_STAKE {
        return Err(SureError::StakeTooLittle.into());
    }

    Ok(())
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
        init,
        payer = proposer,
        seeds = [
            SURE_ORACLE_SEED.as_bytes().as_ref(),
            proposal.key().as_ref(),
        ],
        bump,
        space = 8 + RevealedVoteArray::SPACE
    )]
    pub reveal_vote_array: AccountLoader<'info,RevealedVoteArray>,

    #[account(
        mut, 
        constraint = proposer_account.mint == proposal_vault_mint.key()
    )]
    pub proposer_account: Box<Account<'info,TokenAccount>>,

    #[account(
        constraint = proposal_vault_mint.key() == SURE
    )]
    pub proposal_vault_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = proposer,
        associated_token::mint = proposal_vault_mint,
        associated_token::authority = proposal,
    )]
    pub proposal_vault: Box<Account<'info, TokenAccount>>,

    /// Token minted to represent rewards
    #[account(
        init,
        payer = proposer,
        seeds = [
            SURE_ORACLE_SEED.as_bytes(),
            name.as_bytes().as_ref(),
        ],
        bump,
        mint::authority = proposal,
        mint::decimals = 6,
    )]
    pub token_reward_mint: Box<Account<'info, Mint>>,


    //
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

/// Propose vote 
/// 
/// proposes a vote or observation that the holder of veSure can 
/// vote on 
/// 
/// # Arguments
/// * ctx: Context
/// * name: Name of the observation
/// * description: Clear description about the event
/// * stake: The amount staked on event. In BN:  x*10^{decimals}
pub fn handler(
    ctx: Context<ProposeVote>,
    name: String,
    description: String,
    stake: u64, //Q64.0
) -> Result<()> {
    let proposal = ctx.accounts.proposal.as_mut();
    let proposal_bump = *ctx.bumps.get("proposal").unwrap();
    let reveal_vote_array_bump = *ctx.bumps.get("reveal_vote_array").unwrap();
    let decimals = ctx.accounts.proposal_vault_mint.decimals;
    let mut reveal_vote_array = ctx.accounts.reveal_vote_array.load_mut()?;
    let token_supply = ctx.accounts.proposal_vault_mint.supply;

    // Initialize state 
    proposal.initialize(
        proposal_bump,
        name.clone(),
        description,
        &ctx.accounts.proposer.key(),
        stake,
        &ctx.accounts.token_reward_mint.key(),
        token_supply,
        &ctx.accounts.proposal_vault.key(),
        None,
        decimals,
    )?;

    // initialize reveal_vote_array
    reveal_vote_array.initialize(proposal.key(), reveal_vote_array_bump);

    // deposit stake into vault 
    token::deposit_into_vault(&ctx.accounts.proposer, &ctx.accounts.proposal_vault, &ctx.accounts.proposer_account, &ctx.accounts.token_program, stake)?;

    emit!(ProposeVoteEvent{
        name: name,
        proposer: ctx.accounts.proposer.key()
    });
   
    Ok(())
}
