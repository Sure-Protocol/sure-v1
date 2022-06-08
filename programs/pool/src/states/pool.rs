
use anchor_lang::{prelude::*, solana_program::account_info::Account};

/// Account describing the pool manager
///
#[account]
#[derive(Default)]
pub struct PoolManager {
    // the current pool manager
    pub owner: Pubkey, // 32 bytes
    // bump to identify the PDA
    pub bump: u8, // 1 byte
}

impl PoolManager {
    pub const SIZE: usize = 32 + 1;
}

/// SurePools holds information on which programs are
/// insured by Sure
#[account]
pub struct SurePools {
    pub bump: u8, // 1 byte

    /// Vec of insured programs
    pub pools: Vec<Pubkey>, // 4 + 32*256 = 8196, 256 insured contracts
}

impl SurePools {
    pub const SPACE: usize = 1 + 4 + 32 * 256;
}

/// Pool Account (PDA) contains information describing the
/// insurance pool
#[account]
pub struct PoolAccount {
    /// Bump to identify the PDA
    pub bump: u8, // 1 byte

    /// Name of pool visible to the user
    pub name: String, // 4 + 200 bytes

    /// Fee paid when buying insurance.
    /// in basis points
    pub insurance_fee: u16, // 4 bytes

    /// The public key of the smart contract that is
    /// insured
    pub smart_contract: Pubkey, // 32 bytes

    /// Whether the insurance pool is locked
    pub locked: bool, // 1 byte
}

impl PoolAccount {
    pub const SPACE: usize = 1 + 4 + 200 + 4  + 32 + 1;
}

#[event]
pub struct CreatePool {
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
    pub insurance_fee: u16,
}

#[event]
pub struct CreatePoolVaults {}
