use anchor_lang::{prelude::*, solana_program::clock};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount},
};
use std::{io::Read, mem::size_of};

use crate::states::{proposal::Proposal, Config, RevealedVoteArray};
use crate::utils::tokenTx;
use crate::utils::{SureError, SURE_ORACLE_PROPOSAL_VAULT_SEED, SURE_ORACLE_REVEAL_ARRAY_SEED, SURE_ORACLE_SEED};
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
#[instruction(id: Vec<u8>)]
pub struct ProposeVote<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    /// configuration for the proposal
    pub config: Box<Account<'info, Config>>,

    #[account(
        init,
        payer = proposer,
        seeds = [
            SURE_ORACLE_SEED.as_bytes().as_ref(),
            id.as_ref(), 
        ],
        bump,
        space = 8 + Proposal::SPACE
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        init,
        payer = proposer,
        seeds = [
            SURE_ORACLE_REVEAL_ARRAY_SEED.as_bytes().as_ref(),
            id.as_ref(), 
        ],
        bump,
        space = 8 + size_of::<RevealedVoteArray>()
    )]
    pub reveal_vote_array: AccountLoader<'info, RevealedVoteArray>,

    #[account(
        mut,
        constraint = proposer_account.mint == proposal_vault_mint.key()
    )]
    pub proposer_account: Box<Account<'info, TokenAccount>>,

    #[account(
        constraint = proposal_vault_mint.key() == config.token_mint
    )]
    pub proposal_vault_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = proposer,
        seeds = [
            SURE_ORACLE_PROPOSAL_VAULT_SEED.as_bytes().as_ref(),
            id.as_ref(), 
        ],
        bump,
        token::mint = proposal_vault_mint,
        token::authority = proposal,
    )]
    pub proposal_vault: Box<Account<'info, TokenAccount>>,

    #[account(address = token::ID)]
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
    id: Vec<u8>,
    name: String,
    description: String,
    stake: u64, //Q64.0
) -> Result<()> {
    let proposal = ctx.accounts.proposal.as_mut();
    let proposal_bump = *ctx.bumps.get("proposal").unwrap();
    let config = ctx.accounts.config.as_ref();
    let reveal_vote_array_bump = *ctx.bumps.get("reveal_vote_array").unwrap();
    let mut reveal_vote_array = ctx.accounts.reveal_vote_array.load_init()?;
    let time = clock::Clock::get()?.unix_timestamp;

    // TODO: might fail if not used hasher 
    let id_hash: [u8; 32] = id.clone().try_into().unwrap();

    // Initialize state
    proposal.initialize(
        config,
        proposal_bump,
        name.clone(),
        &id_hash,
        &description,
        &ctx.accounts.proposer.key(),
        stake,
        &ctx.accounts.proposal_vault.key(),
        None,
    )?;

    // initialize reveal_vote_array
    reveal_vote_array.initialize(proposal.key(), reveal_vote_array_bump);

    // cb: update status of proposal
    proposal.update_status(time);

    // deposit stake into vault
    tokenTx::deposit_into_vault(
        &ctx.accounts.proposer,
        &ctx.accounts.proposal_vault,
        &ctx.accounts.proposer_account,
        &ctx.accounts.token_program,
        stake,
    )?;

    emit!(ProposeVoteEvent {
        name: name,
        description: description,
        id: id.clone().to_vec(),
        proposer: ctx.accounts.proposer.key(),
        stake: stake,
    }); 

    Ok(())
}

#[event]
pub struct ProposeVoteEvent {
    pub name: String,
    pub description: String,
    pub id: Vec<u8>,
    pub proposer: Pubkey,
    pub stake: u64,
}
