use crate::common::seeds::*;
use crate::states::*;
use anchor_lang::prelude::*;
use std::mem::size_of;

/// Initialize Tick Array
///
/// The tick array holds information about which ticks
/// contains liquidity
#[derive(Accounts)]
#[instruction(start_tick_index: i16)]
pub struct InitializeTickArray<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Pool associated with the tick array / bitmap
    pub pool: Account<'info, Pool>,

    /// Initialize tick array
    /// Each pool can consist of one or more tick arrays
    /// Each array holds information about whether 256 ticks
    /// is initialized or not
    #[account(
        init,
        seeds = [
            SURE_BITMAP.as_bytes(),
            pool.token_mint_0.as_ref(),
            pool.token_mint_1.as_ref(),
            &pool.fee_rate.to_be_bytes(),
            &start_tick_index.to_be_bytes()
        ],
        bump,
        payer=creator,
        space = 8 + size_of::<TickArray>()
    )]
    pub tick_array: AccountLoader<'info, tick_v2::TickArray>,

    /// System Program
    pub system_program: Program<'info, System>,
}

/// InitializeTickArray
pub fn handler(ctx: Context<InitializeTickArray>, start_tick_index: i32) -> Result<()> {
    let tick_array = &mut ctx.accounts.tick_array.load_init()?;
    tick_array.initialize(&ctx.accounts.pool, start_tick_index)?;
    Ok(())
}
