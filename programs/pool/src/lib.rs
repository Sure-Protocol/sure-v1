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
        productId: u8,
        tick_spacing: u16,
        sqrt_price_x32: u64,
        name: String,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, productId, tick_spacing, sqrt_price_x32, name)
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
    /// * ctx:
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
}
