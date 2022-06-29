use crate::helpers::tick;
use crate::states::*;
use anchor_lang::prelude::*;
use std::mem::size_of;

#[derive(Accounts)]
#[instruction(sqrt_price_x32: u64)]
pub struct InitializeTick<'info> {
    /// Signer of the transaction
    #[account(mut)]
    pub creator: Signer<'info>,

    pub pool: Box<Account<'info, Pool>>,

    /// Tick array
    /// for which the newly created tick is a part of
    #[account(mut)]
    pub tick_array: AccountLoader<'info, TickArray>,

    /// Create tick account
    #[account(
        init,
        payer = creator,
        seeds = [
            SURE_TICK_SEED.as_bytes(),
            pool.key().as_ref(),
            sqrt_price_x32.to_le_bytes().as_ref()
        ],
        bump,
        space = 8 + size_of::<Tick>(),
    )]
    pub tick: AccountLoader<'info, Tick>,

    /// System program required to make changes
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeTick>, sqrt_price_x32: u64) -> Result<()> {
    let mut liquidity_tick_info = ctx.accounts.tick.load_init()?;
    let pool = ctx.accounts.pool;
    let mut tick_array = ctx.accounts.tick_array.load_mut()?;
    let tick = tick::get_tick_at_sqrt_ratio(sqrt_price_x32)?;

    // Flip tick at tick_array
    tick_array.flip_bit(tick, pool.tick_spacing)?;

    // Initialize tick
    liquidity_tick_info.initialize(
        *ctx.bumps.get("liquidity_tick_info").unwrap(),
        sqrt_price_x32,
    )?;
    Ok(())
}
