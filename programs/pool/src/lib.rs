pub mod context;
pub mod modules;
pub mod states;
pub mod utils;

use crate::states::bitmap::*;
use crate::states::tick;
use crate::states::tick::TickTrait;
use crate::states::liquidity::*;
use crate::states::contract::*;
use crate::states::*;
use crate::utils::errors::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::*;
use context::*;
use mpl_token_metadata::instruction::{create_metadata_accounts_v2, update_metadata_accounts_v2};
use mpl_token_metadata::state::Creator;
use std::cmp;
use vipers::prelude::*;

declare_id!("sureLJ8UXoy3WF3Dk6Hy1ak8DscZkmHvv1hprvhVCxB");

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
    pub fn initialize_protocol(ctx: Context<InitializeProtocol>) -> Result<()> {
        // load accounts
        let protocol_owner = &mut ctx.accounts.protocol_owner;
        let pools = &mut ctx.accounts.pools;

        protocol_owner.bump = unwrap_bump!(ctx, "protocol_owner");
        protocol_owner.owner = ctx.accounts.owner.key();

        // Initialize pools overview
        pools.bump = unwrap_bump!(ctx, "pools");
        pools.pools = Vec::new();

        emit!(owner::ChangeProtocolOwner {
            owner: ctx.accounts.owner.key(),
            old_owner: Pubkey::default(),
        });

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

        emit!(utils::events::InitializedManager {
            owner: ctx.accounts.initial_manager.key()
        });
        Ok(())
    }


    /// --- Initialize Policy Holder ----
    ///
    /// Prepare a new user for buying insurance 
    ///
    /// # Arguments
    ///
    /// * ctx - initialize the manager
    ///
    pub fn initialize_policy_holder(ctx: Context<InitializePolicyHolder>) -> Result<()> {
        let insurance_contracts = &mut ctx.accounts.insurance_contracts;
        insurance_contracts.owner = ctx.accounts.signer.key();
        insurance_contracts.bump = *ctx.bumps.get("insurance_contracts").unwrap();
        insurance_contracts.pools = Vec::new();
        
        emit!(InitializePolicyHolderEvent {
            owner: ctx.accounts.signer.key()
        });
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
    pub fn create_pool(
        ctx: Context<CreatePool>,
        insurance_fee: u16,
        name: String,
    ) -> Result<()> {
        // ________________ Validation ________________
        // Only allow the owner of the protocol to create pools
        // let protocol_owner = &ctx.accounts.protocol_owner.load()?;
        // require!(
        //     ctx.accounts.pool_creator.key() == protocol_owner.owner,
        //     SureError::InvalidPoolCreator
        // );

        // Load Accounts
        let pool = &mut ctx.accounts.pool;
        let sure_pools = &mut ctx.accounts.pools;
        
        let insured_smart_contract = ctx.accounts.smart_contract.key();
        // Range size should be less than 100. Meaning that the premium should be less than 100%

        // Set up pool account
        pool.bump = *ctx.bumps.get("pool").unwrap();
        pool.insurance_fee = insurance_fee;
        pool.name = name.clone();
        pool.smart_contract = insured_smart_contract.clone();
        pool.locked = false;
        pool.token_pools = Vec::new();

        // Update the pools
        sure_pools.pools.push(pool.key().clone());

        emit!(pool::CreatePool {
            name: name,
            smart_contract: ctx.accounts.smart_contract.key(),
            insurance_fee: insurance_fee
        });

        Ok(())
    }

    /// Remove pool
    /// The pool owner can remove the pool
    /// This results in
    /// - All insurance contracts being voided
    /// - All liquidity sent to the holder of the NFT
    ///
    // pub fn remove_pool() -> Result<()> {
    //     Ok(())
    // }

    /// Initialize Token Pool
    /// 
    /// Initialize
    /// - Liquidity Vault
    /// - Premium Vault
    /// - TokenPool
    /// 
    /// for the provided Pool.
    ///
    /// # Arguments
    /// * ctx:
    ///
    pub fn initialize_token_pool(ctx: Context<InitializeTokenPool>) -> Result<()> {
        // Load accounts
        let pool_liquidity_tick_bitmap = &mut ctx.accounts.pool_liquidity_tick_bitmap;
        let pool = &mut ctx.accounts.pool;
        let token_pool = &mut ctx.accounts.token_pool;

        // Initialize Pool Liquidity Tick Bitmap
        pool_liquidity_tick_bitmap.bump = *ctx.bumps.get("pool_liquidity_tick_bitmap").unwrap();
        pool_liquidity_tick_bitmap.spacing = 10;
        pool_liquidity_tick_bitmap.word = [0; 4];

        // Initialize Token Pool
        token_pool.bump = *ctx.bumps.get("token_pool").unwrap();
        token_pool.liquidity = 0;
        token_pool.used_liquidity = 0;
        token_pool.token_mint = ctx.accounts.pool_vault_token_mint.key();

        // Update pool with new tokenPool entry
        pool.token_pools.push(token_pool.key().clone());
        
        emit!(pool::InitializeTokenPool{});
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
    /// * liquidity_position_id: should be an id that is currently not in the tick pool
    pub fn deposit_liquidity(
        ctx: Context<DepositLiquidity>,
        tick: u16,
        tick_pos: u64,
        amount: u64,
    ) -> Result<()> {
        // ___________________ Validation ____________________________
        // #### Check input arguments
        // tick must be greater than 0 and less than 1,000 bp .
        require!(
            tick > 0 && tick < 1_000,
            utils::errors::SureError::InvalidTick
        );
        require!(amount > 0, utils::errors::SureError::InvalidAmount);

        // Load Accounts
        let pool_liquidity_tick_bitmap = &mut ctx.accounts.pool_liquidity_tick_bitmap;
        let pool = &mut ctx.accounts.pool;
        let token_pool = &mut ctx.accounts.token_pool;
        let protocol_owner = &ctx.accounts.protocol_owner;
        let pool_vault = &ctx.accounts.pool_vault;

        // The existence of a liquidity position should be checked by anchor.

        // _________________ Functionality _________________________

        let protocol_owner_seeds = [
            &SURE_PROTOCOL_OWNER.as_bytes() as &[u8],
            &[protocol_owner.bump],
        ];
        // # 1. Mint NFT to represent liquidity position
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone(),
                token::MintTo {
                    mint: ctx.accounts.liquidity_position_nft_mint.to_account_info().clone(),
                    to: ctx.accounts.liquidity_position_nft_account.to_account_info().clone(),
                    authority: ctx.accounts.protocol_owner.to_account_info().clone(),
                },
                &[&protocol_owner_seeds[..]],
            ),
            1,
        )?;

        // Add metadata to nft in order to represent position
        let create_metadata_accounts_ix = create_metadata_accounts_v2(
            ctx.accounts.metadata_program.key(),
            ctx.accounts.metadata_account.key(),
            ctx.accounts.liquidity_position_nft_mint.key(),
            ctx.accounts.protocol_owner.key(),
            ctx.accounts.liquidity_provider.key(),
            ctx.accounts.protocol_owner.key(),
            String::from("Sure LP NFT V1"),
            String::from("SURE-LP"),
            format!("https://sure.claims"),
            Some(vec![Creator {
                address: ctx.accounts.protocol_owner.key(),
                verified: true,
                share: 100,
            }]),
            0,
            true,
            true,
            None,
            None,
        );

        // Protocol owner signs the transaction with seeds
        // and bump
       
        solana_program::program::invoke_signed(
            &create_metadata_accounts_ix,
            &[
                ctx.accounts.metadata_account.to_account_info().clone(),
                ctx.accounts.liquidity_position_nft_mint.to_account_info().clone(),
                ctx.accounts.protocol_owner.to_account_info().clone(),
                ctx.accounts.liquidity_provider.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
            ],
            &[&protocol_owner_seeds[..]],
        )?;

        // # 2. Transfer tokens from liquidity provider account into vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                token::Transfer {
                    from: ctx
                        .accounts
                        .liquidity_provider_ata
                        .to_account_info()
                        .clone(),
                    to: pool_vault.to_account_info().clone(),
                    authority: ctx.accounts.liquidity_provider.to_account_info(),
                },
            ),
            amount,
        )?;

        // Load tick account state
        let liquidity_tick_info_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.liquidity_tick_info.to_account_info())?;
        let mut liquidity_tick_info = liquidity_tick_info_state.load_mut()?;

        let new_id = liquidity_tick_info.get_new_id();

        // # 3.  Save liqudity position
        let liquidity_position = &mut ctx.accounts.liquidity_position;
        liquidity_position.bump = *ctx.bumps.get("liquidity_position").unwrap();
        liquidity_position.liquidity = amount;
        liquidity_position.nft_account = ctx.accounts.liquidity_position_nft_account.key();
        liquidity_position.used_liquidity = 0;
        liquidity_position.pool = pool.key();
        liquidity_position.nft_mint = ctx.accounts.liquidity_position_nft_mint.key();
        liquidity_position.tick = tick;
        liquidity_position.created_at = Clock::get()?.unix_timestamp;
        liquidity_position.tick_id = new_id;
        liquidity_position.token_mint = pool_vault.mint;

        // If bitmap position is turned off, turn it on
        if !pool_liquidity_tick_bitmap.is_initialized(tick) {
            pool_liquidity_tick_bitmap.flip_bit(tick);
        }

        // Update Token Pool 
        token_pool.liquidity += amount;

        // # 4. Update tick with new liquidity position
        liquidity_tick_info
            .add_liquidity(new_id, amount, pool_vault.mint)
            .map_err(|e| e.to_anchor_error())?;
        //tick_account.update_callback()?;

        emit!(liquidity::NewLiquidityPosition {
            tick: tick,
            liquidity: amount
        });
        Ok(())
    }

    /// Update Rewards
    ///
    /// Crank for updating the rewards for each liquidity position in the
    /// tick liquidity
    ///
    /// # Arguments
    /// * ctx:
    ///
    //pub fn update_rewards_in_tick(ctx: )

    /// Redeem liquidity
    /// A holder can redeem liquidity that is not in use.
    /// position is used can redeem the unused liquidity.
    ///
    /// If some of the liquidity is active then it can only be withdrawn
    /// if there is free liquidity in the tick pool.
    ///
    /// # Arguments
    /// * ctx
    ///
    pub fn redeem_liquidity(ctx: Context<RedeemLiquidity>) -> Result<()> {
        // _______________ LOAD accounts __________________
        let liquidity_position = &ctx.accounts.liquidity_position;

        let tick_account_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.liquidity_tick_info.to_account_info())?;
        let mut liquidity_tick_info = tick_account_state.load_mut()?;
        let pool_liquidity_tick_bitmap = &mut ctx.accounts.pool_liquidity_tick_bitmap;
        let protocol_owner = &ctx.accounts.protocol_owner;
        let pool = &mut ctx.accounts.pool;
        let token_pool = &mut ctx.accounts.token_pool;

        // Available liquidity
        let free_liquidity = liquidity_tick_info.available_liquidity(liquidity_position.tick_id);
        require!(free_liquidity > 0, SureError::LiquidityFilled);

        // _______________ Functionality _______________

        let pool_seeds = [
            &SURE_PRIMARY_POOL_SEED.as_bytes() as &[u8],
            &pool.smart_contract.to_bytes() as &[u8],
            &[pool.bump],
        ];

        // # 1 Transfer excess liquidity back to nft holder
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone(),
                token::Transfer {
                    from: ctx.accounts.pool_vault.to_account_info().clone(),
                    to: ctx.accounts.liquidity_provider_ata.to_account_info().clone(),
                    authority: ctx.accounts.pool.to_account_info().clone(),
                },
                &[&pool_seeds[..]],
            ),
            free_liquidity,
        )?;

        // # 2. Update nft metadata to reflect token
        let updated_metaplex = mpl_token_metadata::state::DataV2 {
            name: String::from("Sure LP NFT V1"),
            symbol: String::from("SURE-LP"),
            uri: format!("https://sure.claims"),
            seller_fee_basis_points: 0,
            creators: Some(vec![Creator {
                address: ctx.accounts.protocol_owner.key(),
                verified: true,
                share: 100,
            }]),
            collection: None,
            uses: None,
        };
        let update_metaplex_metadata_ix = update_metadata_accounts_v2(
            ctx.accounts.metadata_program.key(),
            ctx.accounts.metadata_account.key(),
            ctx.accounts.protocol_owner.key(),
            None,
            Some(updated_metaplex),
            None,
            None,
        );

        solana_program::program::invoke_signed(
            &update_metaplex_metadata_ix,
            &[
                ctx.accounts.metadata_account.to_account_info().clone(),
                ctx.accounts.protocol_owner.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            &[&[&[protocol_owner.bump]]],
        )?;

        // Update Token Pool
        token_pool.liquidity -= free_liquidity;

        // # 3 Update tick poo
        liquidity_tick_info
            .remove_liquidity(liquidity_position.tick_id)
            .map_err(|e| e.to_anchor_error())?;

        if !liquidity_tick_info.active{
            if pool_liquidity_tick_bitmap.is_initialized(liquidity_position.tick) {
                pool_liquidity_tick_bitmap.flip_bit(liquidity_position.tick);
            }
        }
        Ok(())
    }


    /// --- Initialize: User Pool Insurance Contract ---
    /// 
    /// Creates a new insurance contract for a user for the given pool
    /// 
    /// Initializes 
    ///     - insurance_pool_contract_bitmap: accumulative insurance contract information
    ///     - insurance_pool_contract_info:   keeps tracks of ticks used by the user insurance contract
    /// 
    /// # Arguments
    /// * ctx
    ///
    pub fn initialize_user_pool_insurance_contract(
        ctx: Context<InitializeUserPoolInsuranceContract>,
    ) -> Result<()> {
        
        // Load accounts
        let pool_insurance_contract_bitmap = &mut ctx.accounts.pool_insurance_contract_bitmap;
        let pool_insurance_contract_info = &mut ctx.accounts.pool_insurance_contract_info;
        let pool = &mut ctx.accounts.pool;
        let insurance_contracts = &mut ctx.accounts.insurance_contracts;

        let current_time = Clock::get()?.unix_timestamp;
       
        // Initialize the insurance contract overview
        pool_insurance_contract_info.bump = unwrap_bump!(ctx, "pool_insurance_contract_info");
        pool_insurance_contract_info.expiry_ts = current_time;
        pool_insurance_contract_info.insured_amount=0;
        pool_insurance_contract_info.owner = ctx.accounts.signer.key();

        pool_insurance_contract_bitmap.bump = unwrap_bump!(ctx, "pool_insurance_contract_bitmap");
        pool_insurance_contract_bitmap.spacing = 10;
        pool_insurance_contract_bitmap.word = [0; 4];

        // update Insurance contracts
        insurance_contracts.pools.push(pool.key().clone());

        Ok(())
    }
    
    /// --- Initialize Insurance Contract --- 
    ///
    ///  Let a user create an insurance contract with a tick account
    /// in a Sure pool. 
    /// 
    /// Initializes: 
    ///     - insurance_tick_contract: holds information about the insurance for a user at the given tick
    ///
    /// # Arguments
    /// * ctx: Contains the pool, insurance contract and signer
    ///
    pub fn initialize_insurance_contract(ctx: Context<InitializeInsuranceContract>) -> Result<()> {
        
        // Load Accounts
        let insurance_tick_contract = &mut ctx.accounts.insurance_tick_contract;
        let pool_insurance_contract_bitmap = &mut ctx.accounts.pool_insurance_contract_bitmap;
        
        let liquidity_tick_info_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.liquidity_tick_info.to_account_info())?;
        let liquidity_tick_info = liquidity_tick_info_state.load()?;
        
        // Method variables 
        let current_time = Clock::get()?.unix_timestamp;
        
        // Initialize insurance_contract
        insurance_tick_contract.insured_amount = 0;
        insurance_tick_contract.premium = 0;
        insurance_tick_contract.bump = *ctx.bumps.get("insurance_tick_contract").unwrap();
        insurance_tick_contract.pool = ctx.accounts.pool.key();
        insurance_tick_contract.liquidity_tick_info = ctx.accounts.liquidity_tick_info.key();
        insurance_tick_contract.token_mint = ctx.accounts.token_mint.key();
        insurance_tick_contract.active = false;
        insurance_tick_contract.end_ts = current_time;
        insurance_tick_contract.created_ts =current_time;
        insurance_tick_contract.start_ts = current_time; 

        // Update insurance contract
        // Mark the position as filled
        if !pool_insurance_contract_bitmap.is_initialized(liquidity_tick_info.tick) {
            pool_insurance_contract_bitmap.flip_bit(liquidity_tick_info.tick);
        }
        

        Ok(())
    }

   

    /// --- Update Insurance Tick Contract ---
    /// 
    /// Updates the insurance contract for the given tick and the pool contract information
    /// and bitmap.
    /// 
    /// Initializes:
    ///     <nothing>
    ///
    /// TODO: Allow for unlocking of insured amount
    /// 
    /// # Arguments
    /// * ctx
    /// * new_insured_amount_on_tick: Final insurance amount for tick
    /// * new_expiry_ts: expiry of the contract in timestamp 
    ///
    pub fn update_insurance_tick_contract(
        ctx: Context<UpdateInsuranceTickContract>,
        new_insured_amount_on_tick: u64,
        new_expiry_ts: i64,
    ) -> Result<()> {

        // Load accounts
        let liquidity_tick_info_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.liquidity_tick_info.to_account_info())?;
        let mut liquidity_tick_info = liquidity_tick_info_state.load_mut()?;
        
        let pool = &mut ctx.accounts.pool;
        let token_pool = &mut ctx.accounts.token_pool;
        let insurance_tick_contract = &mut ctx.accounts.insurance_tick_contract;
        let pool_insurance_contract_info = &mut ctx.accounts.pool_insurance_contract_info;
        let liquidity_tick_bitmap = &mut ctx.accounts.liquidity_tick_bitmap;

        // Calculate coverage amount
        let current_insured_amount_on_tick = insurance_tick_contract.insured_amount;
        let amount_diff = if new_insured_amount_on_tick > current_insured_amount_on_tick {new_insured_amount_on_tick-current_insured_amount_on_tick} else {current_insured_amount_on_tick-new_insured_amount_on_tick};

        // Calculate the premium that has to be refunded or paid
        let (increase_premium,premium) = insurance_tick_contract.update_position_and_get_premium(liquidity_tick_info.tick, new_insured_amount_on_tick, new_expiry_ts)?;
        
        
        if new_insured_amount_on_tick > current_insured_amount_on_tick{
            liquidity_tick_info
            .buy_insurance(amount_diff)
            .map_err(|e| e.to_anchor_error())?;

            if liquidity_tick_info.is_pool_full(){
                liquidity_tick_bitmap.flip_bit(liquidity_tick_info.tick);
            }
            
            // Increase total insured amount for insurance pool contract
            pool_insurance_contract_info.insured_amount += amount_diff;
            
            token_pool.used_liquidity += amount_diff;
            pool_insurance_contract_info.expiry_ts = new_expiry_ts
        }else{
            liquidity_tick_info
            .exit_insurance(amount_diff)
            .map_err(|e| e.to_anchor_error())?;
            
            if !liquidity_tick_info.is_pool_full(){
                if !liquidity_tick_bitmap.is_initialized(liquidity_tick_info.tick) {
                    liquidity_tick_bitmap.flip_bit(liquidity_tick_info.tick);
                }
            }

            // Reduce total insured amount for pool
            pool_insurance_contract_info.insured_amount -= amount_diff;
            token_pool.used_liquidity -= amount_diff;
            pool_insurance_contract_info.expiry_ts = new_expiry_ts
        }
        
        if increase_premium {
            token::transfer(
                CpiContext::new(
                    ctx.accounts.token_program.to_account_info().clone(),
                    token::Transfer {
                        from: ctx.accounts.token_account.to_account_info().clone(),
                        to: ctx.accounts.premium_vault.to_account_info().clone(),
                        authority: ctx.accounts.buyer.to_account_info().clone(),
                    },
                ),
                premium,
            )?;
        }else{
            let pool_seed = [
                &SURE_PRIMARY_POOL_SEED.as_bytes() as &[u8],
                &pool.smart_contract.to_bytes() as &[u8],
                &[pool.bump],
            ];
    
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info().clone(),
                    token::Transfer {
                        from: ctx.accounts.premium_vault.to_account_info().clone(),
                        to: ctx.accounts.token_account.to_account_info().clone(),
                        authority: ctx.accounts.pool.to_account_info().clone(),
                    },
                    &[&pool_seed[..]],
                ),
                premium,
            )?;
        }
        Ok(())
    }


    /// -.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.--.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.--.-.-.
    /// -.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.- TICK -.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-
    /// -.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.--.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.--.-.-.

    /// --- Initialize Tick Account ---
    /// 
    /// 
    /// Initializes:
    ///     - tick_account: holds info about the liquidity for the given tick
    ///
    ///  # Argument
    /// * ctx:
    /// 
    pub fn initialize_pool_liquidity_tick(
        ctx: Context<InitializeTick>,
        _pool: Pubkey,
        _token: Pubkey,
        tick_bp: u16,
    ) -> Result<()> {
        let mut liquidity_tick_info = ctx.accounts.liquidity_tick_info.load_init()?;

        // Initialize account
        liquidity_tick_info.initialize(*ctx.bumps.get("liquidity_tick_info").unwrap(), tick_bp)?;
        Ok(())
    }

    /// --- Close Tick Account ---
    /// 
    /// Closes tick account if there is no more liquidity in the account
    /// and transfers the rent back 
    ///
    /// # Arguments
    /// * ctx
    ///
    pub fn close_pool_liquidity_tick(ctx: Context<CloseTick>) -> Result<()> {
        let liquidity_tick_info_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.liquidity_tick_info.to_account_info())?;
        let liquidity_tick_info = liquidity_tick_info_state.load_mut()?;

        if !liquidity_tick_info.is_pool_empty() {
            return Err(error!(SureError::TickAccountNotEmpty));
        }
        Ok(())
    }
}
