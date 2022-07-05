pub mod common;
pub mod instructions;
pub mod states;
use crate::states::*;
use anchor_lang::prelude::*;

use instructions::*;

declare_id!("D47wvD2bTDXR9XqqHdP8bwYSXu2QPMW6fGHg2aEBKunM");

#[program]
pub mod sure_pool {

    use super::*;

    /// Initialize Policy Holder
    ///
    /// Prepare a new user for buying insurance
    ///
    /// # Arguments
    ///
    /// * ctx - initialize the manager
    ///
    pub fn initialize_customer(ctx: Context<InitializeCustomer>) -> Result<()> {
        instructions::initialize_customer::handler(ctx)
    }
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
    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        instructions::initialize_pool::handler(ctx)
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
    pub fn increase_liquidity_position(
        ctx: Context<IncreaseLiquidityPosition>,
        tick: u16,
        tick_pos: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::increase_liquidity_position::handler(ctx, tick, tick_pos, amount)
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
    pub fn decrease_liquidity_position(ctx: Context<DecreaseLiquidityPosition>) -> Result<()> {
        instructions::decrease_liquidity_position::handler(ctx)
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
    pub fn initialize_pool_contract(ctx: Context<InitializePoolContract>) -> Result<()> {
        instructions::initialize_pool_contract::handler(ctx)
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
    pub fn initialize_pool_tick_contract(ctx: Context<InitializePoolTickContract>) -> Result<()> {
        instructions::initialize_pool_tick_contract::handler(ctx)
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
        ctx: Context<UpdatePoolTickContract>,
        new_insured_amount_on_tick: u64,
        new_expiry_ts: i64,
    ) -> Result<()> {
        instructions::update_pool_tick_contract::handler(
            ctx,
            new_insured_amount_on_tick,
            new_expiry_ts,
        )
    }

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
        ctx: Context<InitializePoolTickLiquidity>,
        _pool: Pubkey,
        _token: Pubkey,
        tick_bp: u16,
    ) -> Result<()> {
        instructions::initialize_tick::handler(ctx, tick_bp)
    }

    /// --- Close Tick Account ---
    ///
    /// Closes tick account if there is no more liquidity in the account
    /// and transfers the rent back
    ///
    /// # Arguments
    /// * ctx
    ///
    pub fn close_pool_liquidity_tick(ctx: Context<ClosePoolLiquidityTick>) -> Result<()> {
        instructions::close_pool_tick_liquidity::handler(ctx)
    }
}
