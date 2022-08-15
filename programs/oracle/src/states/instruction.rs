use anchor_lang::prelude::*;

/// Invoked if a vote is successful
#[account]
#[derive(Default, Copy)]
pub struct VoteInstruction {
    pub program_id: Pubkey,      // 32 bytes
    pub keys: [AccountKeys; 24], // 4 + len*
    pub data: [u8; 24],          // 4 + 1*len
}

impl VoteInstruction {
    pub const SPACE: usize = 32 + 4 + 24 * AccountKeys::SPACE + 24 * 1;

    /// Invokes proposal
    /// TODO
    pub fn invoke_proposal(&self) -> Result<()> {
        Ok(())
    }
}

#[account]
#[derive(Default, Copy)]
pub struct AccountKeys {
    pub account_pubkey: Pubkey, // 32 bytes
    pub is_signer: bool,        // 1 byte
    pub is_writable: bool,      // 1 byte
}

impl AccountKeys {
    pub const SPACE: usize = 32 + 1 + 1;
}
