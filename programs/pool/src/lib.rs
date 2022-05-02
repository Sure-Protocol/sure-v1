pub mod utils;
pub mod context;
pub mod states;
pub mod modules;
use context::*;
use crate::states::*;
use crate::utils::errors::*;

use anchor_lang::prelude::*;
use anchor_spl::*;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod sure_pool {
    use super::*;

    // ---------- Sure Protocol Management ------------------
    // Everything regarding the management of the Sure protocol

    /// Initialize protocol
    /// Set protocol owner and other metadata necessary to 
    /// initialize the protocol.__rust_force_expr!
    /// 
    /// # Arguments
    /// 
    /// * ctx: 
    /// 
    pub fn initialize_protocol(ctx: Context<Initialize>) -> Result<()> {
        let protocol_owner = &mut ctx.accounts.protocol_owner.load_init()?;
        protocol_owner.bump = *ctx.bumps.get("protocol_owner").unwrap();
        protocol_owner.owner = ctx.accounts.owner.key();


        emit!(
            owner::ChangeProtocolOwner{
                owner: ctx.accounts.owner.key(),
                old_owner: Pubkey::default(),
            }
        );

        Ok(())
    }

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
    /// * tick (bp): Tick to provide liquidity at 
    /// * amount: Amount of liquidity to place at given tick
    pub fn deposit_liquidity(ctx:Context<DepositLiquidity>,tick: u32, amount: u64) -> Result<()> {
         
        // ___________________ Validation ____________________________
        // #### Check input arguments

        // tick must be greater than 0 and less than 100 .
        require!(tick > 0 && tick < 100,utils::errors::SureError::InvalidTick);

        require!(amount > 0, utils::errors::SureError::InvalidAmount);

        // Check that the correct vault is provided
        let pool_vault_pb = &ctx.accounts.pool.vault;
        let token_vault = &ctx.accounts.token_vault.to_account_info();
        require!(pool_vault_pb.key() == token_vault.key(),utils::errors::SureError::InvalidMint);

        // The existence of a liquidity position should be checked by anchor. 



        // _________________ Functionality _________________________

        // # Mint NFT to represent liquidity position
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone()
                , token::MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info().clone(),
                    to: ctx.accounts.nft_account.to_account_info().clone(),
                    authority: ctx.accounts.protocol_owner.to_account_info().clone()
                },
                &[&[&[ctx.accounts.protocol_owner.load()?.bump] as &[u8]]])
            , 1)?;

        // # Transfer tokens from liquidity provider account into vault 
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                 token::Transfer{
                     from: ctx.accounts.liquidity_provider_account.to_account_info().clone(),
                     to:   ctx.accounts.token_vault.to_account_info().clone(),
                     authority: ctx.accounts.liquidity_provider.to_account_info(),
                 }
                ),
        amount)?;

        // TODO Add metaplex data to the NFT mint. 


        // # Save state in Sure 
        let liquidity_position = &mut ctx.accounts.liquidity_position;
        liquidity_position.bump = *ctx.bumps.get("liquidity_position").unwrap();
        liquidity_position.liquidity = amount;
        liquidity_position.token_mint = ctx.accounts.token_program.key();
        liquidity_position.used_liquidity = 0;
        liquidity_position.pool = ctx.accounts.pool.key();
        liquidity_position.nft_mint = ctx.accounts.nft_mint.key();
        liquidity_position.tick = tick;
        liquidity_position.created_at = Clock::get()?.unix_timestamp;

        // Update Liquidity Pool
        let liquidity_pool = &mut ctx.accounts.pool;
        liquidity_pool.liquidity += amount;

        emit!(
           pool::NewLiquidityPosition{
                tick: tick,
                liquidity: amount
            }
        );

        Ok(())
    }

    /// Redeem liquidity
    /// Holders of the LP NFT can burn it in return for liquidity 
    /// However, it takes about 5 days to extract the liquidity
    /// so that there isn't a draw on liquidity. 
    /// 
    /// If some of the liquidity is active then it can only be withdrawn 
    /// if there is free liquidity in the tick pool. 
    /// 
    /// # Arguments
    /// * ctx
    ///
    pub fn redeem_liquidity(ctx: Context<RedeemLiquidity>) -> Result<()> {

        // _______________ Validation __________________
        // * Check that the 
        let liquidity_position = &ctx.accounts.liquidity_position;
        require!(liquidity_position.used_liquidity < liquidity_position.liquidity,SureError::LiquidityFilled);



        // _______________ Functionality _______________
        // # 1. Find the available liquidity
        let available_liquidity = liquidity_position.liquidity - liquidity_position.used_liquidity;

        // # 2. Burn nft 
        token::burn(CpiContext::new(
            ctx.accounts.token_account.to_account_info().clone(), 
            token::Burn{
                mint: ctx.accounts.nft_mint.to_account_info().clone(),
                from: ctx.accounts.nft.to_account_info().clone(),
                authority: ctx.accounts.nft_holder.to_account_info().clone()
                }), 1)?;
        
        // # 3 Transfer liquidity back to nft holder
        token::transfer(
            CpiContext::new_with_signer(ctx.accounts.token_account.to_account_info().clone(), token::Transfer{
                from: ctx.accounts.vault_account.to_account_info().clone(),
                to: ctx.accounts.nft_holder.to_account_info().clone(),
                authority: ctx.accounts.vault_account.to_account_info().clone(),
            }, &[&[&[ctx.accounts.protocol_owner.load()?.bump] as &[u8]]])
            , available_liquidity)?;

        // # 4 create new liquidity position based on remaining liquidity 
        // Need custom method for creating liquidity positon and mint NFT 

        Ok(())
    }


    /// Buy insurance 
    /// A buyer should be able to easily buy insurance buy paying a yearly 
    /// premium
    /// 
    /// 
    /// # Arguments
    /// * ctx
    /// 
    pub fn buy_insurance(ctx: Context<BuyInsurance>) -> Result<()> {

         // _______________ Validation __________________
        // * Check that the 

        // _______________ Functionality _______________
        // # 1. 
        Ok(())
    }

    /// Initialize Tick
    /// If there has never been provided liquidity at a position then a 
    /// new tick have to be created. 
    /// 
    /// Will called when depositing liquidity 
    /// 
    ///  # Argument
    /// * ctx: 
    pub fn initialize_tick(ctx: Context<InitializeTick>) -> Result<()> {
    
        Ok(())

    }
}

