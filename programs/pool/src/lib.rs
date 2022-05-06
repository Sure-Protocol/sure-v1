pub mod context;
pub mod modules;
pub mod states;
pub mod utils;

use crate::states::bitmap::*;
use crate::states::tick;
use crate::states::*;
use crate::utils::errors::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::*;
use context::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod sure_pool {

    use crate::states::tick::TickTrait;

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
        amount: u64,
        liquidity_position_id: u8,
    ) -> Result<()> {
        // ___________________ Validation ____________________________
        // #### Check input arguments

        // tick must be greater than 0 and less than 100 .
        require!(
            tick > 0 && tick < 100,
            utils::errors::SureError::InvalidTick
        );
        require!(amount > 0, utils::errors::SureError::InvalidAmount);

        // Check that the correct vault is provided
        let pool_vault_pb = &ctx.accounts.pool.vault;
        let token_vault = &ctx.accounts.token_vault.to_account_info();
        require!(
            pool_vault_pb.key() == token_vault.key(),
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
                    to: ctx.accounts.token_vault.to_account_info().clone(),
                    authority: ctx.accounts.liquidity_provider.to_account_info(),
                },
            ),
            amount,
        )?;

        // TODO Add metaplex data to the NFT mint.

        // # 2.  Save liqudity position
        let liquidity_position = &mut ctx.accounts.liquidity_position;
        liquidity_position.bump = *ctx.bumps.get("liquidity_position").unwrap();
        liquidity_position.liquidity = amount;
        liquidity_position.token_mint = ctx.accounts.token_program.key();
        liquidity_position.used_liquidity = 0;
        liquidity_position.pool = ctx.accounts.pool.key();
        liquidity_position.nft_mint = ctx.accounts.nft_mint.key();
        liquidity_position.tick = tick;
        liquidity_position.created_at = Clock::get()?.unix_timestamp;
        liquidity_position.tick_id = liquidity_position_id;

        // 3. Update tick with new liquidity position
        let tick_account_state =
            AccountLoader::<tick::Tick>::try_from(&ctx.accounts.tick_account.to_account_info())?;
        let mut tick_account = tick_account_state.load_mut()?;
        match tick_account.add_liquidity(liquidity_position_id, amount) {
            Ok(_) => (),
            Err(_) => return Err(error!(SureError::CouldNotProvideLiquidity)),
        }

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
        let liquidity_position = &mut ctx.accounts.liquidity_position;
        require!(
            liquidity_position.used_liquidity < liquidity_position.liquidity,
            SureError::LiquidityFilled
        );

        // _______________ Functionality _______________
        // # 1. Find the available liquidity
        let available_liquidity = liquidity_position.liquidity - liquidity_position.used_liquidity;
        liquidity_position.liquidity = liquidity_position.used_liquidity;

        // # 2 Transfer liquidity back to nft holder
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_account.to_account_info().clone(),
                token::Transfer {
                    from: ctx.accounts.vault_account.to_account_info().clone(),
                    to: ctx.accounts.nft_holder.to_account_info().clone(),
                    authority: ctx.accounts.vault_account.to_account_info().clone(),
                },
                &[&[&[ctx.accounts.protocol_owner.load()?.bump] as &[u8]]],
            ),
            available_liquidity,
        )?;

        Ok(())
    }

    /// Buy insurance
    /// A buyer should select an amount to insure and the smart contract should
    /// premium
    ///
    ///
    /// # Arguments
    /// * ctx
    ///
    pub fn buy_insurance(ctx: Context<BuyInsurance>, amount: u64) -> Result<()> {
        // _______________ Validation __________________
        // * Check that the

        // _______________ Functionality _______________
        // Buys liquidity (within a tick) 
        // # 1. Iterate across positions + ticks, marking as bought until you find a matching amount
        // # 2.  
        let mut tick = ctx.accounts.tick.load_mut()?;
        let mut idx = tick.find_idx_of_available_liquidity().unwrap();

        let mut bought_liquidity: u64 = 0;

        // iterate until we have all the ticks we need to handle the buy
        while idx < tick.last_liquidity_position_idx as usize {

            bought_liquidity += tick.liquidity_position_size[idx];
            if bought_liquidity >= amount { break }

            idx += 1;
            // if there is not enough liquidity
            if idx > tick.last_liquidity_position_idx as usize {
                return Err(error!(SureError::NotEnoughLiquidity))
            }
        }

        // set last position
        tick.last_liquidity_position_idx = idx as u8;
        // set used liquidity
        tick.used_liquidity += amount; //TODO: partial

        // TODO: mint contract 

        // TODO: token transfer into 

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
