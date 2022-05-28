pub mod context;
pub mod modules;
pub mod states;
pub mod utils;

use crate::states::bitmap::*;
use crate::states::tick;
use crate::states::tick::TickTrait;
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

declare_id!("79zTSyMWgBpHQmkWmdiRH4Jz727UJzpUW5rEutseyMrP");

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
        let protocol_owner = &mut ctx.accounts.protocol_owner.load_init()?;
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
        let pool_account = &mut ctx.accounts.pool;
        let sure_pools = &mut ctx.accounts.sure_pools;

        let insured_token_account = ctx.accounts.insured_token_account.key();
        // Range size should be less than 100. Meaning that the premium should be less than 100%

        // Set up pool account
        pool_account.bump = *ctx.bumps.get("pool").unwrap();
        pool_account.insurance_fee = insurance_fee;
        pool_account.liquidity = 0;
        pool_account.used_liquidity = 0;
        pool_account.name = name;
        pool_account.premium_rate = 0;
        pool_account.smart_contract = insured_token_account.clone();
        pool_account.locked = false;

        // Update the pools
        sure_pools.pools.push(insured_token_account.clone());

        emit!(pool::InitializedPool {
            name: "".to_string(),
            smart_contract: ctx.accounts.insured_token_account.key()
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

    /// Create Token Pool Vaults
    /// Create and initialize
    /// - Liquidity Vault
    /// - Premium Vault
    /// for the provided Pool.
    ///
    /// # Arguments
    /// * ctx:
    ///
    pub fn create_pool_vaults(ctx: Context<CreatePoolVaults>) -> Result<()> {
        // Load accounts
        let bitmap = &mut ctx.accounts.bitmap;

        // Initialize Bitmap
        bitmap.bump = *ctx.bumps.get("bitmap").unwrap();
        bitmap.spacing = 10;
        bitmap.word = [0; 4];

        emit!(pool::CreatePoolVaults {});
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
        let bitmap = &mut ctx.accounts.bitmap;
        let pool = &mut ctx.accounts.pool;
        let protocol_owner = &ctx.accounts.protocol_owner.load()?;
        let vault = &ctx.accounts.vault;

        // The existence of a liquidity position should be checked by anchor.

        // _________________ Functionality _________________________

        // # 1. Mint NFT to represent liquidity position
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info().clone(),
                token::MintTo {
                    mint: ctx.accounts.nft_mint.to_account_info().clone(),
                    to: ctx.accounts.nft_account.to_account_info().clone(),
                    authority: ctx.accounts.protocol_owner.to_account_info().clone(),
                },
                &[&[&[ctx.accounts.protocol_owner.load()?.bump] as &[u8]]],
            ),
            1,
        )?;

        // Add metadata to nft in order to represent position
        let create_metadata_accounts_ix = create_metadata_accounts_v2(
            /// TODO: Find correct metadata id
            ctx.accounts.metadata_program.key(),
            ctx.accounts.metadata_account.key(),
            ctx.accounts.nft_mint.key(),
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
                ctx.accounts.nft_mint.to_account_info().clone(),
                ctx.accounts.protocol_owner.to_account_info().clone(),
                ctx.accounts.liquidity_provider.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
                ctx.accounts.rent.to_account_info().clone(),
            ],
            &[&[&[protocol_owner.bump]]],
        )?;

        // # 2. Transfer tokens from liquidity provider account into vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                token::Transfer {
                    from: ctx
                        .accounts
                        .liquidity_provider_account
                        .to_account_info()
                        .clone(),
                    to: vault.to_account_info().clone(),
                    authority: ctx.accounts.liquidity_provider.to_account_info(),
                },
            ),
            amount,
        )?;

        // Update pool
        pool.liquidity += amount;

        // Load tick account state
        let tick_account_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.tick_account.to_account_info())?;
        let mut tick_account = tick_account_state.load_mut()?;

        let new_id = tick_account.get_new_id();

        // # 3.  Save liqudity position
        let liquidity_position = &mut ctx.accounts.liquidity_position;
        liquidity_position.bump = *ctx.bumps.get("liquidity_position").unwrap();
        liquidity_position.liquidity = amount;
        liquidity_position.nft_account = ctx.accounts.nft_account.key();
        liquidity_position.used_liquidity = 0;
        liquidity_position.pool = pool.key();
        liquidity_position.nft_mint = ctx.accounts.nft_mint.key();
        liquidity_position.tick = tick;
        liquidity_position.created_at = Clock::get()?.unix_timestamp;
        liquidity_position.tick_id = new_id;
        liquidity_position.token_mint = vault.mint;

        // Update bitmap
        if !bitmap.is_initialized(tick) {
            bitmap.flip_bit(tick);
        }

        // # 4. Update tick with new liquidity position
        tick_account
            .add_liquidity(new_id, amount, vault.mint)
            .map_err(|e| e.to_anchor_error())?;
        //tick_account.update_callback()?;

        emit!(liquidity::NewLiquidityPosition {
            tick: tick,
            liquidity: amount
        });
        //require!(true == false, SureError::InvalidAmount);
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

        let vault = &ctx.accounts.vault;

        let liquidity_position = &ctx.accounts.liquidity_position;

        let tick_account_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.tick_account.to_account_info())?;
        let mut tick_account = tick_account_state.load_mut()?;

        let protocol_owner = &ctx.accounts.protocol_owner.load()?;
        let pool = &ctx.accounts.pool;

        /// Available liquidity
        let free_liquidity = tick_account.available_liquidity(liquidity_position.tick_id);
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
                    from: ctx.accounts.vault.to_account_info().clone(),
                    to: ctx.accounts.token_account.to_account_info().clone(),
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

        // # 3 Update tick poo
        tick_account
            .remove_liquidity(liquidity_position.tick_id)
            .map_err(|e| e.to_anchor_error())?;
        Ok(())
    }

    /// Initialize Insurance Contract
    /// Let a user create an insurance contract with a Sure pool
    ///
    /// # Arguments
    /// * ctx: Contains the pool, insurance contract and signer
    ///
    pub fn initialize_insurance_contract(ctx: Context<InitializeInsuranceContract>) -> Result<()> {
        // Load insurance_contract
        let insurance_contract = &mut ctx.accounts.insurance_contract;
        let insurance_contracts = &mut ctx.accounts.insurance_contracts;
        let tick_account_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.tick_account.to_account_info())?;
        let tick_account = tick_account_state.load()?;
        let current_time = Clock::get()?.unix_timestamp;
        // Initialize insurance_contract
        insurance_contract.insured_amount = 0;
        insurance_contract.premium = 0;
        insurance_contract.bump = *ctx.bumps.get("insurance_contract").unwrap();
        insurance_contract.pool = ctx.accounts.pool.key();
        insurance_contract.tick_account = ctx.accounts.tick_account.key();
        insurance_contract.token_mint = ctx.accounts.token_mint.key();
        insurance_contract.active = false;
        insurance_contract.end_ts = current_time;
        insurance_contract.created_ts =current_time;
        insurance_contract.start_ts = current_time; 
        insurance_contract.owner = ctx.accounts.owner.key();

        // Update insurance contract
        // Mark the position as filled
        if !insurance_contracts.is_initialized(tick_account.tick) {
            insurance_contracts.flip_bit(tick_account.tick);
        }

        Ok(())
    }

    pub fn initialize_user_insurance_contracts(
        ctx: Context<InitializeUserInsuranceContracts>,
    ) -> Result<()> {
        // Load accounts
        let insurance_contracts = &mut ctx.accounts.insurance_contracts;

        // Initialize the insurance contract overview
        insurance_contracts.bump = unwrap_bump!(ctx, "insurance_contracts");
        insurance_contracts.spacing = 10;
        insurance_contracts.word = [0; 4];

        Ok(())
    }

    /// Buy insurance for tick
    /// A buyer should select an amount to insure and the smart contract should
    /// premium
    ///
    /// NOTE: Protocol fee will be subtracted continously from the premium vaults
    /// TODO: Rename method to adjust_contract_position
    /// TODO: Allow for unlocking of insured amount
    /// 
    /// # Arguments
    /// * ctx
    /// * Amount: the amount the user want to insure
    /// * Until ts: The timestamp for the end of the contract
    ///
    pub fn buy_insurance_for_tick(
        ctx: Context<BuyInsurance>,
        insured_amount: u64,
        end_ts: i64,
    ) -> Result<()> {
        // Load accounts
        let tick_account_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.tick_account.to_account_info())?;
        let mut tick_account = tick_account_state.load_mut()?;
        let pool_account = &mut ctx.accounts.pool;
        let insurance_contract = &mut ctx.accounts.insurance_contract;

        // Calculate coverage amount
        let current_insured_amount = insurance_contract.insured_amount;
        let amount_diff = if insured_amount > current_insured_amount {insured_amount-current_insured_amount}else {current_insured_amount-insured_amount};


        let (is_increase_premium,premium) = insurance_contract.update_position_and_get_premium(tick_account.tick, insured_amount, end_ts)?;
        if insured_amount > current_insured_amount{
            tick_account
            .buy_insurance(amount_diff)
            .map_err(|e| e.to_anchor_error())?;
            pool_account.used_liquidity += amount_diff;
        }else{
            tick_account
            .exit_insurance(amount_diff)
            .map_err(|e| e.to_anchor_error())?;
            pool_account.used_liquidity -= amount_diff;
        }
        
        if is_increase_premium {
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
                &pool_account.smart_contract.to_bytes() as &[u8],
                &[pool_account.bump],
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

    /// Initialize Tick
    /// If there has never been provided liquidity at a position then a
    /// new tick have to be created.
    ///
    /// Will called when depositing liquidity
    ///
    ///  # Argument
    /// * ctx:
    pub fn initialize_tick(
        ctx: Context<InitializeTick>,
        _pool: Pubkey,
        _token: Pubkey,
        tick_bp: u16,
    ) -> Result<()> {
        let mut tick_account = ctx.accounts.tick_account.load_init()?;

        // Initialize account
        tick_account.initialize(*ctx.bumps.get("tick_account").unwrap(), tick_bp)?;
        Ok(())
    }

    /// Close tick
    /// closes tick account if there is no more liquidity in the account
    ///
    /// # Arguments
    /// * ctx
    ///
    pub fn close_tick(ctx: Context<CloseTick>) -> Result<()> {
        let tick_account_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.tick_account.to_account_info())?;
        let tick_account = tick_account_state.load_mut()?;

        if !tick_account.is_pool_empty() {
            return Err(error!(SureError::TickAccountNotEmpty));
        }
        Ok(())
    }
}
