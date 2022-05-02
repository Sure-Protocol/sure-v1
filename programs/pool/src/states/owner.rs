use anchor_lang::prelude::*;

/// Owner of the Sure Protocol
/// 
/// # Capabilities
/// 
/// * Adjust pool fee 
/// * Mint tokens 
/// 

#[account(zero_copy)]
pub struct ProtocolOwner {
    /// Bump
    pub bump: u8, // 1 bytes

    /// Owner of the protocol 
    pub owner: Pubkey, // 32bytes
}

impl ProtocolOwner {
    pub const SPACE:usize = 1 + 32;
}

/// Event for changing the owner of the protocol
#[event]
pub struct ChangeProtocolOwner {
    pub owner: Pubkey,
    pub old_owner: Pubkey,
}