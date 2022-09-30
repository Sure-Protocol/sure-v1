use agnostic_orderbook::{
    instruction::cancel_order,
    instruction::new_order,
    state::{OrderSummary, Side},
};
use anchor_lang::prelude::*;

use crate::utils::{CallbackInfo, ShieldError};

#[derive(Accounts)]
pub struct OrderBook<'info> {
    #[account(mut)]
    pub market: AccountInfo<'info>,
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,
    #[account(mut)]
    pub asks: AccountInfo<'info>,
    #[account(mut)]
    pub bids: AccountInfo<'info>,
}

impl<'info> OrderBook<'info> {
    pub fn new_order_accounts(&self) -> new_order::Accounts<AccountInfo<'info>> {
        new_order::Accounts {
            market: &self.market,
            event_queue: &self.event_queue,
            asks: &self.asks,
            bids: &self.bids,
        }
    }

    pub fn new_cancel_accounts(&self) -> cancel_order::Accounts<AccountInfo<'info>> {
        cancel_order::Accounts {
            market: &self.market,
            event_queue: &self.event_queue,
            asks: &self.asks,
            bids: &self.bids,
        }
    }

    pub fn push_order(
        &self,
        max_coverage_value: u64,
        max_coverage: u64,
        limit_price: u64,
        side: Side,
        owner: &Pubkey,
        post_only: bool,
        post_allowed: bool,
    ) -> Result<OrderSummary> {
        let callback_info = CallbackInfo::new(*owner);
        let order_params = agnostic_orderbook::instruction::new_order::Params::<CallbackInfo> {
            max_base_qty: max_coverage_value,
            max_quote_qty: max_coverage,
            limit_price: limit_price,
            match_limit: limit_price,
            side: side,
            callback_info: callback_info,
            post_only: post_only,
            post_allowed: post_allowed,
            self_trade_behavior: agnostic_orderbook::state::SelfTradeBehavior::AbortTransaction,
        };

        let order_accounts = self.new_order_accounts();

        let order_summary = agnostic_orderbook::instruction::new_order::process(
            &crate::id(),
            order_accounts,
            order_params,
        )?;

        require!(
            order_summary.posted_order_id.is_some() || order_summary.total_base_qty > 0,
            ShieldError::CoveragePositionRejected
        );
        Ok(order_summary)
    }

    /// cancels an existing order
    pub fn cancel_order(&self, order_id: u128) -> Result<OrderSummary> {
        let order_params = cancel_order::Params { order_id };
        let cancel_order_accounts = self.new_cancel_accounts();
        let order_summary = cancel_order::process::<CallbackInfo>(
            &crate::id(),
            cancel_order_accounts,
            order_params,
        )?;
        Ok(order_summary)
    }
}
