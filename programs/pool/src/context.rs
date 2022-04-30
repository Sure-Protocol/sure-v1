use anchor_lang::prelude::*;
use crate::states::pool::PoolManager;
use crate::states::pool::PoolAccount;
use anchor_spl::token::{Mint,TokenAccount,Token};

pub const SURE_PRIMARY_POOL_SEED: &str = "sure-insurance-pool";
pub const SURE_ASSOCIATED_TOKEN_ACCOUNT_SEED: &str = "sure-ata";

#[derive(Accounts)]
pub struct InitializePoolManager<'info> {
    // Account for keeping track of the pool manager
    #[account(init,
        payer= initial_manager,
        space = 8 + PoolManager::POOL_MANAGER_SIZE,
        seeds = [b"sure-pool-manager"],
        bump 
    )]
    pub manager: Account<'info, PoolManager>,

    // The signer becomes the initial manager
    #[account(mut)]
    pub initial_manager: Signer<'info>,

    // System program
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(smart_contract: Pubkey)]
pub struct CreatePool<'info> {
    #[account(
        init,
        space = 8 + PoolAccount::SPACE, 
        payer = pool_creator,
        seeds = [
            SURE_PRIMARY_POOL_SEED.as_bytes(),
            token.key().as_ref(),
            insured_token_account.key().to_bytes().as_ref(),
        ],
        bump
    )]
    pub pool: Account<'info, PoolAccount>,

    // Assume the contract being hacked is a token account
    /// CHECK: This accounts represents the executable contract 
    /// that is to be insured. 
    #[account(
        constraint = insured_token_account.executable == false
    )]
    pub insured_token_account: AccountInfo<'info>,

    // Initialized vault to hold the pool token
    #[account(
        init,
        payer = pool_creator,
        seeds = [
            SURE_ASSOCIATED_TOKEN_ACCOUNT_SEED.as_bytes(), 
            pool.key().as_ref(),
            token.key().as_ref()
        ],
        bump,
        token::mint = token,
        token::authority = vault
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    /// Pool creator
    #[account(mut)]
    pub pool_creator: Signer<'info>,

    /// Token to be deposited into the pool
    pub token: Account<'info,Mint>,

    /// Sysvar for Associated Token Account
    pub rent: Sysvar<'info, Rent>,

    // Token program
    pub token_program: Program<'info, Token>,

    /// Provide the system program
    pub system_program: Program<'info,System>,

}

/// Deposit Liquidity into an exisitng pool
#[derive(Accounts)]
pub struct DepositLiquidity<'info>{
    /// Liquidity provider 
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    /// Pool to provide liquidity to 
    #[account(mut)]
    pub pool:  Account<'info, PoolAccount>,

    // Token program that executes the transfer
    pub token_account: Program<'info,Token>, 


}