use agnostic_orderbook::{
    state::{OrderSummary, Side},
    *,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use sure_common::fp::fp_from_float;

use crate::state::*;
use crate::utils::{CallbackInfo, ShieldError, SURE_SHIELD};

#[derive(Clone, Copy, AnchorDeserialize, AnchorSerialize)]
pub struct OrderParams {
    /// The maximum quantity of base to be traded.
    pub max_base_qty: u64,
    /// The maximum quantity of quote to be traded.
    pub max_quote_qty: u64,
    /// The limit price of the order. This value is understood as a 32-bit fixed point number.
    /// Must be rounded to the nearest tick size multiple (see [`round_price`][`crate::utils::round_price`])
    pub limit_price: u64,

    pub side: Side,
    /// The maximum number of orders to match against before performing a partial fill.
    ///
    /// It is then possible for a caller program to detect a partial fill by reading the [`OrderSummary`][`crate::orderbook::OrderSummary`]
    /// in the event queue register.
    pub match_limit: u64,
    /// The order will not be matched against the orderbook and will be direcly written into it.
    ///
    /// The operation will fail if the order's limit_price crosses the spread.
    pub post_only: bool,
    /// The order will be matched against the orderbook, but what remains will not be written as a new order into the orderbook.
    pub post_allowed: bool,
}

#[derive(Accounts)]
pub struct ProvideCoverage<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    pub coverage_mint: Account<'info, Mint>,

    pub coverage_mint_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = provider,
        space = 8 + CoveragePosition::SPACE,
        seeds = [
            SURE_SHIELD.as_bytes(),
            coverage_mint.key().as_ref()
        ],
        bump
    )]
    pub coverage_position: Box<Account<'info, CoveragePosition>>,

    /// pool vault to hold deposits
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,

    /// provider token account
    #[account(mut)]
    pub provider_vault: Account<'info, TokenAccount>,

    // == serum acounts ==
    /// orderbook
    pub orderbook: OrderBook<'info>,

    /// === metaplex accounts ====
    /// CHECK: checked in instruction
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// Program id for metadata program
    /// CHECK: checks that the address matches the mpl token metadata id
    //#[account(address =mpl_token_metadata::ID )]
    #[account(address = mpl_token_metadata::ID)]
    pub metadata_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// handler for selling coverage
///
/// fx:
/// - create bid order of base = inf, quote=1000, limit 1020
/// - pays 1000, receive bond of 1020 given no breach
/// - entitled to 20 over the lifetime of the shield
/// - T: no breach -> receives 1020
/// - T: breach -> receives 20
pub fn handler(ctx: Context<ProvideCoverage>, order: OrderParams) -> Result<()> {
    // push to orderbook
    // TODO: move create new order to utils

    let orderbook = &ctx.accounts.orderbook;
    let order_summary = orderbook.push_order(
        u64::MAX,
        order.max_quote_qty,
        fp_from_float(1. / 1.02),
        Side::Bid,
        &ctx.accounts.provider.key(),
        false,
        false,
    )?;

    // set coverage position
    let coverage_position = ctx.accounts.coverage_position.as_mut();
    coverage_position.initialize(
        *ctx.bumps.get("coverage_position").unwrap(),
        &ctx.accounts.coverage_mint.key(),
        order_summary.total_base_qty_posted,
    );
    let total_provided = order_summary.total_base_qty + order_summary.total_base_qty_posted;
    coverage_position.provide_coverage(
        order_summary.total_quote_qty,
        order_summary.total_base_qty - order_summary.total_quote_qty,
        order_summary.posted_order_id,
    );

    // TODO: mint tokens to represent position

    // transfer amount provided
    sure_common::token::deposit_into_vault(
        &ctx.accounts.provider,
        &ctx.accounts.vault,
        &ctx.accounts.provider_vault,
        &ctx.accounts.token_program,
        total_provided,
    )?;

    // emit event
    emit!(ProvidedCoverage { order_summary });

    Ok(())
}

#[event]
pub struct ProvidedCoverage {
    order_summary: OrderSummary,
}
