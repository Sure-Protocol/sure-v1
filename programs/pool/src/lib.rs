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
use vipers::prelude::*;
use mpl_token_metadata::instruction::{create_metadata_accounts_v2,update_metadata_accounts_v2};
use mpl_token_metadata::state::Creator;

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
        protocol_owner.bump = unwrap_bump!(ctx, "protocol_owner");
        protocol_owner.owner = ctx.accounts.owner.key();

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
        tick_spacing: u16,
        name: String,
    ) -> Result<()> {
        let liquidity_token = &mut ctx.accounts.token;

        // ________________ Validation ________________
        // Only allow the owner of the protocol to create pools
        let protocol_owner = &ctx.accounts.protocol_owner.load()?;
        require!(
            ctx.accounts.pool_creator.key() == protocol_owner.owner,
            SureError::InvalidPoolCreator
        );

        // Only allow for USDC
        // Must be mocked in tests.
        //require!(liquidity_token.key() == USDC.key(),utils::errors::SureError::InvalidMint);

        // Range size should be less than 100. Meaning that the premium should be less than 100%
        require!(
            tick_spacing < 100 * 100 && tick_spacing > 0,
            utils::errors::SureError::InvalidRangeSize
        );

        // Bitmap
        let bitmap = &mut ctx.accounts.bitmap;
        bitmap.bump = *ctx.bumps.get("bitmap").unwrap();
        bitmap.spacing = 10;

        // Get pool account
        let pool_account = &mut ctx.accounts.pool;

        // Set up pool account
        pool_account.bump = *ctx.bumps.get("pool").unwrap();
        pool_account.token = liquidity_token.key();
        pool_account.insurance_fee = insurance_fee;
        pool_account.tick_spacing = tick_spacing;
        pool_account.liquidity = 0;
        pool_account.active_liquidity = 0;
        pool_account.name = name;
        pool_account.premium_rate = 0;
        pool_account.smart_contract = ctx.accounts.insured_token_account.key();
        pool_account.locked = false;
        pool_account.vault = ctx.accounts.vault.key();
        pool_account.bitmap = bitmap.key();

        emit!(utils::events::InitializedPool {
            name: "".to_string(),
            smart_contract: ctx.accounts.insured_token_account.key()
        });

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

        // Check that the correct vault is provided
        let bitmap = &mut ctx.accounts.bitmap;
        let pool = &mut ctx.accounts.pool;
        let pool_vault_pb =pool.vault; 
        let protocol_owner = &ctx.accounts.protocol_owner.load()?;
        let vault = &ctx.accounts.vault.to_account_info();
        require!(
            pool_vault_pb.key() == vault.key(),
            utils::errors::SureError::InvalidMint
        );

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
            Some(vec![
                Creator{
                    address: ctx.accounts.protocol_owner.key(),
                    verified: true,
                    share: 100,
                }
            ]),
            0,
            true,
            true,
            None,
            None
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

            ], &[&[&[protocol_owner.bump]]])?;


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
                    to: ctx.accounts.vault.to_account_info().clone(),
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
        liquidity_position.token_mint = pool.token;

        // Update bitmap
        if !bitmap.is_initialized(tick, pool.tick_spacing.clone()){
            bitmap.flip_bit(tick);
        }
       

        // # 4. Update tick with new liquidity position
        tick_account
            .add_liquidity(new_id, amount)
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
    pub fn redeem_liquidity(ctx: Context<RedeemLiquidity>,token_mint:Pubkey,insured_token_account: Pubkey) -> Result<()> {
        // _______________ LOAD accounts __________________
    

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
            &token_mint.to_bytes() as &[u8],
            &insured_token_account.to_bytes() as &[u8],
            &[pool.bump]
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
            uri:format!("https://sure.claims"),
            seller_fee_basis_points: 0,
            creators: Some(vec![
                Creator{
                    address: ctx.accounts.protocol_owner.key(),
                    verified: true,
                    share: 100,
                }
            ]),
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

            ], &[&[&[protocol_owner.bump]]])?;

        // # 3 Update tick poo
        tick_account.remove_liquidity(liquidity_position.tick_id).map_err(|e| e.to_anchor_error())?;
        Ok(())
    }

    /// Initialize Insurance Contract
    /// Let a user create an insurance contract with a Sure pool
    /// 
    /// # Arguments
    /// * ctx: Contains the pool, insurance contract and signer
    /// 
    pub fn initialize_insurance_contract(
        ctx: Context<InitializeInsuranceContract>,
    ) -> Result<()> {
        // TODO: Add period as a seed and variable

        // Load insurance_contract
        let insurance_contract = &mut ctx.accounts.insurance_contract;

        // Initialize insurance_contract
        insurance_contract.amount = 0;
        insurance_contract.bump = *ctx.bumps.get("insurance_contract").unwrap();
        insurance_contract.pool = ctx.accounts.pool.key();
        insurance_contract.active = true;
        insurance_contract.owner = ctx.accounts.owner.key();

        Ok(())
    }

    /// Buy insurance for tick
    /// A buyer should select an amount to insure and the smart contract should
    /// premium
    ///
    ///
    /// # Arguments
    /// * ctx
    ///
    pub fn buy_insurance_for_tick(
        ctx: Context<BuyInsurance>,
        amount: u64,
    ) -> Result<()> {
        // _______________ Validation __________________
        // * Check that the

        // _______________ Functionality _______________

        // Load accounts
        let tick_account_state =
        AccountLoader::<tick::Tick>::try_from(&ctx.accounts.tick_account.to_account_info())?;
        let mut tick_account = tick_account_state.load_mut()?;

        let pool_account = &mut ctx.accounts.pool;

        // #1 credit tick account
        tick_account.buy_insurance(amount).map_err(|e| e.to_anchor_error())?;

        // #2 Update pool account
        pool_account.active_liquidity -= amount;

        // #3 transfer premium to premium vault 


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
        tick_bp: u64,
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
