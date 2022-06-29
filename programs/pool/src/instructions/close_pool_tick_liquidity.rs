use crate::states::*;
use crate::utils::*;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
pub struct ClosePoolTickLiquidity<'info> {
    // Account to receive remaining rent
    pub recipient: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        close = recipient,
    )]
    pub liquidity_tick_info: AccountLoader<'info, Tick>,
}

pub fn handler(ctx: Context<ClosePoolTickLiquidity>) -> Result<()> {
    let liquidity_tick_info_state =
        AccountLoader::<tick::Tick>::try_from(&ctx.accounts.liquidity_tick_info.to_account_info())?;
    let liquidity_tick_info = liquidity_tick_info_state.load_mut()?;

    if !liquidity_tick_info.is_pool_empty() {
        return Err(error!(errors::SureError::TickAccountNotEmpty));
    }
    Ok(())
}
