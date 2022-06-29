use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use vipers::*;

/// --- Initialize User Insurance Contract ---
///
/// Accounts used to initialize a user for a new pool
///
/// seeds: [
///     signer,
///     pool
///     token_mint
/// ]
#[derive(Accounts)]
pub struct InitializePoolContract<'info> {
    /// Signer
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Pool associated with the insurance contracts
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Token mint used for the insurance contracts
    pub token_mint: Box<Account<'info, Mint>>,

    /// Insurance contracts keeping an overview over
    /// Pools held by user
    #[account(mut)]
    pub insurance_contracts: Box<Account<'info, InsuranceContracts>>,

    /// Insurance Contracts Bitmap
    /// Bitmap for identifying for which ticks the user's
    /// insurance contract is located
    #[account(
        init,
        space = 8 + BitMap::SPACE,
        payer = signer,
        seeds = [
            SURE_INSURANCE_CONTRACTS_BITMAP.as_bytes(),
            signer.key().as_ref(),
            pool.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_insurance_contract_bitmap: Box<Account<'info, BitMap>>,
    /// Insurance pool contract info
    /// Holds aggregate information on all the
    /// insurance contracts for a given user
    #[account(
        init,
        space = 8 + PoolInsuranceContract::SPACE,
        payer = signer,
        seeds = [
            SURE_INSURANCE_CONTRACTS_INFO.as_bytes(),
            signer.key().as_ref(),
            pool.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_insurance_contract_info: Box<Account<'info, PoolInsuranceContract>>,

    /// System program
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializePoolContract>) -> Result<()> {
    let pool_insurance_contract_bitmap = &mut ctx.accounts.pool_insurance_contract_bitmap;
    let pool_insurance_contract_info = &mut ctx.accounts.pool_insurance_contract_info;
    let pool = &mut ctx.accounts.pool;
    let insurance_contracts = &mut ctx.accounts.insurance_contracts;

    let current_time = Clock::get()?.unix_timestamp;
    // Initialize the insurance contract overview
    pool_insurance_contract_info.bump = unwrap_bump!(ctx, "pool_insurance_contract_info");
    pool_insurance_contract_info.expiry_ts = current_time;
    pool_insurance_contract_info.insured_amount = 0;
    pool_insurance_contract_info.owner = ctx.accounts.signer.key();

    pool_insurance_contract_bitmap.bump = unwrap_bump!(ctx, "pool_insurance_contract_bitmap");
    pool_insurance_contract_bitmap.spacing = 10;
    pool_insurance_contract_bitmap.word = [0; 4];

    // update Insurance contracts
    insurance_contracts.pools.push(pool.key().clone());

    Ok(())
}
