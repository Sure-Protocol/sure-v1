use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};
use vipers::*;
/// --- Update Insurance Tick Contract ---
///
/// Updates the insurance contract for the given tick account
///
/// Method adjust the position and expiry and calculates the
/// new premium that has to be refunded or paid
#[derive(Accounts)]
pub struct UpdatePoolTickContract<'info> {
    /// Buyer
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// Account to buy tokens with
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Pool owning the token pool to buy from
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Pool Token Account
    /// Keeps an overview over used liquidity
    #[account(mut)]
    pub token_pool: Box<Account<'info, TokenPool>>,

    /// Tick account to buy from
    #[account(mut)]
    pub liquidity_tick_info: AccountLoader<'info, Tick>,

    /// Liquidity Tick Bitmap
    ///
    /// Holds information on which ticks that contains
    /// available liquidity
    #[account(mut)]
    pub liquidity_tick_bitmap: Box<Account<'info, BitMap>>,

    /// Premium Vault
    #[account(
        mut,
        constraint = premium_vault.owner ==  pool.key(),
        constraint = premium_vault.mint == token_account.mint,
    )]
    pub premium_vault: Box<Account<'info, TokenAccount>>,

    /// Insurance Contract
    #[account(mut,
    constraint = insurance_tick_contract.pool == pool.key(),
    )]
    pub insurance_tick_contract: Box<Account<'info, InsuranceTickContract>>,

    /// Insurance Contracts
    #[account(mut)]
    pub pool_insurance_contract_info: Box<Account<'info, PoolInsuranceContract>>,

    /// Token program, needed to transfer tokens
    pub token_program: Program<'info, Token>,

    /// System Contract used to create accounts
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for UpdatePoolTickContract<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

pub fn handler(
    ctx: Context<UpdatePoolTickContract>,
    new_insured_amount_on_tick: u64,
    new_expiry_ts: i64,
) -> Result<()> {
    // Load accounts
    let liquidity_tick_info_state =
        AccountLoader::<tick::Tick>::try_from(&ctx.accounts.liquidity_tick_info.to_account_info())?;
    let mut liquidity_tick_info = liquidity_tick_info_state.load_mut()?;
    let pool = &mut ctx.accounts.pool;
    let token_pool = &mut ctx.accounts.token_pool;
    let insurance_tick_contract = &mut ctx.accounts.insurance_tick_contract;
    let pool_insurance_contract_info = &mut ctx.accounts.pool_insurance_contract_info;
    let liquidity_tick_bitmap = &mut ctx.accounts.liquidity_tick_bitmap;

    // Calculate coverage amount
    let current_insured_amount_on_tick = insurance_tick_contract.insured_amount;
    let amount_diff = if new_insured_amount_on_tick > current_insured_amount_on_tick {
        new_insured_amount_on_tick - current_insured_amount_on_tick
    } else {
        current_insured_amount_on_tick - new_insured_amount_on_tick
    };

    // Calculate the premium that has to be refunded or paid
    let (increase_premium, premium) = insurance_tick_contract.update_position_and_get_premium(
        liquidity_tick_info.tick,
        new_insured_amount_on_tick,
        new_expiry_ts,
    )?;

    if new_insured_amount_on_tick > current_insured_amount_on_tick {
        liquidity_tick_info
            .buy_insurance(amount_diff)
            .map_err(|e| e.to_anchor_error())?;

        if liquidity_tick_info.is_pool_full() {
            liquidity_tick_bitmap.flip_bit(liquidity_tick_info.tick);
        }
        // Increase total insured amount for insurance pool contract
        pool_insurance_contract_info.insured_amount += amount_diff;
        token_pool.used_liquidity += amount_diff;
        pool_insurance_contract_info.expiry_ts = new_expiry_ts
    } else {
        liquidity_tick_info
            .exit_insurance(amount_diff)
            .map_err(|e| e.to_anchor_error())?;

        if !liquidity_tick_info.is_pool_full() {
            if !liquidity_tick_bitmap.is_initialized(liquidity_tick_info.tick) {
                liquidity_tick_bitmap.flip_bit(liquidity_tick_info.tick);
            }
        }

        // Reduce total insured amount for pool
        pool_insurance_contract_info.insured_amount -= amount_diff;
        token_pool.used_liquidity -= amount_diff;
        pool_insurance_contract_info.expiry_ts = new_expiry_ts
    }
    if increase_premium {
        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                Transfer {
                    from: ctx.accounts.token_account.to_account_info().clone(),
                    to: ctx.accounts.premium_vault.to_account_info().clone(),
                    authority: ctx.accounts.buyer.to_account_info().clone(),
                },
            ),
            premium,
        )?;
    } else {
        let pool_seed = [
            &SURE_PRIMARY_POOL_SEED.as_bytes() as &[u8],
            &pool.smart_contract.to_bytes() as &[u8],
            &[pool.bump],
        ];
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone(),
                Transfer {
                    from: ctx.accounts.premium_vault.to_account_info().clone(),
                    to: ctx.accounts.token_account.to_account_info().clone(),
                    authority: ctx.accounts.pool.to_account_info().clone(),
                },
                &[&pool_seed[..]],
            ),
            premium,
        )?;
    }
    Ok(())
}
