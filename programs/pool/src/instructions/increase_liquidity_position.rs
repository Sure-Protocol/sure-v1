use crate::common::account::validate_token_account_ownership;
use crate::common::liquidity::{calculate_token_0_delta, calculate_token_1_delta};
use crate::common::token_tx::deposit_into_vault;
use crate::common::{self, account, errors::SureError, liquidity};
use crate::factory::liquidity::{build_new_liquidity_state, update_liquidity};
use crate::factory::*;
use crate::states::*;
use anchor_spl::token::{self};

use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};
use mpl_token_metadata::instruction::{create_metadata_accounts_v2, update_metadata_accounts_v2};
use mpl_token_metadata::state::Creator;

/// --- Deposit Liquidity ---
///
/// Deposits liquidity into a
///
/// Liquidity Positions on Sure is represented as an NFT.
/// The holder has the right to manage the liquidity position
///
/// The associated method does
///     - Mint a new Liquidity Position NFT
///     - Transfer capital to pool vault
///     - Creates liquidity position
///     - Updates
///
/// Initializes:
///     - nft_mint: Mint associated with the liquidity NFT position
///     - liquidity_position: keeps a summary of liquidity position
///     - nft_account: nft account to hold the newly minted Liquidity position NFT
///
#[derive(Accounts)]
pub struct IncreaseLiquidityPosition<'info> {
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

/// Increase Liquidity Position
/// is responsible for
///  - Calculating the amounts that have to be transferred to the pools
///  - Updating the liquidity position
///  - Updating
///  - Transfer tokens to the correct vaults
///  
/// If it is a one sided pool (insurance, loan) the liquidity amount is the amount
/// to be transferred into vault a. Vault b would stay empty and will be used for premiums
///
/// If it is a
pub fn handler(
    ctx: Context<IncreaseLiquidityPosition>,
    liquidity_amount: u64, // Amount of liquidity in token a
    max_token_a: u64,      // Max amount of token a that can be deposited
    min_token_b: u64,      // Max amount of token b that can be deposited
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
        true,
    )?;

    update_liquidity(
        &mut ctx.accounts.pool,
        &mut ctx.accounts.liquidity_position,
        &ctx.accounts.tick_array_lower,
        &ctx.accounts.tick_array_upper,
        &updated_liquidity_state,
    )?;

    // Deposit tokens 0 into vault
    deposit_into_vault(
        &ctx.accounts.liquidity_provider,
        &ctx.accounts.vault_a,
        &ctx.accounts.origin_account_a,
        &ctx.accounts.token_program,
        liquidity_amount,
    )?;

    // Deposit token 1 into vault
    deposit_into_vault(
        &ctx.accounts.liquidity_provider,
        &ctx.accounts.vault_b,
        &ctx.accounts.origin_account_b,
        &ctx.accounts.token_program,
        liquidity_amount,
    )?;
    Ok(())
}

#[event]
pub struct NewLiquidityPosition {
    pub tick: u16,
    pub liquidity: u64,
}
