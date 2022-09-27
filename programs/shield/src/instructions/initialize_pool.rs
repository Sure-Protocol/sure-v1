use agnostic_orderbook::instruction::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    *,
};

use crate::utils::SURE_SHIELD;
use crate::{state::pool::*, utils::CallbackInfo};

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(
        init,
        space = 8 + Pool::SPACE,
        payer = creator,
        seeds =[
            SURE_SHIELD.as_bytes(),
            smart_contract.key().to_bytes().as_ref()
        ],
        bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    // smart contract to be insured
    #[account(
        constraint = smart_contract.executable == true
    )]
    pub smart_contract: UncheckedAccount<'info>,

    #[account()]
    pub vault_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = creator,
        seeds = [
            SURE_SHIELD.as_bytes(),
            pool.key().as_ref(),
        ],
        bump,
        token::mint = vault_mint,
        token::authority = pool,
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    // === accounts for the AOB ===
    // market
    #[account(mut)]
    pub orderbook_market: AccountInfo<'info>,

    // event queue
    #[account(mut)]
    pub event_queue: AccountInfo<'info>,

    // bids
    #[account(mut)]
    pub bids: AccountInfo<'info>,

    // ask
    #[account(mut)]
    pub asks: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

/// initialize pool
///
/// initializes pool acount and creates a new serum market
///
pub fn handler(ctx: Context<InitializePool>) -> Result<()> {
    let pool = ctx.accounts.pool.as_mut();

    // initialize pool
    pool.initialize(
        ctx.bumps["pool"],
        "test",
        ctx.accounts.creator.key,
        ctx.accounts.smart_contract.key,
        &ctx.accounts.vault_mint.key(),
        &ctx.accounts.vault.key(),
        ctx.accounts.orderbook_market.key,
        ctx.accounts.event_queue.key,
        ctx.accounts.asks.key,
        ctx.accounts.asks.key,
    );

    // create new market on serum
    let create_market_accounts = create_market::Accounts {
        market: &ctx.accounts.orderbook_market.to_account_info(),
        event_queue: &ctx.accounts.event_queue.to_account_info(),
        bids: &ctx.accounts.bids.to_account_info(),
        asks: &ctx.accounts.asks.to_account_info(),
    };

    let create_market_params = create_market::Params {
        min_base_order_size: 1_000_000_000 as u64,
        tick_size: 1_u64,
    };

    create_market::process::<CallbackInfo>(
        ctx.program_id,
        create_market_accounts,
        create_market_params,
    )?;
    Ok(())
}
