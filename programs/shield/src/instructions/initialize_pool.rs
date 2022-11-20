use std::ops::Mul;

use agnostic_orderbook::instruction::*;
use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount},
    *,
};

use crate::utils::SURE_SHIELD;
use crate::{state::pool::*, utils::CallbackInfo};
use oracle::cpi::accounts::ProposeVote;
use oracle::{accounts::Config, instructions::propose_vote};

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    #[account(mut)]
    pub creator_account: Box<Account<'info, TokenAccount>>,

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

    pub propose_vote: ProposeVote,

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

    // sure oracle program
    pub sure_oracle_program: AccountInfo<'info>,

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

    // propose vote on Sure prediction market
    let oracle_program = ctx.accounts.sure_oracle_program.to_account_info();
    let propose_vote_ctx = CpiContext::new(oracle_program, ctx.accounts.propose_vote);
    // todo: generate values
    let id = vec![10];
    let name = String::from("new markets");
    let description = String::from("new market");
    let stake = 10.mul(100000000);

    oracle::cpi::propose_vote(propose_vote_ctx, id, name, description, stake)?;

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
