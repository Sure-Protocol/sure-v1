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
    pub const POOL_MANAGER_SIZE: usize = 32 + 1;
}

/// Pool Account (PDA) contains information describing the 
/// insurance pool 
#[account]
#[derive(Default)]
pub struct PoolAccount {
    /// Bump to identify the PDA
    pub bump: u8, // 1 byte 

    /// Token held in the pool.
    /// In the beginning this is just USDC
    pub token: Pubkey, // 32 bytes 

    /// Fee paid when buying insurance. 
    /// in 10^-6
    pub insurance_fee: i32, // 4 bytes

    /// Size of range to provide liquidity in
    /// Measured in basis points. Standard is 1 (basis point, 0.01%)
    pub range_size: i32, // 4 bytes 

    /// Number of ranges 
    pub ranges: i32, //4 bytes,

    /// The total liquidity in the pool 
    pub liquidity: u64, // 8 bytes

    /// Available Liquidity in the pool
    pub free_liquidity: u64, // 8 bytes 

    /// Current premium rate in basis points (0.01%). 
    pub premium_rate: u64, // 8 bytes

    /// Name of pool visible to the user
    pub name: String, // 4 + 200 bytes

    /// The public key of the smart contract that is
    /// insured 
    pub smart_contract: Pubkey, // 32 bytes

    /// Vault that holds the liquidity (tokens)
    pub vault: Pubkey, // 32 bytes

    /// Whether the insurance pool is locked 
    pub locked: bool, // 1 byte 
}

impl PoolAccount{
    pub const SPACE:usize = 1+32+4+4+4+8+8+8+4+200+32+32+1;
}

/// Deposit Liquidity
/// allows any user to deposit liquidity into a range of premiums 
/// in return for NFTs representing the positions
#[derive(Accounts)]
pub struct DepositLiquidity<'info>{
    /// Liquidity Provider is also the signer of the transaction
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    /// Pool to provide liquidity to
    pub pool: Account<'info,PoolAccount>
}
