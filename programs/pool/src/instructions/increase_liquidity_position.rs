use crate::states::*;
use crate::utils::{self, account, errors::SureError, liquidity};
use anchor_spl::token::{self};

use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::{
    associated_token::AssociatedToken,
    mint,
    token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer},
};
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
#[instruction(tick: u16,tick_pos: u64)]
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
        constraint = origin_account_a.mint == pool.token_mint_a
    )]
    pub origin_account_a: Box<Account<'info, TokenAccount>>,

    /// Associated token acount for tokens of type B
    #[account(mut,
        constraint = origin_account_b.mint == pool.token_mint_b
    )]
    pub origin_account_b: Box<Account<'info, TokenAccount>>,

    /// Pool Vault A to deposit into
    #[account(mut,
        constraint = vault_a.key() == pool.pool_vault_a
    )]
    pub vault_a: Account<'info, TokenAccount>,

    /// Pool Vault A to deposit into
    #[account(mut,
        constraint = vault_b.key() == pool.pool_vault_b
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
    let pool = &ctx.accounts.pool;
    let liquidity_position = &ctx.accounts.liquidity_position;
    if liquidity_amount == 0 {
        return Err(SureError::LiquidityHaveToBeGreaterThan0.into());
    }

    // Check that the liquidity provider owns the
    // the liquidity position nft account
    account::validate_token_account_ownership(
        &ctx.accounts.position_token_account,
        &ctx.accounts.liquidity_provider,
    )?;

    let productId = pool.productId;
    let liquidity_delta = liquidity::validate_liquidity_amount(liquidity_amount, true)?;

    // Calculate Tick changes
    // Get Tick accounts
    let tick_array_lower = &ctx.accounts.tick_array_lower.load()?;
    let tick_lower =
        tick_array_lower.get_tick(liquidity_position.tick_index_lower, pool.tick_spacing)?;
    let tick_array_upper = &ctx.accounts.tick_array_upper.load()?;
    let tick_upper =
        tick_array_upper.get_tick(liquidity_position.tick_index_upper, pool.tick_spacing)?;

    // Calculate the updated liquidity
    let next_pool_liquidity = pool.next_pool_liquidity(liquidity_position, liquidity_delta)?;

    // Update lower tick
    tick_lower.update_tick(
        liquidity_position.tick_index_lower,
        pool.get_current_tick_index()?,
        pool.fee_growth_0_x32,
        pool.fee_growth_1_x32,
        liquidity_delta,
        false,
    )?;
    // Update upper tick
    tick_upper.update_tick(
        liquidity_position.tick_index_upper,
        pool.get_current_tick_index()?,
        pool.fee_growth_0_x32,
        pool.fee_growth_1_x32,
        liquidity_delta,
        true,
    )?;

    // Update liquidity position
    let (fee_growth_inside_0, fee_growth_inside_1) = tick_lower.calculate_next_fee_growth(
        liquidity_position.tick_index_lower,
        tick_upper,
        liquidity_position.tick_index_upper,
        pool.get_current_tick_index()?,
        pool.fee_growth_0_x32,
        pool.fee_growth_1_x32,
    )?;

    Ok(())
}

#[event]
pub struct NewLiquidityPosition {
    pub tick: u16,
    pub liquidity: u64,
}
