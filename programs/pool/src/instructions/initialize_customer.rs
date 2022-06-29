use crate::states::*;
use anchor_lang::prelude::*;

/// --- Initialize Customer  ---
///
/// Prepare a new user for being able to buy insurance
///
#[derive(Accounts)]
pub struct InitializeCustomer<'info> {
    /// Signer - the new users
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Insurance Contracts
    #[account(
        init,
        space = 8 + InsuranceContracts::SPACE,
        payer = signer,
        seeds = [
            SURE_INSURANCE_CONTRACTS.as_bytes(),
            signer.key().as_ref(),
        ],
        bump,
    )]
    pub insurance_contracts: Box<Account<'info, InsuranceContracts>>,

    /// System program
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeCustomer>) -> Result<()> {
    let insurance_contracts = &mut ctx.accounts.insurance_contracts;
    insurance_contracts.owner = ctx.accounts.signer.key();
    insurance_contracts.bump = *ctx.bumps.get("insurance_contracts").unwrap();
    insurance_contracts.pools = Vec::new();
    emit!(InitializePolicyHolderEvent {
        owner: ctx.accounts.signer.key()
    });
    Ok(())
}

#[event]
pub struct InitializePolicyHolderEvent {
    pub owner: Pubkey,
}
