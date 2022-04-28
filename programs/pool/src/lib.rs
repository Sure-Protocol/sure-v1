pub mod utils;
use anchor_lang::accounts::loader::Loader;
use anchor_lang::prelude::*;
use anchor_lang::context::Context;
use anchor_spl::token::{Mint,TokenAccount,Token};

use anchor_spl::mint::USDC;
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod sure_pool {
    use super::*;
    // ---------- Pool Management ---------------------------
    // Initialize Manager Owner
    // The pool owner is responsible for managing the pool.

    /// Initialize the pool manager
    ///
    /// # Arguments
    ///
    /// * ctx - initialize the manager
    ///
    pub fn initialize_pool_manager(ctx: Context<InitializePoolManager>) -> Result<()> {
        let pool_manager = &mut ctx.accounts.manager;
        pool_manager.owner = ctx.accounts.initial_manager.key();
        pool_manager.bump = *ctx.bumps.get("manager").unwrap();

        emit!(
            utils::events::InitializedManager{
                owner: ctx.accounts.initial_manager.key()
            }
        );
        Ok(())
    }

    // ------------ Pool -----------------------------------------------
    /// Create an insurance pool for a smart contract
    /// also create an associated vault to hold the tokens 
    /// 
    /// # Arguments 
    /// * ctx: 
    /// * insurance_fee: 
    pub fn create_pool(ctx: Context<CreatePool>,insurance_fee:i32,range_size:i32,name: String,smart_contract: Pubkey) -> Result<()> {

        let liquidity_token = &mut ctx.accounts.token;
        
        // Only allow for USDC 
        // Must be mocked in tests. 
        //require!(liquidity_token.key() == USDC.key(),utils::errors::SureError::InvalidMint);
        
        // Range size should be less than 100. Meaning that the premium should be less than 100%
        require!(range_size < 100*100 && range_size > 0,utils::errors::SureError::InvalidRangeSize);
        
        // Get pool account
        let pool_account = &mut ctx.accounts.pool;
       
        // Set up pool account 
        pool_account.bump = *ctx.bumps.get("pool").unwrap();
        pool_account.token = liquidity_token.key();
        pool_account.insurance_fee=insurance_fee;
        pool_account.range_size = range_size;
        pool_account.ranges = 0; 
        pool_account.liquidity = 0;
        pool_account.free_liquidity = 0;
        pool_account.name=name;
        pool_account.premium_rate = 0;
        pool_account.smart_contract = smart_contract.key();
        pool_account.locked=false;
        pool_account.vault = ctx.accounts.vault.key();

        emit!(
            utils::events::InitializedPool{
                name: "".to_string(),
                smart_contract: smart_contract.key()
            }
        );
        Ok(())
    }

    /// Deposit liquidity into a pool
    /// 
    /// # Arguments
    /// *ctx: 
    /// 
    pub fn deposit_liquidity(ctx:Context<DepositLiquidity>,amount:f64) -> Result<()>{

        Ok(())
    }
}

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
pub const POOL_MANAGER_SIZE: usize = 32 + 1;


#[derive(Accounts)]
pub struct InitializePoolManager<'info> {
    // Account for keeping track of the pool manager
    #[account(init,
        payer= initial_manager,
        space = 8 + POOL_MANAGER_SIZE,
        seeds = [b"sure-pool-manager"],
        bump 
    )]
    pub manager: Account<'info, PoolManager>,

    // The signer becomes the initial manager
    #[account(mut)]
    pub initial_manager: Signer<'info>,

    // System program
    pub system_program: Program<'info, System>,
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


pub const SURE_PRIMARY_POOL_SEED: &str = "sure-insurance-pool";
pub const SURE_ASSOCIATED_TOKEN_ACCOUNT_SEED: &str = "sure-ata";

#[derive(Accounts)]
#[instruction(smart_contract: Pubkey)]
pub struct CreatePool<'info> {
    #[account(
        init,
        space = 8 + PoolAccount::SPACE, 
        payer = pool_creator,
        seeds = [
            SURE_PRIMARY_POOL_SEED.as_bytes(),
            token.key().as_ref(),
            insured_token_account.key().to_bytes().as_ref(),
        ],
        bump
    )]
    pub pool: Account<'info, PoolAccount>,

    // Assume the contract being hacked is a token account
    /// CHECK: This accounts represents the executable contract 
    /// that is to be insured. 
    #[account(
        constraint = insured_token_account.executable == false
    )]
    pub insured_token_account: AccountInfo<'info>,

    // // Initialized vault to hold the pool token
    #[account(
        init,
        payer = pool_creator,
        seeds = [
            SURE_ASSOCIATED_TOKEN_ACCOUNT_SEED.as_bytes(), 
            pool.key().as_ref(),
            token.key().as_ref()
        ],
        bump,
        token::mint = token,
        token::authority = vault
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    // /// Pool creator
    #[account(mut)]
    pub pool_creator: Signer<'info>,

    /// Token to be deposited into the pool
    pub token: Account<'info,Mint>,

    /// Sysvar for Associated Token Account
    pub rent: Sysvar<'info, Rent>,

    // Token program
    pub token_program: Program<'info, Token>,

    /// Provide the system program
    pub system_program: Program<'info,System>,

}



/// Deposit Liquidity
/// allows any user to deposit liquidity into a range of premiums 
/// in return for NFTs representing the positions
///
#[derive(Accounts)]
pub struct DepositLiquidity<'info>{
    /// Liquidity Provider is also the signer of the transaction
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    /// Pool to provide liquidity to
    pub pool: Account<'info,PoolAccount>
}
