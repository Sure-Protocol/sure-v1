use agnostic_orderbook::{
    state::{OrderSummary, Side},
    *,
};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

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

/// coverage position
/// holds information about the
#[account]
pub struct CoveragePosition {
    /// pda bump
    pub bump: u8, // 1 byte

    /// mint of position
    pub mint: Pubkey, // 32 bytes
}

impl CoveragePosition {
    pub const SPACE: usize = 0;

    pub fn initialize(&mut self, bump: u8, mint: &Pubkey) {
        self.bump = bump;
        self.mint = *mint;
    }
}

#[derive(Accounts)]
pub struct ProvideCoverage<'info> {
    #[account(mut)]
    pub provider: Signer<'info>,

    pub coverage_mint: Account<'info, Mint>,

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

    // == serum acounts ==
    /// orderbook
    #[account(mut)]
    pub market_orderbook: AccountInfo<'info>,

    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

    #[account(mut)]
    pub asks: AccountInfo<'info>,

    #[account(mut)]
    pub bids: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ProvideCoverage>, order: OrderParams) -> Result<()> {
    // create coverage position

    // create serum order
    // TODO: move create new order to utils
    let callback_info = CallbackInfo::new(ctx.accounts.provider.key());

    let order_params = agnostic_orderbook::instruction::new_order::Params::<CallbackInfo> {
        max_base_qty: order.max_base_qty,
        max_quote_qty: order.max_quote_qty,
        limit_price: order.limit_price,
        match_limit: order.limit_price,
        side: Side::Ask,
        callback_info: callback_info,
        post_only: order.post_only,
        post_allowed: order.post_allowed,
        self_trade_behavior: agnostic_orderbook::state::SelfTradeBehavior::AbortTransaction,
    };

    let order_accounts = agnostic_orderbook::instruction::new_order::Accounts {
        market: ctx.accounts.market_orderbook.as_ref(),
        event_queue: &ctx.accounts.event_queue,
        asks: &ctx.accounts.asks,
        bids: &ctx.accounts.bids,
    };

    let order_summary = agnostic_orderbook::instruction::new_order::process(
        ctx.program_id,
        order_accounts,
        order_params,
    )?;

    require!(
        order_summary.posted_order_id.is_some() || order_summary.total_base_qty > 0,
        ShieldError::CoveragePositionRejected
    );

    // mint coverage position

    // emit event
    emit!(ProvidedCoverage { order_summary });

    Ok(())
}

#[event]
pub struct ProvidedCoverage {
    order_summary: OrderSummary,
}
