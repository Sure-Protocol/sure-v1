use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};

use crate::states::{Config, Proposal};
use crate::utils::{tokenTx, SureError, SURE_ORACLE_CONFIG_SEED};

#[derive(Accounts)]
pub struct CollectProtocolFees<'info> {
    #[account( address = config.protocol_authority @ SureError::UnauthorizedSigner )]
    pub protocol_authority: Signer<'info>,

    #[account(
        seeds = [
            SURE_ORACLE_CONFIG_SEED.as_bytes().as_ref(),
            config.token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        has_one = config
    )]
    pub proposal: Box<Account<'info, Proposal>>,

    #[account(
        mut,
        constraint = proposal_vault.key() == proposal.vault
    )]
    pub proposal_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = fee_destination.mint == config.token_mint,
        constraint = fee_destination.mint == proposal_vault.mint
    )]
    pub fee_destination: Account<'info, TokenAccount>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CollectProtocolFees>) -> Result<()> {
    let proposal = ctx.accounts.proposal.as_mut();

    let protocol_fees = proposal.payout_accrued_protocol_fees()?;

    // withdraw
    tokenTx::withdraw_from_vault(
        proposal,
        &ctx.accounts.proposal_vault,
        &ctx.accounts.fee_destination,
        &ctx.accounts.token_program,
        protocol_fees,
    )?;

    emit!(CollectedProtocolFees {
        proposal: proposal.key(),
        fees: protocol_fees,
        destination: ctx.accounts.fee_destination.key(),
    });
    Ok(())
}

#[event]
pub struct CollectedProtocolFees {
    proposal: Pubkey,
    fees: u64,
    destination: Pubkey,
}
