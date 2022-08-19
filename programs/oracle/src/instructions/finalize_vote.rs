use anchor_lang::prelude::*;

#[Accounts]
pub struct FinalizeVote<'info> {}

pub fn handle(ctx: Context<FinalizeVote>) -> Result<()> {}
