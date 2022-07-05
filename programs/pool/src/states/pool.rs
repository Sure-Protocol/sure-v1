use super::fee::FeePackage;
use crate::common::{
    errors::SureError, liquidity::calculate_new_liquidity, seeds::*, tick_math::*,
};
use anchor_lang::{prelude::*, Result};

use super::{liquidity::LiquidityPosition, tick_v2::TickArrayPool};

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
    pub bump: u8, // 1 byte

    /// ProductId
    pub productId: u8,

    /// Name of pool
    pub name: String, // 4 + 200 bytes

    /// Founder of pool
    pub founder: Pubkey,

    /// space between each tick in basis point
    pub tick_spacing: u16,

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
    pub fees_token_0_owed: u64,
    /// total fees in vault a collected per unit of liquidity
    pub fee_growth_0_x32: u64,

    /// Tokens in vault 0 that is owed to the sure
    pub fees_token_1_owed: u64,
    /// total fees collected in vault b per unit of liquidity
    pub fee_growth_1_x32: u64,

    /// Token mint A of pool
    pub token_mint_0: Pubkey, // 32 bytes
    pub pool_vault_0: Pubkey, //32 bytes

    /// Token mint B of pool
    pub token_mint_1: Pubkey, // 32 bytes
    pub pool_vault_1: Pubkey, //32 bytes

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
            self.tick_spacing.to_le_bytes().as_ref(),
            &[self.bump],
        ]
    }

    pub fn initialize(
        &mut self,
        bump: u8,
        productId: u8,
        name: String,
        founder: Pubkey,
        tick_spacing: u16,
        fee_package: FeePackage,
        sqrt_price_x32: u64,
        token_mint_0: Pubkey,
        token_mint_1: Pubkey,
        pool_vault_0: Pubkey,
        pool_vault_1: Pubkey,
    ) -> Result<()> {
        self.bump = bump;
        self.productId = productId;
        self.name = name;
        self.founder = founder;
        self.tick_spacing = tick_spacing;
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
        self.pool_vault_0 = pool_vault_0;
        self.pool_vault_1 = pool_vault_1;

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
            self.fees_token_0_owed += protocol_fee;
        } else {
            self.fee_growth_1_x32 = fee_growth;
            self.fees_token_1_owed += protocol_fee;
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

    /// Buy coverage
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
    pub fn buy_coverage(
        &mut self,
        tick_array_pool: TickArrayPool,
        coverage_amount: u64,
    ) -> Result<u64> {
        // Nothing to cover
        if coverage_amount == 0 {
            return Err(SureError::InvalidAmount.into());
        }

        let mut coverage_amount_remaining = coverage_amount; // given in u64
        let mut coverage_premium: u64 = 0; // premium to be deposited

        // Buy insurance left -> right
        let mut current_liquidity = self.liquidity; // u64 given in token 0
        let mut current_fee_growth = self.fee_growth_0_x32; //Q32.32
        let mut current_tick_index = self.current_tick_index; // runs from -221_818 to 221_818
        let mut current_array_index: usize = 0; // which array in the tick array pool
        let mut current_protocol_fee: u64 = 0; // Q32.32 to represent the fee
        let mut current_fee_growth = self.fee_growth_0_x32; // Q32.32
        let mut current_founders_fee: u64 = 0; // Q32.32

        // Fee rates
        let fee_rate = self.fee_rate;
        let protocol_fee = self.protocol_fee_rate;
        let founders_rate = self.founders_fee_rate;

        // while the coverage amount remaining is greater
        // than 0 and there is more liquidity
        while coverage_amount_remaining > 0 {
            // find the next tick index with enough liquidity, starting from current_tick_index, left -> right
            let (tick_array_index, next_tick_index) = tick_array_pool.find_next_free_tick_index(
                current_tick_index,
                self.tick_spacing,
                true,
                current_array_index,
            )?;

            // find the price/premium at current tick
            let sqrt_price_x32 = get_sqrt_ratio_at_tick(next_tick_index)?;

            //
            let current_tick_array = tick_array_pool.arrays.get(tick_array_index).unwrap();
            let current_tick = current_tick_array.get_tick(next_tick_index, self.tick_spacing)?;
            let available_liquidity = current_tick.get_available_liquidity();

            // Calculate the amount of coverage for tick
            let coverage_delta = if available_liquidity > coverage_amount_remaining {
                coverage_amount_remaining
            } else {
                available_liquidity
            };

            // Calculate premium for given tick
            let price_x32 = sqrt_price_x32
                .checked_mul(sqrt_price_x32)
                .ok_or(SureError::MultiplictationQ3232Overflow)?;
            let premium = price_x32
                .checked_mul(coverage_delta)
                .ok_or(SureError::MultiplictationQ3232Overflow)?;
            coverage_premium += premium;

            // Calculate fees
            // Get the fee amount
            // Calculate sub fees
            let (next_protocol_fee, next_founders_fee, next_fee_growth) = calculate_fee_amounts(
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
            let next_liquidity = current_tick.update_tick_on_buy(
                true,
                current_liquidity,
                current_fee_growth,
                self.fee_growth_1_x32,
            )?;
            current_liquidity = next_liquidity;

            // if current tick is the last tick in the
            // tick array iterate to the next tick array.
            let tick_array = tick_array_pool.arrays[current_array_index];

            if tick_array.is_last_tick(current_tick_index, self.tick_spacing)? {
                current_array_index += 1;
                current_tick_index = tick_array
                    .find_next_available_tick(current_tick_index, self.tick_spacing, true)?
                    .unwrap();
            } else {
                //current_array_index
            };
        }

        Ok(0)
    }
}

#[event]
pub struct CreatePool {
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
    pub insurance_fee: u16,
}
