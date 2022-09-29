use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;

use crate::utils::CallbackInfo;

pub struct OrderBook {}

impl OrderBook {
    pub fn push_order(
        &self,
        max_coverage_value: u64,
        max_coverage: u64,
        limit_price: u64,
        side: Side,
    ) -> Result<()> {
        // let callbackIn
        // let order_params = agnostic_orderbook::instruction::new_order::Params::<CallbackInfo> {
        //     max_base_qty: max_coverage_value,
        //     max_quote_qty: max_coverage,
        //     limit_price: limit_price,
        //     match_limit: limit_price,
        //     side: side,
        //     callback_info: callback_info,
        //     post_only: order.post_only,
        //     post_allowed: order.post_allowed,
        //     self_trade_behavior: agnostic_orderbook::state::SelfTradeBehavior::AbortTransaction,
        // };

        // let order_accounts = agnostic_orderbook::instruction::new_order::Accounts {
        //     market: ctx.accounts.market_orderbook.as_ref(),
        //     event_queue: &ctx.accounts.event_queue,
        //     asks: &ctx.accounts.asks,
        //     bids: &ctx.accounts.bids,
        // };

        // let order_summary = agnostic_orderbook::instruction::new_order::process(
        //     ctx.program_id,
        //     order_accounts,
        //     order_params,
        // )?;

        // require!(
        //     order_summary.posted_order_id.is_some() || order_summary.total_base_qty > 0,
        //     ShieldError::CoveragePositionRejected
        // );
        Ok(())
    }

    pub fn cancel_order(&self) -> Result<()> {
        Ok(())
    }
}
