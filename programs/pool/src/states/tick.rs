///! Tick contains methods to manage tick pool
///! tick will not manage the premium pool but rather keep
///! track of the potential rewards.
///!
use anchor_lang::prelude::*;
use std::fmt::{self};
use std::{error::Error, fmt::Display, fmt::Formatter, result::Result};

use crate::utils::bitmap::C256;
use crate::utils::errors;
use crate::utils::uint::U256;

pub const MAX_NUMBER_OF_LIQUIDITY_POSITIONS: u8 = u8::MAX;
pub const SECONDS_IN_A_YEAR: i64 = 31556926;
/// Tick acount (PDA) is used to hold information about
/// the liquidity at a current tick




#[account(zero_copy)]
pub struct Tick {
    /// The bump identity of the PDA
    pub bump: u8, // 1 byte

    /// The active liquidity at the tick
    pub liquidity: u64, // 8bytes

    /// Amount of liquidity used from the pool
    pub used_liquidity: u64, // 8 bytes

    /// last slot the tick was updated on
    pub last_updated: i64,

    /// The tick in basis points
    pub tick: u64, // 8 bytes

    /// Boolean representing whether the liquidity is active
    pub active: bool, // 1 byte

    /// Ids of liquidity positions
    pub liquidity_position_idx: [u8; (MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize)], // 2048*4 = 8192 bytes, 8kb

    /// Accumulation of Liquidity Provided
    pub liquidity_position_accumulated: [u64; (MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize)],

    /// rewards
    pub liquidity_position_rewards: [u64; (MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize)], // 8192 bytes

    pub last_liquidity_position_idx: u8,
}

impl Tick {
    pub fn validate(&self) -> Result<(),errors::SureError> {
        Ok(())
    }

    pub fn initialize(&mut self,bump: u8,tick_bp: u64) -> Result<(),error::Error> {
        self.bump = bump;
        self.liquidity = 0;
        self.used_liquidity = 0;
        self.last_updated = Clock::get()?.unix_timestamp;
        self.tick = tick_bp;
        self.active = true;
        self.liquidity_position_idx = [0;MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize];
        self.liquidity_position_accumulated = [0;MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize];
        self.liquidity_position_rewards = [0;MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize];

        Ok(())

    }
 }

pub trait TickTrait {
    fn get_new_id(&self) -> u8;
    fn buy_insurance(&mut self, size: u64) -> Result<(), TickError>;
    fn exit_insurance(&mut self, size: u64) -> Result<(), TickError>;
    fn add_liquidity(&mut self, id: u8, size: u64) -> Result<(), TickError>;
    fn remove_liquidity(&mut self, id: u8) -> Result<(), TickError>;
    fn free_liquidity(&mut self,id: u8) -> u64;
    fn is_empty(&self) -> bool;
    fn increase_rewards(&mut self) -> Result<(), TickError>;
    fn get_rewards(&mut self, id: u8) -> Result<u64, TickError>;
    fn withdraw_rewards(&mut self, id: u8) -> Result<(), TickError>;
    fn update_callback(&mut self) -> Result<(), TickAnchorError>;
}

#[derive(Debug)]
pub struct TickError {
    pub cause: String,
}

#[error_code]
pub enum TickAnchorError {
    #[msg("could not update timestamp")]
    CouldNotUpdateTimestamp,
}

impl TickError {
    pub fn to_anchor_error(&self) -> TickAnchorError {
        TickAnchorError::CouldNotUpdateTimestamp
    }
}

impl Error for TickError {}

impl Display for TickError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Tick error")
    }
}

impl TickTrait for Tick {

    /// Get new id 
    /// generates a new id for a liquidity position
    fn get_new_id(&self) -> u8 {
        let mut idx = 1;
        while self.is_id_taken(idx) {
            idx += 1;
        }
        idx
    }

    /// Buy insurance
    /// Method to call when buying insurance from the tick pool
    ///
    /// # Arguments:
    /// * self: tick state
    /// * size: amount of insurance to buy
    ///
    fn buy_insurance(&mut self, size: u64) -> Result<(), TickError> {
        if self.used_liquidity + size > self.liquidity {
            return Err(TickError {
                cause: "too little liquidity".to_string(),
            });
        }
        self.used_liquidity += size;

        self.update_callback();
        Ok(())
    }

    /// Exit insurance
    /// take the liquidity off the tick state.
    ///
    /// # Arguments
    /// * self: tick state
    /// * size: amount to exit the pool
    ///
    fn exit_insurance(&mut self, size: u64) -> Result<(), TickError> {
        if self.liquidity < size {
            return Err(TickError {
                cause: "not possible to exit the pool.".to_string(),
            });
        }
        self.used_liquidity -= size;

        self.update_callback();
        Ok(())
    }

    /// Add liquidity
    ///
    /// Updates the liquidity in the tick by adding a new position to the the
    ///     - liquidity index
    ///     - liquidity size
    ///     - liquidity rewards
    ///
    /// # Arguments
    /// * id: The id in the liquidity position seed
    /// * size: the size of the liquidity added
    fn add_liquidity(&mut self, id: u8, size: u64) -> Result<(), TickError> {
        if (MAX_NUMBER_OF_LIQUIDITY_POSITIONS) == (self.last_liquidity_position_idx+1) {
            return Err(TickError {
                cause: "no liquidity spots left".to_string(),
            });
        }

        // Update the tick liquidity
        self.liquidity += size;
        let mut liquidity_accumulated:u64 = 0;
        let next_liquidity_idx = if self.active {
            liquidity_accumulated = self.liquidity_position_accumulated[self.last_liquidity_position_idx as usize];
            (self.last_liquidity_position_idx + 1) as usize
        } else {
            self.last_liquidity_position_idx as usize
        };

        self.liquidity_position_idx[next_liquidity_idx] = id;
        self.liquidity_position_accumulated[next_liquidity_idx] = liquidity_accumulated + size;
        self.liquidity_position_rewards[next_liquidity_idx] = 0;
        self.last_liquidity_position_idx += 1;

        self.active = true;

        self.update_callback();
        Ok(())
    }

    /// Remove Liquidity
    ///
    /// Finds the id in the liquidity position index array and
    /// moves every element after the position one spot to the left
    /// Ex: idx: [2,43,12,53,32,0,0], rm 12 -> [2,44,53,32,0,0,0]
    ///
    /// Can only remove if the liquidity is not in use
    ///
    /// # Arguments
    /// * id: The id in the liquidity position seed
    ///
    fn remove_liquidity(&mut self, id: u8) -> Result<(), TickError> {
        let idx = self.find_liquidity_position_idx(id);
        if self.liquidity_position_rewards[idx] != 0 {
            return Err(TickError {
                cause: "rewards should be withdrawn".to_string(),
            });
        }

        // Check if position is in use 
        let accumulated_position = self.liquidity_position_accumulated[idx as usize];
        if self.used_liquidity > accumulated_position {
            return Err(TickError{
                cause: "can not exit position as it is in use.".to_string(),
            })
        }

        // If only parts of position is in use, remove the part that 
        // is not active 
        let liquidity_position_size = self.liquidity_position(idx as u8);
        println!("Liquidity position size: {}",liquidity_position_size);
        let current_idx = idx;

        let max_number_of_liquidity_positions_as_usize =
            (MAX_NUMBER_OF_LIQUIDITY_POSITIONS-1) as usize;

        while self.liquidity_position_idx[current_idx] != 0
            && current_idx < max_number_of_liquidity_positions_as_usize
        {
           self.shift_liquidity_position_left_by1(current_idx as u8);
        }
        // Set last position in the arrays to 0
        self.liquidity_position_idx[max_number_of_liquidity_positions_as_usize] = 0;
        self.liquidity_position_rewards[max_number_of_liquidity_positions_as_usize] = 0;
        self.liquidity_position_accumulated[max_number_of_liquidity_positions_as_usize] = 0;

        // Update liquidity position counter
        self.last_liquidity_position_idx -= 1;
        self.liquidity -= liquidity_position_size;

        if self.last_liquidity_position_idx == 0 {
            self.active = false;
        }

        self.update_callback();
        Ok(())
    }

    /// Crank for updating rewards.
    /// Assume that method is called on each change
    ///
    /// # Arguments
    /// * Tick
    fn increase_rewards(&mut self) -> Result<(), TickError> {
        let mut cumulative_liquidity = 0;
        let mut idx = 0;
        while cumulative_liquidity < self.used_liquidity {
            let liquidity_position = self.liquidity_position(idx);
            self.liquidity_position_rewards[idx as usize] +=
                self.reward_calculations(liquidity_position, 1.0)?;
            cumulative_liquidity += liquidity_position;
            idx += 1;
        }
        let remaining_liquidity = self.used_liquidity - cumulative_liquidity;
        let current_liquidity_position = self.liquidity_position(idx);

        // Since liquidity is in lamports, ratio would be in
        let last_lp_utilization = remaining_liquidity as f64 / current_liquidity_position as f64;
        self.liquidity_position_rewards[idx as usize] +=
            self.reward_calculations(self.liquidity_position(idx), last_lp_utilization)?;

        self.update_callback();
        Ok(())
    }

    /// Get Rewards
    /// get rewards allows users to check the current reward
    ///
    /// # Arguments
    /// * Tick
    /// * id: The id in the liquidity position seed
    ///
    fn get_rewards(&mut self, id: u8) -> Result<u64, TickError> {
        let idx = self.find_liquidity_position_idx(id);

        self.update_callback();
        Ok(self.liquidity_position_rewards[idx])
    }

    /// Withdraw Rewards
    /// empties rewards struct 
    /// 
    /// # Arguments
    /// * ctx: Tick account
    /// * id: the liquidity position identifier
    fn withdraw_rewards(&mut self, id: u8) -> Result<(), TickError> {
        let idx = self.find_liquidity_position_idx(id);

        self.liquidity_position_rewards[idx] = 0;

        self.update_callback();
        Ok(())
    }

    /// Update Callback
    /// this function should be called each time a
    ///  - write
    ///  - update
    /// has occurred. Its only function is to update the last
    /// updated field to the current unix timestamp provided by
    /// the solana runtime.
    ///
    /// # Arguments
    /// * self: Tick account
    /// 
    fn update_callback(&mut self) -> Result<(), TickAnchorError> {
        self.last_updated = match self.get_unix_timestamp() {
            Ok(res) => res,
            Err(err) => return Err(err.to_anchor_error())
        };
        Ok(())
    }

    /// free liquidity
    /// calculates the amount of liquidity for the given 
    /// position that is not in use
    /// 
    /// # Arguments
    /// * id: the Liquidity Position id
    /// 
    fn free_liquidity(&mut self,id: u8) -> u64 {
        let current_index = id as usize;
        // All liquidity is used 
        if self.used_liquidity > self.liquidity_position_accumulated[current_index] {
            return 0
        }

        
        let liquidity_position_size = self.liquidity_position_accumulated[current_index]-self.liquidity_position_accumulated[current_index-1];
        let diff = self.used_liquidity - self.liquidity_position_accumulated[current_index-1];
        // Parts of the liquidity is used 
        if diff > 0 {
            return liquidity_position_size - diff
        }

        self.update_callback();

        return liquidity_position_size
    }

    fn is_empty(&self) -> bool{
        self.used_liquidity == 0 &&self.liquidity == 0 && !self.active
    }
}

impl Tick {

    // ____________ Internal functions ___________________ //
    /// Calculate reward for a liquidity position
    ///
    /// # Arguments
    /// * Liquidity: the amount of liquidity, given with decimals. 1 token = 1*e^dec
    /// * Utilization: percentage the amount of liquidity use, [0,1]
    ///
    fn reward_calculations(&self, liquidity: u64, utilization: f64) -> Result<u64, TickError> {
        let liquidity_f = liquidity as f64;
        let current_timestamp = self.get_unix_timestamp()?;
        let time_diff_seconds = current_timestamp - self.last_updated;
        let time_factor = time_diff_seconds as f64 / SECONDS_IN_A_YEAR as f64;
        let tick_percentage = self.tick as f64 / 10_000.0;
        let reward = time_factor * tick_percentage * liquidity_f * utilization;
        Ok(reward as u64)
    }

    /// Find liquidity position index
    /// find the location of the liquidity position in the
    /// liquidity position array in the tick account
    ///
    /// # Arguments
    /// * self: Tick
    /// * id: the id in the liquidity position seed
    ///
    fn find_liquidity_position_idx(&self, id: u8) -> usize {
        self.liquidity_position_idx
            .iter()
            .position(|&idx| idx == id)
            .unwrap()
    }

    /// Get Unix Timestamp
    /// Simple helper function to the the timestamp
    /// from the solana runtime
    ///
    /// # Arguments
    /// * self: Tick account
    ///
    fn get_unix_timestamp(&self) -> Result<i64, TickError> {
        let last_updated = match Clock::get() {
            Ok(clock) => clock.unix_timestamp,
            Err(_) => {
                return Err(TickError {
                    cause: "could not get tick timestamp".to_string(),
                })
            }
        };
        Ok(last_updated)
    }

    /// Percentage Liquidity Used
    /// the amount of the liquidity used in a liquidity position
    /// All but max one liquidity position should have 100%.__rust_force_expr!
    ///
    /// # Arguments:
    /// * self: Tick Account
    /// * id: the id in the liquidity position seed
    ///
    fn percentage_liquidity_used(&self, id: u8) -> Result<f64, TickError> {
        // [1000,1400,2400]
        // 1300
        if self.liquidity_position_accumulated[id as usize] > self.used_liquidity{
            if self.liquidity_position_accumulated[id as usize -1] > self.used_liquidity {
                return Ok(0.0)
            }
            let liquidity_provided = self.liquidity_position_accumulated[id as usize] - self.liquidity_position_accumulated[id as usize - 1];
            let liquidity_used = self.used_liquidity - self.liquidity_position_accumulated[id as usize - 1];
            return Ok(liquidity_used as f64 / liquidity_provided as f64)
        }
        return Ok(100.0)
    }

    fn is_id_taken(&self, id: u8) -> bool {
        self.liquidity_position_idx
            .iter()
            .any(|&id_candidate| id_candidate == id)
    }

    fn liquidity_position(&self,id:u8) -> u64 {
        if id > 0{
            self.liquidity_position_accumulated[id as usize] - self.liquidity_position_accumulated[id as usize -1]
        }else {
            self.liquidity_position_accumulated[id as usize]
        }
    }

    fn get_remaining_liquidity(&self) -> u64 {
        self.used_liquidity - self.liquidity_position_accumulated[self.last_liquidity_position_idx as usize]
    }

    fn shift_liquidity_position_left_by1(&mut self, idx: u8) {
        let current_index = idx as usize;
        self.liquidity_position_idx[current_index] = self.liquidity_position_idx[current_index+ 1];
        
        // Update rewards
        self.liquidity_position_rewards[current_index] =
            self.liquidity_position_rewards[current_index + 1];

        // Accumulated position
        if self.liquidity_position_accumulated[current_index+1] == 0 {
            self.liquidity_position_accumulated[current_index] = 0;
        }else {
            let next_diff = self.liquidity_position_accumulated[current_index+1]-self.liquidity_position_accumulated[current_index];
            
            if idx > 0 {
                self.liquidity_position_accumulated[current_index] = self.liquidity_position_accumulated[current_index-1]+next_diff;
            }else {
                self.liquidity_position_accumulated[current_index] = next_diff;
            }
        }
    }

    

  
}

#[cfg(test)]
mod tests {
    use std::time::{self, SystemTime};

    use crate::utils::bitmap::C256;

    use super::*;

    fn initialize_tick() -> Tick {
        let time = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let init_liq_idx = [0; (MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize)];
        let init_liq_size = [0; (MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize)];
        let init_liq_rewards = [0; (MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize)];
        let last_liq = 0;
        Tick {
            bump: 1,
            liquidity: 0,
            used_liquidity: 0,
            last_updated: time,
            tick: 300,
            active: false,
            liquidity_position_idx: init_liq_idx,
            liquidity_position_rewards: init_liq_rewards,
            liquidity_position_accumulated: init_liq_size,
            last_liquidity_position_idx: last_liq,
        }
    }

    #[test] 
    fn shift_position() {
        let mut tick = initialize_tick();
        tick.add_liquidity(0, 1000).unwrap();
        let id: u8 = 0;
        assert_eq!(tick.liquidity_position_accumulated[id as usize],1000);
        println!("accumulated: {:?}",tick.liquidity_position_accumulated);
        println!(
            "cool: {:?}",tick.liquidity_position_accumulated[id as usize +1]
        );
        tick.shift_liquidity_position_left_by1(id);
        println!("accumulated shifted: {:?}",tick.liquidity_position_accumulated);
    }

    #[test]
    fn add_remove_liquidity() {
        // INitialize
        let mut tick = initialize_tick();

        // Add liquidity
        tick.add_liquidity(244, 1_000).unwrap();
        println!("liquidity pos: {:?}", tick.liquidity_position_accumulated);
        assert_eq!(tick.last_liquidity_position_idx, 1);
        assert_eq!(tick.liquidity, 1_000);
        assert_eq!(tick.liquidity_position_accumulated[0], 1_000);
        assert_eq!(tick.liquidity_position_idx[0], 244);
        assert_eq!(tick.liquidity_position_rewards[0], 0);

        // Remove liquidity
        tick.remove_liquidity(244).unwrap();
        assert_eq!(tick.last_liquidity_position_idx, 0);
        assert_eq!(tick.liquidity, 0);
        assert_eq!(tick.liquidity_position_accumulated[0], 0);
        assert_eq!(tick.liquidity_position_idx[0], 0);
        assert_eq!(tick.liquidity_position_rewards[0], 0);
        assert_eq!(tick.active, false);
    }

    #[test]
    fn add_and_buy_insurance() {
        let mut tick = initialize_tick();

        // Add liquidity
        tick.add_liquidity(244, 1_000).unwrap();
        println!("liquidity pos: {:?}", tick.liquidity_position_accumulated);
        assert_eq!(tick.last_liquidity_position_idx, 1);
        assert_eq!(tick.liquidity, 1_000);
        assert_eq!(tick.liquidity_position_accumulated[0], 1_000);
        assert_eq!(tick.liquidity_position_idx[0], 244);
        assert_eq!(tick.liquidity_position_rewards[0], 0);

        /// Buy insurance
        tick.buy_insurance(1_000).unwrap();
        assert_eq!(tick.used_liquidity, 1_000);
    }
}
