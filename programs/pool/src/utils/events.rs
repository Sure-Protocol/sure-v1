use anchor_lang::prelude::*;

/// Initialized Pool Manager Event
#[event]
pub struct InitializedManager {
    #[index]
    pub owner: Pubkey,
}
