use std::{borrow::Borrow, cell::RefMut};

use super::{fee::FeePackage, tick_v2::TickUpdate};
use crate::common::{
    errors::SureError, liquidity::calculate_new_liquidity, seeds::*, tick_math::*,
};
use anchor_lang::{prelude::*, Result};

use super::{
    liquidity::LiquidityPosition,
    tick_v2::{calculate_fees, TickArrayPool},
    CoveragePosition,
};

/// Product Pool
/// The product pool holds information regarding the specific product
#[account]
pub struct ProductPool {
    // Product
    pub productId: u8, // 1

    /// Pools in the ProductPool
    pub pools: Vec<Pubkey>, // 4 + 32*64, 64 pools for each product
}

impl ProductPool {
    const MAX_POOLS: usize = 64;
    pub const SPACE: usize = 1 + 4 + 32 * 64;

    pub fn initialize(&mut self, productId: u8) -> Result<()> {
        self.productId = productId;
        self.pools = Vec::new();
        Ok(())
    }

    pub fn add_pool(&mut self, pool: Pubkey) -> Result<()> {
        if self.pools.len() >= ProductPool::MAX_POOLS {
            return Err(SureError::PoolsInProductPoolExceeded.into());
        }
        self.pools.push(pool);
        Ok(())
    }

    pub fn remove_pool(&mut self, pool_pk: Pubkey) -> Result<()> {
        if self.pools.len() == 0 {
            return Err(SureError::ProductPoolIsEmpty.into());
        }
        if let Some(idx) = self.pools.iter().position(|x| *x == pool_pk) {
            self.pools.remove(idx);
        }
        Ok(())
    }
}

/// Sure Pool
///
/// Sure pool is used to provide liquidity for
/// customers of the given product
#[account]
pub struct Pool {
    /// bump
    pub bump_array: [u8; 1], // 1 byte

    /// ProductId
    pub productId: u8,

    /// Name of pool
    pub name: String, // 4 + 200 bytes

    /// Founder of pool
    pub founder: Pubkey,

    /// space between each tick in basis point
    pub tick_spacing: u16,
    pub tick_spacing_seed: [u8; 2],

    /// fee rate for each transaction in pool
    // hundreth of a basis point i.e. fee_rate = 1 = 0.01 bp = 0.00001%
    pub fee_rate: u16,

    /// Protocol fee of transaction
    /// (1/x)% of fee_rate
    pub protocol_fee_rate: u16,

    /// Founder fee of transaction
    /// (1/x)% of fee_rate
    pub founders_fee_rate: u16,

    /// Liquidity in Pool
    pub liquidity: u64, // 8 bytes

    /// The current market price as
    /// use Q32.32 - 32 bytes at each
    /// side of decimal point
    pub sqrt_price_x32: u64, // 16bytes

    /// Current tick index corrensponding to sqrt price
    pub current_tick_index: i32,

    /// Tokens in vault 0 that is owed to the sure
    pub protocol_fees_owed_0: u64,
    pub founders_fees_owed_0: u64,
    /// total fees in vault a collected per unit of liquidity
    pub fee_growth_0_x32: u64,

    /// Tokens in vault 0 that is owed to the sure
    pub protocol_fees_owed_1: u64,
    pub founders_fees_owed_1: u64,
    /// total fees collected in vault b per unit of liquidity
    pub fee_growth_1_x32: u64,

    /// Token mint A of pool
    pub token_mint_0: Pubkey, // 32 bytes
    pub token_vault_0: Pubkey, //32 bytes

    /// Token mint B of pool
    pub token_mint_1: Pubkey, // 32 bytes
    pub token_vault_1: Pubkey, //32 bytes

    /// Used liquidity
    pub used_liquidity: u64, // 8 bytes
}

impl Pool {
    pub const SPACE: usize = 1 + 32 + 32 + 8 + 8 + 4 + 200;

    pub fn seeds(&self) -> [&[u8]; 5] {
        [
            &SURE_TOKEN_POOL_SEED.as_bytes() as &[u8],
            self.token_mint_0.as_ref(),
            self.token_mint_1.as_ref(),
            self.tick_spacing_seed.as_ref(),
            self.bump_array.as_ref(),
        ]
    }

    pub fn initialize(
        &mut self,
        bump: u8,
        productId: u8,
        name: String,
        founder: Pubkey,
        tick_spacing: u16,
        fee_package: &Account<FeePackage>,
        sqrt_price_x32: u64,
        token_mint_0: Pubkey,
        token_mint_1: Pubkey,
        pool_vault_0: Pubkey,
        pool_vault_1: Pubkey,
    ) -> Result<()> {
        self.bump_array = bump.to_le_bytes();

        self.productId = productId;
        self.name = name;
        self.founder = founder;
        self.tick_spacing = tick_spacing;
        self.tick_spacing_seed = tick_spacing.to_le_bytes();
        fee_package.validate_fee_rates()?;

        self.fee_rate = fee_package.fee_rate;
        self.protocol_fee_rate = fee_package.protocol_fee_rate;
        self.founders_fee_rate = fee_package.founders_fee_rate;

        self.liquidity = 0;
        self.current_tick_index = get_tick_at_sqrt_ratio(sqrt_price_x32)?;
        self.sqrt_price_x32 = sqrt_price_x32;
        if token_mint_0.ge(&token_mint_1) {
            return Err(SureError::WrongTokenMintOrder.into());
        }
        self.token_mint_0 = token_mint_0;
        self.token_mint_1 = token_mint_1;
        self.token_vault_0 = pool_vault_0;
        self.token_vault_1 = pool_vault_1;

        Ok(())
    }

    /// Update fees collected by the pool
    /// Should happen on each tx
    pub fn update_post_transaction(
        &mut self,
        liquidity: u64,
        tick: i32,
        fee_growth: u64,
        protocol_fee: u64,
        is_fee_in_a: bool,
    ) -> Result<()> {
        self.liquidity = liquidity;
        self.sqrt_price_x32 = get_sqrt_ratio_at_tick(tick)?;
        if is_fee_in_a {
            self.fee_growth_0_x32 = fee_growth;
            self.protocol_fees_owed_0 += protocol_fee;
        } else {
            self.fee_growth_1_x32 = fee_growth;
            self.protocol_fees_owed_1 += protocol_fee;
        }

        Ok(())
    }

    pub fn update_liquidity(&mut self, liquidity: u64) -> Result<()> {
        self.liquidity = liquidity;
        Ok(())
    }

    /// Get the current tick index from the
    /// current sqrt price
    pub fn get_current_tick_index(&self) -> Result<i32> {
        get_tick_at_sqrt_ratio(self.sqrt_price_x32)
    }

    /// Update the fee package
    pub fn update_fee_package(&mut self, fee_package: FeePackage) -> Result<()> {
        fee_package.validate_fee_rates()?;
        self.fee_rate = fee_package.fee_rate;
        self.protocol_fee_rate = fee_package.protocol_fee_rate;
        self.founders_fee_rate = fee_package.founders_fee_rate;
        Ok(())
    }

    /// Is position in range
    /// Check to see if the position given by lower and upper
    /// tick is in range
    pub fn is_position_in_range(&self, position: &LiquidityPosition) -> Result<bool> {
        if self.current_tick_index >= position.tick_index_lower
            && self.current_tick_index < position.tick_index_upper
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get next liquidity
    /// If the current tick in the pool is in the position
    /// then calculate the next liquidity
    pub fn get_next_liquidity(&self, position: &LiquidityPosition, delta: i64) -> Result<u64> {
        if self.is_position_in_range(position)? {
            calculate_new_liquidity(self.liquidity, delta)
        } else {
            Ok(self.liquidity)
        }
    }

    /// Update the pool after a coverage change
    pub fn update_after_coverage_change(
        &mut self,
        buy_coverage_result: &BuyCoverageResult,
    ) -> Result<()> {
        self.current_tick_index = buy_coverage_result.next_tick_index;
        self.sqrt_price_x32 = buy_coverage_result.next_sqrt_price;
        self.liquidity = self.liquidity;
        self.fee_growth_0_x32 = buy_coverage_result.fee_growth;
        self.protocol_fees_owed_0 += buy_coverage_result.protocol_fee;
        self.founders_fees_owed_0 += buy_coverage_result.founders_fee;

        Ok(())
    }
    /// Update coverage
    ///
    /// Start from the current sqrt price and move upwards
    /// until either the liquidity ends or coverage is fullfilled
    ///
    /// The coverage is bought from the tick_array_pool but the
    /// state has to be synced in the pool as well
    ///
    /// The operation will move from the lowest tick array and fill up
    /// ticks until either the coverage amount is met or there is no
    /// more liquidity in the ticks.
    ///
    /// The method will return the calculated premium that the used have
    /// to deposit in the premium vault (token_vault_1)
    ///
    /// The fees will be drawn from the premium
    /// * Parameters
    /// - tick_array_pool: keeps and overview over liquidity at 3 tick arrays
    /// - coverage_position: keeps track of covered amounts at each tick
    /// - coverage_amount_delta: the change in coverage amount
    /// - increase_coverage <bool>
    /// If increase_coverage = true then increase coverage
    pub fn update_coverage(
        &mut self,
        mut tick_array_pool: TickArrayPool,
        mut coverage_position: RefMut<CoveragePosition>,
        coverage_amount_delta: u64,
        expiry_ts: i64,
        increase_coverage: bool,
        is_target_amount: bool,
    ) -> Result<BuyCoverageResult> {
        // If we are reducing the coverage position
        // we are actually moving downwards from
        // the current coverage position
        // Nothing to cover
        if coverage_amount_delta == 0 {
            return Err(SureError::InvalidAmount.into());
        }

        let mut coverage_amount_remaining = coverage_amount_delta; // given in u64
        let mut coverage_amount: u64 = 0; // amount that is covered
        let mut coverage_premium: u64 = 0; // premium to be deposited

        let mut current_liquidity = self.liquidity; // u64 given in token 0
        let mut current_fee_growth = self.fee_growth_0_x32; //Q32.32
                                                            // If increase coverage start at the current price, which is the global min
        let mut current_tick_index = if increase_coverage {
            self.current_tick_index
        } else {
            coverage_position.get_current_coverage_position(self.tick_spacing)?
        }; // runs from -221_818 to 221_818

        let mut current_array_index: usize = 0; // which array in the tick array pool
        let mut current_protocol_fee: u64 = 0; // Q32.32 to represent the fee
        let mut current_founders_fee: u64 = 0; // Q32.32
        let mut current_sqrt_price = get_sqrt_ratio_at_tick(current_tick_index)?;
        // set the sqrt price limit at either the max of the tick array
        // or the minimum of the coverage position
        let sqrt_price_limit = if increase_coverage {
            tick_array_pool.max_sqrt_price_x32(self.tick_spacing)?
        } else {
            coverage_position.get_lowest_sqrt_price_x32()?
        };

        // Fee rates
        let fee_rate = self.fee_rate;

        // while the coverage amount remaining is greater
        // than 0 and there is more liquidity
        while coverage_amount_remaining > 0
            && current_sqrt_price != sqrt_price_limit
            && !coverage_position.is_tick_index_out_of_bounds(
                current_tick_index,
                self.tick_spacing,
                true,
            )
        {
            // find the next tick index with enough liquidity
            // if increase_coverage move left -> right
            let (next_array_index, next_tick_index) = tick_array_pool.find_next_free_tick_index(
                current_tick_index,
                self.tick_spacing,
                !increase_coverage,
                current_array_index,
            )?;

            // find the price/premium at current tick
            let next_sqrt_price_x32 = get_sqrt_ratio_at_tick(next_tick_index)?;

            //
            let current_tick =
                tick_array_pool.get_tick(next_array_index, next_tick_index, self.tick_spacing)?;

            // calculate tick change
            let current_covered_amount =
                coverage_position.get_coverage_at_tick_index(next_tick_index, self.tick_spacing)?;
            let coverage_tick_delta = if increase_coverage {
                current_tick
                    .get_available_liquidity()
                    .min(coverage_amount_remaining)
            } else {
                current_covered_amount.min(coverage_amount_remaining)
            };

            // Calculate the fee and the in and out amounts
            let (fee_amount, amount_in, amount_out) = current_tick.calculate_coverage_delta(
                next_tick_index,
                coverage_tick_delta,
                current_covered_amount,
                fee_rate,
                coverage_position.expiry_ts,
                expiry_ts,
                increase_coverage,
            )?;

            // calculate remaining coverage
            if increase_coverage {
                coverage_amount_remaining = coverage_amount_remaining
                    .checked_sub(coverage_tick_delta)
                    .ok_or(SureError::OverflowU64)?;
                coverage_amount = coverage_amount
                    .checked_add(coverage_tick_delta)
                    .ok_or(SureError::OverflowU64)?;
            } else {
                coverage_amount_remaining = coverage_amount_remaining
                    .checked_sub(coverage_tick_delta)
                    .ok_or(SureError::OverflowU64)?;
                coverage_amount = coverage_amount
                    .checked_sub(coverage_tick_delta)
                    .ok_or(SureError::OverflowU64)?;
            }

            // calculate premium based on current price
            coverage_premium += if increase_coverage {
                amount_in
            } else {
                amount_out
            };

            // Calculate fees
            let (next_protocol_fee, next_founders_fee, next_fee_growth) = calculate_fees(
                fee_amount,
                self.protocol_fee_rate,
                self.founders_fee_rate,
                self.liquidity,
                current_protocol_fee,
                current_founders_fee,
                current_fee_growth,
            )?;
            current_protocol_fee = next_protocol_fee;
            current_founders_fee = next_founders_fee;
            current_fee_growth = next_fee_growth;

            // Update tick
            let (fee_growth_0, fee_growth_1) = (current_fee_growth, self.fee_growth_1_x32);

            let (tick_update, next_liquidity) = current_tick.calculate_coverage_update(
                increase_coverage,
                current_liquidity,
                coverage_tick_delta,
                fee_growth_0,
                fee_growth_1,
            )?;
            tick_array_pool.update_tick(
                next_array_index,
                next_tick_index,
                self.tick_spacing,
                &tick_update,
            )?;

            current_liquidity = next_liquidity;

            // Update coverage position
            coverage_position.update_coverage_at_tick(
                next_tick_index,
                self.tick_spacing,
                coverage_tick_delta,
                expiry_ts,
                increase_coverage,
            )?;

            // Calculate sub fees
            let last_tick_index_in_array = tick_array_pool.is_last_tick_index_in_array(
                next_array_index,
                next_tick_index,
                self.tick_spacing,
            )?;
            current_array_index = if last_tick_index_in_array {
                current_array_index + 1
            } else {
                current_array_index
            };
            current_tick_index = next_tick_index;
            current_sqrt_price = next_sqrt_price_x32;
        }

        Ok(BuyCoverageResult {
            liquidity: current_liquidity,
            coverage_amount,
            premium: coverage_premium,
            fee_growth: current_fee_growth,
            protocol_fee: current_protocol_fee,
            founders_fee: current_founders_fee,
            next_sqrt_price: current_sqrt_price,
            next_tick_index: current_tick_index,
        })
    }
}

pub struct BuyCoverageResult {
    liquidity: u64,
    coverage_amount: u64,
    premium: u64,
    fee_growth: u64,
    protocol_fee: u64,
    founders_fee: u64,
    next_sqrt_price: u64,
    next_tick_index: i32,
}

impl BuyCoverageResult {
    pub fn get_total_cost_of_coverage(&self) -> Result<u64> {
        self.premium
            .checked_add(self.fee_growth)
            .ok_or(SureError::AdditionQ3232OverflowError.into())
    }

    pub fn get_coverage_amount(&self) -> u64 {
        self.coverage_amount
    }
}

#[event]
pub struct CreatePool {
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
    pub insurance_fee: u16,
}
