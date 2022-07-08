use crate::common::account::validate_token_account_ownership;
use crate::common::token_tx::withdraw_from_vault;
use crate::common::{
    account,
    errors::SureError,
    liquidity::{calculate_token_0_delta, calculate_token_1_delta, validate_liquidity_amount},
};
use crate::factory::liquidity::*;
use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::token;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};
use vipers::*;
/// Redeem liquidity
/// Allow holder of NFT to redeem liquidity from pool
#[derive(Accounts)]
pub struct DecreaseLiquidityPosition<'info> {
    /// Liquidity provider
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    /// Liquidity position
    #[account(mut,has_one = pool)]
    pub liquidity_position: Box<Account<'info, LiquidityPosition>>,

    /// Position token account
    /// holds the nft representing the liquidity
    /// position
    #[account(
       constraint = position_token_account.mint == liquidity_position.position_mint,
       constraint = position_token_account.amount == 1,
   )]
    pub position_token_account: Box<Account<'info, TokenAccount>>,

    /// Token pool account which holds overview
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    /// Associated token acount for tokens of type A
    #[account(mut,
       constraint = origin_account_a.mint == pool.token_mint_0
   )]
    pub origin_account_a: Box<Account<'info, TokenAccount>>,

    /// Associated token acount for tokens of type B
    #[account(mut,
       constraint = origin_account_b.mint == pool.token_mint_1
   )]
    pub origin_account_b: Box<Account<'info, TokenAccount>>,

    /// Pool Vault A to deposit into
    #[account(mut,
       constraint = vault_a.key() == pool.token_vault_0
   )]
    pub vault_a: Account<'info, TokenAccount>,

    /// Pool Vault A to deposit into
    #[account(mut,
       constraint = vault_b.key() == pool.token_vault_1
   )]
    pub vault_b: Account<'info, TokenAccount>,

    /// Lower tick array to use to deposit liquidity into
    #[account(mut,has_one = pool)]
    pub tick_array_lower: AccountLoader<'info, tick_v2::TickArray>,

    #[account(mut,has_one = pool)]
    pub tick_array_upper: AccountLoader<'info, tick_v2::TickArray>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,
}

impl<'info> Validate<'info> for DecreaseLiquidityPosition<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

pub fn handler(
    ctx: Context<DecreaseLiquidityPosition>,
    liquidity_amount: u64,
    token_min_a: u64,
    token_min_b: u64,
) -> Result<()> {
    // Check that the liquidity provider owns the
    // the liquidity position nft account
    validate_token_account_ownership(
        &ctx.accounts.position_token_account,
        &ctx.accounts.liquidity_provider,
    )?;

    let updated_liquidity_state = build_new_liquidity_state(
        ctx.accounts.liquidity_position.as_ref(),
        ctx.accounts.pool.as_ref(),
        &ctx.accounts.position_token_account,
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        liquidity_amount,
        false,
    )?;

    update_liquidity(
        &mut ctx.accounts.pool,
        &mut ctx.accounts.liquidity_position,
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        &updated_liquidity_state,
    )?;

    // Withdraw from vault to LP
    withdraw_from_vault(
        &ctx.accounts.pool,
        &ctx.accounts.vault_a,
        &ctx.accounts.origin_account_a,
        &ctx.accounts.token_program,
        updated_liquidity_state.token_0_delta,
    )?;

    // Withdraw from vault to LP
    withdraw_from_vault(
        &ctx.accounts.pool,
        &ctx.accounts.vault_b,
        &ctx.accounts.origin_account_b,
        &ctx.accounts.token_program,
        updated_liquidity_state.token_1_delta,
    )?;

    Ok(())
}
