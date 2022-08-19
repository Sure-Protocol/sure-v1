use crate::seeds::*;
use crate::states::liquidity::*;
use crate::states::pool::*;
use crate::utils::SURE_NFT_UPDATE_AUTH;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, ID},
};
#[derive(Accounts)]
#[instruction(tick_upper: i32, tick_lower: i32)]
pub struct InitializeLiquidityPosition<'info> {
    #[account(mut)]
    liquidity_provider: Signer<'info>,

    pool: Box<Account<'info, Pool>>,

    #[account(
        init,
        payer = liquidity_provider,
        seeds = [
            SURE_DOMAIN.as_bytes(),
            position_mint.key().as_ref()
        ],
        space = 8 + LiquidityPosition::SPACE,
        bump,
    )]
    pub liquidity_position: Box<Account<'info, LiquidityPosition>>,

    /// Mint of NFT representing the
    /// liquidity position
    /// TODO: Unique seeds
    #[account(
        init,
        payer = liquidity_provider,
        seeds = [
            SURE_DOMAIN.as_bytes(),
            tick_lower.to_le_bytes().as_ref(),
            tick_upper.to_le_bytes().as_ref(),
            pool.key().as_ref(),
        ],
        bump,
        mint::authority = pool,
        mint::decimals = 0,
    )]
    pub position_mint: Account<'info, Mint>,

    /// Token account to hold the minted
    /// NFT
    #[account(
        init,
        payer = liquidity_provider,
        seeds = [
            SURE_TOKEN_ACCOUNT_SEED.as_bytes(),
            position_mint.key().as_ref(),
        ],
        bump,
        token::mint = position_mint,
        token::authority = liquidity_provider,
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

    /// associated token program
    /// used to create an account
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Token program to mint new NFT position
    #[account(address = ID)]
    pub token_program: Program<'info, Token>,

    pub rent: Sysvar<'info, Rent>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeLiquidityPosition>,
    tick_lower: i32,
    tick_upper: i32,
) -> Result<()> {
    Ok(())
}
