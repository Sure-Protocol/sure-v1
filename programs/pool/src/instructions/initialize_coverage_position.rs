use crate::common::{
    access_control::SURE_NFT_UPDATE_AUTH, seeds::SURE_COVERAGE_DOMAIN,
    token_tx::create_coverage_position_with_metadata,
};
use crate::states::{CoveragePosition, Pool};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use std::mem::size_of;
/// Initialize coverage position
///
/// In order to buy coverage from a pool
/// a contract with the pool has to be created
///
/// In addition an nft that represents the position
/// is minted so that the user can exit the contract
/// on secondary markets
#[derive(Accounts)]
#[instruction(start_tick_index:i32)]
pub struct InitializeCoveragePosition<'info> {
    /// Coverage position Owner
    #[account(mut)]
    pub user: Signer<'info>,

    /// Pool to buy from
    pub pool: Box<Account<'info, Pool>>,

    /// New position
    #[account(
        init,
        payer = user,
        seeds = [
            SURE_COVERAGE_DOMAIN.as_bytes(),
            position_mint.key().as_ref(),
        ],
        space = 8 + size_of::<CoveragePosition>(),
        bump,
    )]
    pub coverage_position: AccountLoader<'info, CoveragePosition>,

    /// Position mint
    #[account(
        init,
        payer = user,
        seeds = [
            SURE_COVERAGE_DOMAIN.as_bytes(),
            pool.key().as_ref(),
            start_tick_index.to_le_bytes().as_ref(),
        ],
        bump,
        mint::authority = pool,
        mint::decimals = 0
    )]
    pub position_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = user,
        associated_token::mint = position_mint,
        associated_token::authority = user
    )]
    pub position_token_account: Account<'info, TokenAccount>,
    /// CHECK: Metaplex account is checked in the CPI
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: is checked in account contraints
    #[account(address = mpl_token_metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,

    /// CHECK: is checked in the account contraint
    /// only a given key can upgrade the metadata
    #[account(address = SURE_NFT_UPDATE_AUTH)]
    pub metadata_update_authority: UncheckedAccount<'info>,

    /// Token program to mint new NFT position

    /// associated token program
    /// used to create an account
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeCoveragePosition>, start_tick_index: i32) -> Result<()> {
    let position_owner = &ctx.accounts.user;
    let mut coverage_position = ctx.accounts.coverage_position.load_init()?;

    let coverage_position_bump = *ctx.bumps.get("coverage_position").unwrap();
    coverage_position.initialize(
        coverage_position_bump,
        position_owner,
        ctx.accounts.position_mint.key(),
        start_tick_index,
    )?;

    create_coverage_position_with_metadata(
        &ctx.accounts.metadata_account,
        &ctx.accounts.metadata_program,
        &ctx.accounts.metadata_update_authority,
        &ctx.accounts.pool,
        position_owner,
        &ctx.accounts.position_mint,
        &ctx.accounts.position_token_account,
        &ctx.accounts.token_program,
        &ctx.accounts.system_program,
        &ctx.accounts.rent,
    )?;

    emit!(InitializeCoveragePositionEvent {
        coverage_contract: ctx.accounts.coverage_position.key()
    });

    Ok(())
}

#[event]
pub struct InitializeCoveragePositionEvent {
    coverage_contract: Pubkey,
}
