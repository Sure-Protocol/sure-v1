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

    /// Initialize Fee Package
    ///
    /// Each pool has a fee package attached to it
    ///
    /// the fee package determines the amount of fees
    /// paid to managers of the pool
    ///
    /// ### Arguments
    /// * fee_rate<u16>: 100th of a basis point (0.0001%) as fee
    /// * protocol_fee_rate<u16>: (1/x) of the fee_rate that goes to the protocol
    /// * founders_fee_rate<u16>: (1/x) of the fee_rate that goes to the founder of the pool
    ///
    /// ### Founder
    /// The founder of the pool is essentially the mamager of the pool. The manager is
    /// incentivized to source liquidity and find users and projects that are interested
    /// in reducing the risk of their position. It is basically profit sharing.
    pub fn initialize_fee_package(
        ctx: Context<InitializeFeePackage>,
        fee_rate: u16,
        protocol_fee_rate: u16,
        founders_fee_rate: u16,
    ) -> Result<()> {
        instructions::initialize_fee_package::handler(
            ctx,
            fee_rate,
            protocol_fee_rate,
            founders_fee_rate,
        )
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
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        product_id: u8,
        tick_spacing: u16,
        sqrt_price_x32: u64,
        name: String,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, product_id, tick_spacing, sqrt_price_x32, name)
    }

    /// Initialize Tick Array
    ///
    /// ### Arguments
    /// - start_tick_index<i32>: index in [-221 818,221 818]
    pub fn initialize_tick_array(
        ctx: Context<InitializeTickArray>,
        start_tick_index: i32,
    ) -> Result<()> {
        instructions::initialize_tick_array::handler(ctx, start_tick_index)
    }

    /// Initialize Liquidity Positon
    ///
    /// initializes a liquidity position
    ///
    /// ### Arguments
    /// - tick_index_upper<i32>: the upper tick index in the position
    /// - tick_index_lower<i32>: the lower tick index in the position
    pub fn initialize_liquidity_position(
        ctx: Context<InitializeLiquidityPosition>,
        tick_index_upper: i32,
        tick_index_lower: i32,
    ) -> Result<()> {
        instructions::initialize_liquidity_position::handler(
            ctx,
            tick_index_upper,
            tick_index_lower,
        )
    }

    /// Initialize Coverage Position
    ///
    /// Creates a new coverage position
    ///
    /// ### Arguments
    /// - start_tick_index<i32>: the tick index to start the contract at. The upper bound
    ///                          is start_tick_index + 64*3*tick_spacing
    pub fn initialize_coverage_position(
        ctx: Context<InitializeCoveragePosition>,
        start_tick_index: i32,
    ) -> Result<()> {
        instructions::initialize_coverage_position::handler(ctx, start_tick_index)
    }

    /// Increase Liquidity Position
    ///
    /// Lets a user update the liquidity position with more
    /// liquidity.
    ///
    /// The product id of the pool determines the interpretation of the arguments
    ///
    /// ### Arguments
    /// * amount<u64>: the amount of liquidity to add to the position. Denominated in token 0
    /// * max_token_0: the max amount of token 0 to input
    /// * min_token_0: the min amount of token 0 to input
    pub fn increase_liquidity_position(
        ctx: Context<IncreaseLiquidityPosition>,
        amount: u64,
        max_token_0: u64,
        min_token_0: u64,
    ) -> Result<()> {
        instructions::increase_liquidity_position::handler(ctx, amount, max_token_0, min_token_0)
    }

    /// Decrease Liquidity Position
    ///
    /// A holder can redeem liquidity that is not in use.
    /// position is used can redeem the unused liquidity.
    ///
    /// If some of the liquidity is active then it can only be withdrawn
    /// if there is free liquidity in the tick pool.
    ///
    /// # Arguments
    /// * ctx
    ///
    pub fn decrease_liquidity_position(
        ctx: Context<DecreaseLiquidityPosition>,
        liquidity_amount: u64,
        token_min_a: u64,
        token_min_b: u64,
    ) -> Result<()> {
        instructions::decrease_liquidity_position::handler(
            ctx,
            liquidity_amount,
            token_min_a,
            token_min_b,
        )
    }

    /// Increase Coverage Position
    ///
    /// an user with a coverage position can
    /// at any time increase the coverage position
    ///
    /// If the value of the contract increases
    /// f(expiry+e,amount+n) > f(expiry,amount)
    /// the user have to post more premium
    ///
    /// ### Arguments
    /// * coverage_amount<u64>: the increase in coverage if is_target_amount = false
    /// * expiry_ts<i64>: the unix timestamp of when the contract expires
    /// * is_target_amount: is coverage_amount the final coverage after instruction
    pub fn increase_coverage_position(
        ctx: Context<ChangeCoveragePosition>,
        coverage_amount: u64,
        expiry_ts: i64,
        is_target_amount: bool,
    ) -> Result<()> {
        instructions::increase_coverage_position::handler(
            ctx,
            coverage_amount,
            expiry_ts,
            is_target_amount,
        )
    }

    /// Decrease Coverage Position
    ///
    /// an user with an existing coverage position can
    /// decrease the amount covered and expiry.
    ///
    /// ### Arguments
    /// * coverage_amount<u64>: the increase in coverage if is_target_amount = false
    /// * expiry_ts<i64>: the unix timestamp of when the contract expires
    /// * is_target_amount: is coverage_amount the final coverage after instruction
    pub fn decrease_coverage_position(
        ctx: Context<ChangeCoveragePosition>,
        coverage_amount: u64,
        expiry_ts: i64,
        is_target_amount: bool,
    ) -> Result<()> {
        instructions::decrease_coverage_position::handler(
            ctx,
            coverage_amount,
            expiry_ts,
            is_target_amount,
        )
    }
}
