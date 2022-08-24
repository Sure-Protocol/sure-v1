use crate::states::*;
use crate::utils::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitializeCoveragePosition<'info> {
    /// Signer - the new users
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Insurance Contracts
    #[account(
        init,
        space = 8 + CoveragePosition::SPACE,
        payer = signer,
        seeds = [
            SURE_INSURANCE_CONTRACT.as_bytes(),
            signer.key().as_ref(),
        ],
        bump,
    )]
    pub insurance_contracts: AccountLoader<'info, CoveragePosition>,

    /// System program
    pub system_program: Program<'info, System>,
}
