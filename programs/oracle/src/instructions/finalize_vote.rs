use crate::states::Proposal;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct FinalizeVote<'info> {
    #[account(mut)]
    pub finalizer: Signer<'info>,

    #[account(mut)]
    pub proposal: Box<Account<'info, Proposal>>,

    pub system_program: Program<'info, System>,
}

/// Finalize vote
///
/// when the reveal period is over it is time
/// to close the vote and calculate the necessary parameters
/// in order to distribute rewards
///
/// anyone can finalize the vote
pub fn handle(ctx: Context<FinalizeVote>) -> Result<()> {
    let proposal = ctx.accounts.proposal.as_mut();
    Ok(())
}
