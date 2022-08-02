use crate::common::seeds::SURE_DOMAIN;
use crate::states::{FeePackage, Pool};
use anchor_lang::{prelude::*, solana_program::instruction};
use anchor_spl::{
    mint,
    token::{Mint, Token, TokenAccount},
};
use vipers::*;

/// Create Pool Vaults
/// creates the associated pool vault
/// based on token mint
#[derive(Accounts)]
#[instruction(product_id: u8,tick_spacing: u16)]
pub struct InitializePool<'info> {
    // Signer of the creation
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Sure Pool
    #[account(
        init,
        space = 8 + Pool::SPACE,
        payer = creator,
        seeds = [
            SURE_DOMAIN.as_bytes(),
            product_id.to_le_bytes().as_ref(),
            token_mint_a.key().as_ref(),
            token_mint_b.key().as_ref(),
            tick_spacing.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub pool: Box<Account<'info, Pool>>,

    /// Token Mint for Vault A
    /// This is the main pool
    #[account(constraint = token_mint_a.is_initialized)]
    pub token_mint_a: Account<'info, Mint>,
    /// Token Mint for Vault B
    #[account(constraint = token_mint_b.is_initialized)]
    pub token_mint_b: Account<'info, Mint>,

    // Pool Vault used to hold tokens from token_mint
    #[account(
        init,
        payer = creator,
        seeds = [ 
            SURE_DOMAIN.as_bytes(), 
            pool.key().as_ref(), 
            token_mint_a.key().as_ref()
        ],
        bump,
        token::mint = token_mint_a,
        token::authority = pool,
    )]
    pub pool_vault_a: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = creator,
        seeds = [ 
            SURE_DOMAIN.as_bytes(), 
            pool.key().as_ref(), 
            token_mint_b.key().as_ref()
        ],
        bump,
        token::mint = token_mint_b,
        token::authority = pool,
    )]
    pub pool_vault_b: Box<Account<'info, TokenAccount>>,
    /// Package specifies which fees should apply
    /// to the pool
    pub fee_package: Box<Account<'info, FeePackage>>,

    /// Sysvar for Associated Token Account
    pub rent: Sysvar<'info, Rent>,

    // Token program
    pub token_program: Program<'info, Token>,

    /// Provide the system program
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for InitializePool<'info> {
    fn validate(&self) -> Result<()> {
        // Make sure that token is USDC
        Ok(())
    }
}

/// Initialize Sure Marketplace
///
/// product_id:
///  - 1. Smart Contract Insurance
///
pub fn handler(
    ctx: Context<InitializePool>,
    product_id: u8,
    tick_spacing: u16,
    sqrt_price_x32: u64,
    name: String,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let fee_package = ctx.accounts.fee_package.as_ref();
    let token_mint_a = ctx.accounts.token_mint_a.key();
    let token_mint_b = ctx.accounts.token_mint_b.key();
    let pool_vault_a = ctx.accounts.pool_vault_a.key();
    let pool_vault_b = ctx.accounts.pool_vault_b.key();
    let founder = ctx.accounts.creator.key();
    // Initialize Token Pool
    let bump = *ctx.bumps.get("pool").unwrap();

    // Update pool with new tokenPool entry
    pool.initialize(
        bump,
        product_id,
        name,
        founder,
        tick_spacing,
        fee_package,
        sqrt_price_x32,
        token_mint_a,
        token_mint_b,
        pool_vault_a,
        pool_vault_b,
    )?;
    emit!(InitializePoolEvent {});
    Ok(())
}

#[event]
struct InitializePoolEvent {}