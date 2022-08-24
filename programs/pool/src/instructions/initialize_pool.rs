use crate::states::fee_package::FeePackage;
use crate::states::pool::*;
use crate::utils::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::{token::Mint, token::TokenAccount};
#[derive(Accounts)]
pub struct InitializePool<'info> {
    /// Pool creator
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        space = 8 + Pool::SPACE,
        payer = creator,
        seeds = [
            SURE_PRIMARY_POOL_SEED.as_bytes(),

            smart_contract.key().to_bytes().as_ref(),
        ],
        bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    pub fee_package: Account<'info, FeePackage>,

    // Assume the contract being hacked is a token account
    /// CHECK: This accounts represents the executable contract
    /// that is to be insured.
    #[account(
        constraint = smart_contract.executable == true
    )]
    pub smart_contract: UncheckedAccount<'info>,

    pub token_mint_0: Account<'info, Mint>,
    pub token_mint_1: Account<'info, Mint>,

    // Pool Vault used to hold tokens from token_mint
    #[account(
        init,
        payer = creator,
        seeds = [
            SURE_VAULT_POOL_SEED.as_bytes(),
            pool.key().as_ref(),
            token_mint_0.key().as_ref(),
        ],
        bump,
        token::mint = token_mint_0,
        token::authority = pool,
    )]
    pub vault_0: Box<Account<'info, TokenAccount>>,

    // Premium Vault holding all future premiums
    #[account(
        init,
        payer = creator,
        seeds = [
            SURE_PREMIUM_POOL_SEED.as_bytes(),
            pool.key().as_ref(),
            token_mint_1.key().as_ref()
        ],
        bump,
        token::mint = token_mint_1,
        token::authority = pool
    )]
    pub vault_1: Box<Account<'info, TokenAccount>>,

    // Token program used to create token accounts
    pub token_program: Program<'info, Token>,

    /// Sysvar for Associated Token Account
    pub rent: Sysvar<'info, Rent>,

    /// Provide the system program
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializePool>, name: String, tick_spacing: u16) -> Result<()> {
    // ________________ Validation ________________
    // Only allow the owner of the protocol to create pools
    // let protocol_owner = &ctx.accounts.protocol_owner.load()?;
    // require!(
    //     ctx.accounts.pool_creator.key() == protocol_owner.owner,
    //     SureError::InvalidPoolCreator
    // );

    // Load Accounts
    let pool = &mut ctx.accounts.pool;
    pool.initialize(
        *ctx.bumps.get("pool").unwrap(),
        name.clone(),
        ctx.accounts.creator.key(),
        tick_spacing,
        &ctx.accounts.fee_package,
        ctx.accounts.token_mint_0.key(),
        ctx.accounts.token_mint_1.key(),
        ctx.accounts.vault_0.key(),
        ctx.accounts.vault_1.key(),
    );
    emit!(CreatePool {
        name: name,
        smart_contract: ctx.accounts.smart_contract.key(),
    });

    Ok(())
}

#[event]
pub struct CreatePool {
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
}
