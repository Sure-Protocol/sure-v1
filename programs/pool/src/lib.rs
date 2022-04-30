pub mod utils;
pub mod context;
pub mod states;
use context::*;

use anchor_lang::prelude::*;
use anchor_lang::context::Context;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod sure_pool {

    use super::*;
    // ---------- Pool Management ---------------------------
    // Initialize Manager Owner
    // The pool owner is responsible for managing the pool.

    /// Initialize the pool manager
    /// NOTE: might not need a dedicated pool manager
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
    /// * insurance_fee: fee taken on each insurance bought. In basis points (1bp = 0.01%)
    /// * range_size: The size of the ranges in which users can provide insurance
    /// * name: [optional] Name of the pool
    pub fn create_pool(ctx: Context<CreatePool>,insurance_fee:i32,range_size:i32,name: String) -> Result<()> {

        let liquidity_token = &mut ctx.accounts.token;
        
        // ________________ Validation ________________

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
        pool_account.active_liquidity = 0;
        pool_account.name=name;
        pool_account.premium_rate = 0;
        pool_account.smart_contract = ctx.accounts.insured_token_account.key();
        pool_account.locked=false;
        pool_account.vault = ctx.accounts.vault.key();

        emit!(
            utils::events::InitializedPool{
                name: "".to_string(),
                smart_contract: ctx.accounts.insured_token_account.key()
            }
        );
        Ok(())
    }

    /// Deposit liquidity into pool
    /// Let any user deposit tokens into the vault associated 
    /// with the given pool
    /// 
    /// # Argument
    /// * ctx: 
    /// 
    pub fn deposit_liquidity(ctx:Context<DepositLiquidity>,amount: u64) -> Result<()> {
        Ok(())
    }
}

