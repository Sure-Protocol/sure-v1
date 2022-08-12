use crate::{
    seeds::*,
    states::{owner::ProtocolOwner, pool::Pool},
    LiquidityPosition,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

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
pub struct UpdateLiquidity<'info> {
    /// Liquidity provider
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    /// Associated token account to credit
    #[account(mut)]
    pub liquidity_provider_token_account: Box<Account<'info, TokenAccount>>,

    /// Pool which owns token account
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    /// Pool Vault account to deposit liquidity to
    #[account(mut)]
    pub premium_vault: Account<'info, TokenAccount>,

    /// Create Liquidity position
    /// HASH: [sure-lp,liquidity-provider,pool,token,tick]
    #[account(
        init,
        payer = liquidity_provider,
        seeds = [
            SURE_LIQUIDITY_POSITION.as_bytes(),
            liquidity_position_nft_mint.key().as_ref()
        ],
        space = 8 + LiquidityPosition::SPACE,
        bump,
    )]
    pub liquidity_position: Box<Account<'info, LiquidityPosition>>,

    pub liquidity_position_nft_mint: Box<Account<'info, Mint>>,

    /// CHECK: done in method
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// Program id for metadata program
    /// CHECK: checks that the address matches the mpl token metadata id
    //#[account(address =mpl_token_metadata::ID )]
    pub metadata_program: UncheckedAccount<'info>,

    /// Sysvar for token mint and ATA creation
    pub rent: Sysvar<'info, Rent>,

    // Token program that executes the transfer
    pub token_program: Program<'info, Token>,

    /// Provide the system program
    pub system_program: Program<'info, System>,
}

/// Increase liquidity Position
pub fn handler(ctx: Context<UpdateLiquidity>) -> Result<()> {
    Ok(())
}
