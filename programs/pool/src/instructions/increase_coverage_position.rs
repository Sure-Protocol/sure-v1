use crate::states::tick_v2::TickArray;
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::utils::{account};

/// Increase Coverage Position
///
/// Allow users to bind k times liquidity for a longer period of time
/// within
/// 
/// In cover
#[derive(Accounts)]
pub struct IncreaseCoveragePosition<'info> {
    /// Position owner
    pub owner: Signer<'info>,

    /// Pool to buy insurance from
    #[account(mut)]
    pub pool: Box<Account<'info, Pool>>,

    /// Position Mint
    pub position_mint: Account<'info, Mint>,

    /// Position Token account 
    pub position_token_account: Account<'info,TokenAccount>,

    /// Coverage Position
    #[account(
        mut, 
        constraint = coverage_position.position_mint == postion_mint
    )]
    pub coverage_position: Box<Account<'info, CoveragePosition>>,

    /// Coverage position owner token account
    #[account(
        constraint = coverage_position_owner_token_account_0.mint == pool.token_mint_0,
    )]
    pub coverage_position_owner_token_account_0: Box<Account<'info, TokenAccount>>,

    /// Token vault 0 to buy insurance from
    #[account(
        mut,
        constraint = token_vault_0.mint == coverage_position_onwer_token_account_0.mint
        constraint = token_vault_0 == pool.token_vault_0
    )]
    pub token_vault_0: Box<Account<'info, TokenAccount>>,

    /// Token vault 1 to deposit premium into
    /// Constraint: should be of same mint as token vault 0
    #[account(mut,
    constraint =token_vault_1.mint == token_vault_0.mint )]
    pub token_vault_1: Box<Account<'info,TokenAccount>>,

    
    /// Tick array 0
    /// First array to buy insurance from and
    /// where the current price is located
    #[account(mut,has_one = pool)]
    pub tick_array_0: AccountLoader<'info, TickArray>,

    /// Tick array 1
    /// Array after tick array 0 to buy from
    #[account(mut,has_one = pool)]
    pub tick_array_1: AccountLoader<'info, TickArray>,

    /// Tick array 2
    /// Array after tick array 1 to buy from
    #[account(mut,has_one = pool)]
    pub tick_array_2: AccountLoader<'info, TickArray>,
}

 
/// Icrease Coverage Position handler
/// 
/// Increase the amount coveraged by moving from lower to 
/// upper part of tick arrays. 
/// 
/// Assume that current price is at the first available tick array 
/// 
/// Premium is paid into seperate premium vault.  
/// The premium can be collected at any time 
pub fn handler(ctx: Context<IncreaseCoveragePosition>,coverage_amount: u64) -> Result<()>{

    // Validate the coverage position
    account::validate_token_account_ownership(&ctx.accounts.position_token_account, &ctx.accounts.owner)?;

    let mut tick_array_pool = TickArrayPool::new(&ctx.accounts.tick_array_0.load_mut().unwrap(), &ctx.accounts.tick_array_1.load_mut().ok(), &ctx.accounts.tick_array_2.load_mut().ok());

}
