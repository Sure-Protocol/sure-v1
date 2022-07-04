use crate::helpers::sToken::create_coverage_position_with_metadata;
use crate::states::{CoverageContract, Pool};
use crate::utils::seeds::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};

/// Initialize coverage position
///
/// In order to buy coverage from a pool
/// a contract with the pool has to be created
///
/// In addition an nft that represents the position
/// is minted so that the user can exit the contract
/// on secondary markets
#[derive(Accounts)]
pub struct InitializeCoveragePosition<'info> {
    /// Coverage position Owner
    #[account(mut)]
    pub user: Signer<'info>,

    /// Pool to buy from
    pub pool: Box<Account<'info, Pool>>,

    /// Position mint
    #[account(
        init,
        space = Mint::LEN,
        mint::authority = pool,
        mint::decimals = 0
    )]
    pub position_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        associated_token::mint = position_mint,
        associated_token::authority = user
    )]
    pub position_token_account: Account<'info, TokenAccount>,

    /// New position
    #[account(
        init,
        payer = user,
        seeds = [
            SURE_POOL,
            position_mint.key().as_ref(),
        ]
        space = 8 + CoveragePosition::SPACE,
        bump,
    )]
    pub coverage_position: Account<'info, CoveragePosition>,

    /// Token program to mint new NFT position
    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,

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

    /// associated token program
    /// used to create an account
    pub associated_token_program: Program<'info, AssociatedToken>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeCoveragePosition>) -> Result<()> {
    let position_owner = &ctx.accounts.user;
    let coverage_position = &ctx.accounts.coverage_position;

    coverage_position.initialize(position_owner, ctx.accounts.position_mint)?;

    create_coverage_position_with_metadata(
        &ctx.accounts.metadata_account,
        &ctx.accounts.metadata_program,
        &ctx.accounts.metadata_update_authority,
        &ctx.accounts.pool,
        position_owner,
        &ctx.accounts.position_mint,
        &ctx.accounts.token_program,
        &ctx.accounts.system_program,
        &ctx.accounts.rent,
    )?;

    emit!(
        InitializeCoveragePositionEvent{
            coverage_contract.key()
        }
    );

    Ok(())
}

#[event]
pub struct InitializeCoveragePositionEvent {
    coverage_contract: Pubkey,
}
