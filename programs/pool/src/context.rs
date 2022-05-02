

use std::thread::AccessError;

use anchor_lang::prelude::*;
use crate::states::{pool::{PoolManager,PoolAccount,LiquidityPosition,Tick}, owner::ProtocolOwner, contract::InsuranceContract};
use anchor_spl::{token::{Mint,TokenAccount,Token}, associated_token::AssociatedToken};

pub const SURE_PRIMARY_POOL_SEED: &str = "sure-insurance-pool";
pub const SURE_ASSOCIATED_TOKEN_ACCOUNT_SEED: &str = "sure-ata";
pub const SURE_LIQUIDITY_POSITION: &str = "sure-lp";
pub const SURE_PROTOCOL_OWNER: &str = "sure-protocol-owner";
pub const SURE_INSURANCE_CONTRACT: &str ="sure-insurance-contract";


/// Initialize Sure Protocol 
/// by setting the owner of the protocol 
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Owner of the protocol
    #[account(mut)]
    pub owner: Signer<'info>,

    /// 
    #[account(
        init,
        seeds=[],
        bump,
        payer = owner, 
        space = 8 + ProtocolOwner::SPACE,
    )]
    pub protocol_owner: AccountLoader<'info,ProtocolOwner>,

    /// System Program to create a new account
    pub system_program: Program<'info,System>,

}

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
    pub insured_token_account: UncheckedAccount<'info>,

    // Initialized Associated token accoun to hold vault tokens
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
#[instruction(tick: u32, bump: u8)]
pub struct DepositLiquidity<'info>{
    /// Liquidity provider 
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    /// Protocol owner as the authority of mints
    pub protocol_owner: AccountLoader<'info,ProtocolOwner>,

    /// Account to credit
    #[account(mut)]
    pub liquidity_provider_account: Box<Account<'info,TokenAccount>>,

    /// Pool to provide liquidity to 
    #[account(mut)]
    pub pool:  Account<'info, PoolAccount>,

    /// Pool Vault account to deposit liquidity to
    #[account(mut)]
    pub token_vault: Account<'info,PoolAccount>,

    
    /// Create Liquidity position
    /// HASH: [sure-lp,liquidity-provider,pool,token,tick]
    #[account(
        init,
        payer = liquidity_provider,
        seeds = [
            SURE_LIQUIDITY_POSITION.as_bytes(),
            pool.key().as_ref(),
            token_vault.key().as_ref(),
            tick.to_le_bytes().as_ref(),
            nft_mint.key().as_ref(), // NFT points to liquidity position
        ],
        space = 8 + LiquidityPosition::SPACE,
        bump,
    )]
    pub liquidity_position: Account<'info,LiquidityPosition>,

    // NFT minting
    #[account(
        init,
        mint::decimals = 0,
        mint::authority = protocol_owner,
        payer = liquidity_provider,
    )]
    pub nft_mint: Box<Account<'info,Mint>>,

    /// Account to deposit NFT into
    #[account(
        init,
        associated_token::mint = nft_mint,
        associated_token::authority = liquidity_provider,
        payer = liquidity_provider,
    )]
    pub nft_account: Box<Account<'info,TokenAccount>>,


     /// Sysvar for token mint and ATA creation
     pub rent: Sysvar<'info, Rent>,

    // Token program that executes the transfer
    pub token_program: Program<'info,Token>,

    /// Provide the system program
    pub system_program: Program<'info,System>,

    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
}



/// Redeem liquidity 
/// 
#[derive(Accounts)]
pub struct RedeemLiquidity<'info> {
    /// Holder of the LP NFT
    pub nft_holder: Signer<'info>,

    /// NFT that proves ownership of position
    #[account(
        constraint = nft.mint ==liquidity_position.nft_mint
    )] 
    pub nft: Box<Account<'info,TokenAccount>>,

    /// Mint of the NFT 
    #[account(mut)]
    pub nft_mint: Account<'info,Mint>,

    /// Liquidity position
    #[account(mut)]
    pub liquidity_position: Account<'info,LiquidityPosition>,

    /// Token account to recieve the tokens at
    pub token_account: Box<Account<'info,TokenAccount>>,

    /// Pool Vault to transfer tokens from
    pub vault_account: Box<Account<'info,TokenAccount>>,

    /// Sure Protocol Pool Account 
    #[account(mut)]
    pub pool: Account<'info, PoolAccount>,

    /// Sure owner
    pub protocol_owner: AccountLoader<'info,ProtocolOwner>,

}


#[derive(Accounts)]
pub struct InitializeTick<'info>{
    /// Create tick
    pub tick: Account<'info,Tick>,
}

/// Buy Insurance Request
/// 
#[derive(Accounts)]
pub struct BuyInsurance<'info> {
    /// Buyer 
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// Pool to buy from
    pub pool: Account<'info,PoolAccount>,

    /// Insurance Position
    #[account(
        init,
        space = 8 + InsuranceContract::SPACE,
        payer = buyer,
        seeds = [
            SURE_INSURANCE_CONTRACT.as_bytes(),
            pool.key().as_ref(),
        ],
        bump,
    )]
    pub insurance_contract: Account<'info,InsuranceContract>,

    /// System Contract used to create accounts
    pub system_program: Program<'info,System>,
}