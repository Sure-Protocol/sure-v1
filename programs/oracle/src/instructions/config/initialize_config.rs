use crate::states::*;
use crate::utils::SURE_ORACLE_CONFIG_SEED;
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [
            SURE_ORACLE_CONFIG_SEED.as_bytes().as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        space = 8 + Config::SPACE,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account()]
    pub token_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
}

/// initialize config
///
/// unique for a given token mint
///
/// ### args
/// * protocol_authority<Pubkey>: permissions
///     - update config
///     - collect protocol fees
pub fn handler(
    ctx: Context<InitializeConfig>,
    protocol_authority: Pubkey,
    required_votes_fraction: u64,
) -> Result<()> {
    let config = ctx.accounts.config.as_mut();
    config.initialize(
        ctx.accounts.token_mint.as_ref(),
        protocol_authority,
        required_votes_fraction,
    );

    emit!(InitializedConfigEvent {});
    Ok(())
}

#[event]
pub struct InitializedConfigEvent {}
