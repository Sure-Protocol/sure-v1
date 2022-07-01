use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use vipers::*;

/// --- Initialize Insurance Contract ---
///
/// Initializes an insurance contract for a specific tick
/// account.
///
/// side effects:
/// Pool contracts is updated:
///     - Info
///     - Bitmap overview
///
#[derive(Accounts)]
pub struct InitializePoolTickContract<'info> {
    /// Signer of contract
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Pool to buy insurance from
    pub pool: Box<Account<'info, Pool>>,

    /// Token mint used to insure with
    pub token_mint: Box<Account<'info, Mint>>,

    /// Tick account to insure against
    pub liquidity_tick_info: AccountLoader<'info, Tick>,

    /// Insurance Contract
    #[account(
        init,
        space = 8 + InsuranceTickContract::SPACE,
        payer = owner,
        seeds = [
            SURE_INSURANCE_CONTRACT.as_bytes(),
            owner.key().as_ref(),
            liquidity_tick_info.key().as_ref(),
        ],
        bump,
    )]
    pub insurance_tick_contract: Box<Account<'info, InsuranceTickContract>>,

    /// Insurance Contracts
    #[account(mut)]
    pub pool_insurance_contract_info: Box<Account<'info, PoolInsuranceContract>>,

    /// System Contract used to create accounts
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for InitializePoolTickContract<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

pub fn handler(ctx: Context<InitializePoolTickContract>) -> Result<()> {
    let insurance_tick_contract = &mut ctx.accounts.insurance_tick_contract;
    let pool_insurance_contract_bitmap = &mut ctx.accounts.pool_insurance_contract_bitmap;

    let liquidity_tick_info_state =
        AccountLoader::<tick::Tick>::try_from(&ctx.accounts.liquidity_tick_info.to_account_info())?;
    let liquidity_tick_info = liquidity_tick_info_state.load()?;

    // Method variables
    let current_time = Clock::get()?.unix_timestamp;
    // Initialize insurance_contract
    insurance_tick_contract.insured_amount = 0;
    insurance_tick_contract.premium = 0;
    insurance_tick_contract.bump = *ctx.bumps.get("insurance_tick_contract").unwrap();
    insurance_tick_contract.pool = ctx.accounts.pool.key();
    insurance_tick_contract.liquidity_tick_info = ctx.accounts.liquidity_tick_info.key();
    insurance_tick_contract.token_mint = ctx.accounts.token_mint.key();
    insurance_tick_contract.active = false;
    insurance_tick_contract.end_ts = current_time;
    insurance_tick_contract.created_ts = current_time;
    insurance_tick_contract.start_ts = current_time;

    // Update insurance contract
    // Mark the position as filled
    if !pool_insurance_contract_bitmap.is_initialized(liquidity_tick_info.tick) {
        pool_insurance_contract_bitmap.flip_bit(liquidity_tick_info.tick);
    }

    Ok(())
}
