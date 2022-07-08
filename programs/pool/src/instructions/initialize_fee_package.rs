use crate::common::seeds::*;
use crate::states::fee::{self, FeePackage};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeFeePackage<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        payer = owner,
        seeds = [
            SURE_DOMAIN.as_bytes(),
        ],
        space=8 + FeePackage::SIZE,bump
    )]
    pub fee_package: Box<Account<'info, FeePackage>>,

    // System program
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeFeePackage>,
    fee_rate: u16,
    protocol_fee_rate: u16,
    founders_fee_rate: u16,
) -> Result<()> {
    let fee_package = ctx.accounts.fee_package.as_mut();
    fee_package.initialize(
        &ctx.accounts.owner,
        fee_rate,
        protocol_fee_rate,
        founders_fee_rate,
    )?;
    Ok(())
}
