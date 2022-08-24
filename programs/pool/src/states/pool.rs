use crate::states::fee_package::*;
use crate::utils::*;
use anchor_lang::prelude::*;

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
pub struct Pool {
    /// Bump to identify the PDA
    pub bump: u8, // 1 byte
    pub bump_array: [u8; 1],

    ///
    /// Name of pool visible to the user
    pub name: String, // 4 + 200 bytes

    // spaces between ticks
    pub tick_spacing: u16,
    pub tick_spacing_array: [u8; 2],

    // founder
    pub founder: Pubkey,

    /// fees
    /// 100th of a basis point
    pub fee_rate: u16,
    /// (1/x)% of fee_rate
    pub protocol_fee: u16,
    pub founders_fee: u16,

    /// Fee paid when buying insurance.
    /// in basis points
    pub insurance_fee: u16, // 4 bytes

    /// The public key of the smart contract that is
    /// insured
    pub smart_contract: Pubkey, // 32 bytes

    /// Vec of token Pools
    pub token_pools: Vec<Pubkey>, // 4 + 32*64, 64 tokens for each pool

    /// Whether the insurance pool is locked
    pub locked: bool, // 1 byte

    // Mint of
    pub token_mint_0: Pubkey, // 32 bytes
    pub vault_0: Pubkey,      //32 bytes

    /// Token mint B of pool
    pub token_mint_1: Pubkey, // 32 bytes
    pub vault_1: Pubkey, //32 bytes

    /// Used liquidity
    pub used_liquidity: u128, // 8 bytes
}

impl Pool {
    pub const SPACE: usize = 1 + 4 + 200 + 4 + 32 + 4 + 32 * 64 + 1;

    pub fn seeds(&self) -> [&[u8]; 3] {
        [
            &SURE_PREMIUM_POOL_SEED.as_bytes() as &[u8],
            self.smart_contract.as_ref(),
            self.bump_array.as_ref(),
        ]
    }

    pub fn initialize(
        &mut self,
        bump: u8,
        name: String,
        founder: Pubkey,
        tick_spacing: u16,
        fee_package: &Account<FeePackage>,
        token_mint_0: Pubkey,
        token_mint_1: Pubkey,
        vault_0: Pubkey,
        vault_1: Pubkey,
    ) -> Result<()> {
        self.bump = bump;
        self.bump_array = bump.to_le_bytes();
        self.name = name;
        self.founder = founder;
        self.tick_spacing = tick_spacing;
        self.tick_spacing_array = tick_spacing.to_le_bytes();
        self.token_mint_0 = token_mint_0;
        self.token_mint_1 = token_mint_1;
        self.vault_0 = vault_0;
        self.vault_1 = vault_1;
        Ok(())
    }
}

#[event]
pub struct CreatePool {
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
    pub insurance_fee: u16,
}

#[event]
pub struct InitializeTokenPool {}
