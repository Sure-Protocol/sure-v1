use anchor_lang::prelude::*;

/// Initialized Pool Manager Event
#[event]
pub struct InitializedManager {
    #[index]
    pub owner: Pubkey,
}

#[event]
pub struct InitializedPool{
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
}


