use agnostic_orderbook::state::Side;
use anchor_lang::prelude::*;

use crate::state::*;

use super::provide_coverage::OrderParams;

#[derive(Accounts)]
pub struct BuyProtection<'info> {
    #[account(mut)]
    policy_holder: Signer<'info>,

    orderbook: OrderBook<'info>,
}

pub struct UpdateProtection {
    /// amount that is pending and can be
    /// closed on the ob
    pub amount_to_close: u64,

    /// amount that is matched and needs to be
    /// hedged
    pub amount_to_hedge: u64,
}

pub struct Protection {
    pub bump: u8,

    pub owner: Pubkey,

    pub order_id: u128,

    pub pending_protection: u64,

    pub protection: u64,
}

impl Protection {
    pub fn new(
        &mut self,
        bump: u8,
        owner: Pubkey,
        order_id: u128,
        pending_protection: u64,
        protection: u64,
    ) {
        self.bump = bump;
        self.owner = owner;
        self.order_id = order_id;
        self.pending_protection = pending_protection;
        self.protection = protection;
    }

    /// reduce protection by amount
    pub fn reduce_protection(&mut self, amount: u64) -> Result<UpdateProtection> {
        let amount_to_close = amount.min(self.pending_protection);
        let amount_to_hedge = if amount > self.pending_protection {
            amount - self.pending_protection
        } else {
            0
        };
        Ok(UpdateProtection {
            amount_to_close,
            amount_to_hedge,
        })
    }
}

/// handler for buying protection
///
/// fx:
/// - create a ask order, base = 1010, quote = 1000, limit 1010 with (limit -quote) collateral
/// - -10 in collateral, receive protection
/// - T: no breach  -> receives nothing
/// - T: breach -> receives 1000
pub fn handler(ctx: Context<BuyProtection>, order: OrderParams) -> Result<()> {
    // create ask
    let order_summary = ctx.accounts.orderbook.push_order(
        order.max_base_qty,
        order.max_quote_qty,
        order.limit_price,
        Side::Ask,
        &ctx.accounts.policy_holder.key(),
        false,
        false,
    );

    // mint protection position.

    // update state

    Ok(())
}
