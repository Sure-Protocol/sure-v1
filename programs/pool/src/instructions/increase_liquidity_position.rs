use crate::states::*;
use crate::utils::*;
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
use vipers::*;

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
///  - Initializing a liquidity position
///  - Transfer tokens to the correct vaults
///  -
pub fn handler(
    ctx: Context<IncreaseLiquidityPosition>,
    tick: u16,
    tick_pos: u64,
    amount: u64,
) -> Result<()> {
    Ok(())
}

#[event]
pub struct NewLiquidityPosition {
    pub tick: u16,
    pub liquidity: u64,
}
