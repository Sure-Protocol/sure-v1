use std::ops::Div;

use anchor_lang::{prelude::*, solana_program::{clock, sysvar::clock}};
use anchor_spl::token::{Mint,TokenAccount};

use crate::utils::SureError;

/// 
#[account]
pub struct VeTokens{
    pub bump: u8,

    // Owner of veTokens
    pub owner: Pubkey,

    // Mint of tokens
    pub token_mint: Pubkey,

    // Amount of tokens held in custidy
    pub token_amount_remaining: u128,

    /// issed ve tokens
    pub ve_tokens_issued: u128,

    // is unlocking
    pub is_unlocking: bool,

    // remaining locking period 
    pub lock_period_ts: i64,
}

impl VeTokens {
    pub const SPACE: usize = 1 + 32;
    // 4 years
    pub const MAX_LOCK_PERIOD_SECONDS: usize = 126227704;
    pub const SECONDS_IN_YEAR: i64 = 31556926;

    pub fn initialize(&mut self, bump: u8,owner: Pubkey,token_mint: Pubkey,lock_period_ts: i64,token_amount: u128) -> Result<()>{

        self.bump = bump;
        self.owner = owner;
        self.token_mint = token_mint;

        if lock_period_ts > MAX_LOCK_PERIOD_SECONDS || lock_period_ts < 0 {
            return Err(SureErro::InvalidLockPeriod.into())
        }

        self.lock_period_ts = lock_period_ts;
        self.is_unlocking = false;
        
        let ve_sure_tokens_issued = token_amount*(lock_period_ts.div(SECONDS_IN_YEAR));

        Ok(())
    }
}


#[derive(Accounts)]
pub struct LockTokens {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init, 
        payer = owner,
        seeds = [
            b"sure-ve",
            owner.key().as_ref(),
            token_mint.key().as_ref()
        ],
        bump,
        space = 8 + VeTokens::SPACE,
    )]
    pub ve_token_account: Box<Account<'info,VeTokens>>,

    /// Token account that holds the locked tokens
    pub ve_token_vault: Box<Account<'info,TokenAccount>>,

    pub token_mint: Box<Account<'info,Mint>>,

    pub system_program: Program<'info,System>,

}