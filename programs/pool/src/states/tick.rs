
///! Tick contains methods to manage tick
///!

use anchor_lang::prelude::*;
use std::{error::Error,fmt::Display,fmt::Formatter,result::Result};
use std::fmt::{self};


pub const MAX_NUMBER_OF_LIQUIDITY_POSITIONS: u64 = 1024;
pub const SECONDS_IN_A_YEAR: i64 = 31556926;
/// Tick acount (PDA) is used to hold information about 
/// the liquidity at a current tick
#[account]
pub struct Tick{
    /// The bump identity of the PDA
    pub bump: u8, // 1 byte

    /// The active liquidity at the tick
    pub liquidity: u64, // 8bytes

    pub used_liquidity: u64, // 8 bytes

    /// last slot the tick was updated on 
    pub last_updated: i64,

    /// The tick in basis points
    pub tick: u64, // 8 bytes 

    /// Boolean representing whether the liquidity is active
    pub active: bool, // 1 byte 

    /// Ids of liquidity positions
    pub liquidity_position_idx: [u64;(MAX_NUMBER_OF_LIQUIDITY_POSITIONS as  usize)], // 2048*4 = 8192 bytes, 8kb

    /// Liquidity Provided for each id
    pub liquidity_position_size: [u64;(MAX_NUMBER_OF_LIQUIDITY_POSITIONS as  usize)], /// 8192 bytes

    /// rewards 
    pub liquidity_position_rewards: [u64;(MAX_NUMBER_OF_LIQUIDITY_POSITIONS as  usize)], // 8192 bytes 

    /// index of last liquidity position in the lp array
    pub last_liquidity_position_idx: u64, // 2 bytes
}

#[derive(Debug)]
pub struct TickError {
    pub cause: String,
}

impl Error for TickError{}


impl Display for TickError {
    fn fmt(&self,f: &mut Formatter<'_>) -> fmt::Result{
        write!(f,"Tick error")
    }
}

impl Tick {

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
   pub fn add_liquidity(&mut self, id: u64,size: u64) -> Result<(),TickError>{
       if (MAX_NUMBER_OF_LIQUIDITY_POSITIONS-1) == self.last_liquidity_position_idx {
           return Err(TickError{cause: "no liquidity spots left".to_string()})
       }

       // Update the tick liquidity 
       self.liquidity += size;
       
       let next_liquidity_idx = if self.active {(self.last_liquidity_position_idx+1) as usize} else { self.last_liquidity_position_idx as usize};
       
       self.liquidity_position_idx[next_liquidity_idx] = id;
       self.liquidity_position_size[next_liquidity_idx] = size;
       self.liquidity_position_rewards[next_liquidity_idx] = 0;
       self.last_liquidity_position_idx += 1; 

       self.active = true;

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
   pub fn remove_liquidity(&mut self, id: u64) -> Result<(),TickError> {
        let idx = self.find_liquidity_position_idx(id);
        if self.liquidity_position_rewards[idx] != 0 {
            return Err(TickError{cause: "rewards should be withdrawn".to_string()})
        }
        let liquidity_position_size = self.liquidity_position_size[idx];
        let current_idx = idx;
        
        let max_number_of_liquidity_positions_as_usize = (MAX_NUMBER_OF_LIQUIDITY_POSITIONS-1) as usize;
        while self.liquidity_position_idx[current_idx] != 0 && current_idx < max_number_of_liquidity_positions_as_usize{
            self.liquidity_position_idx[current_idx] = self.liquidity_position_idx[current_idx+1];
            self.liquidity_position_rewards[current_idx] = self.liquidity_position_rewards[current_idx+1];
            self.liquidity_position_size[current_idx] = self.liquidity_position_size[current_idx+1];
        }
        self.liquidity_position_idx[max_number_of_liquidity_positions_as_usize] = 0;
        self.liquidity_position_rewards[max_number_of_liquidity_positions_as_usize] = 0;
        self.liquidity_position_size[max_number_of_liquidity_positions_as_usize] =0;
        
        self.last_liquidity_position_idx -= 1;
        self.liquidity -= liquidity_position_size;
        
        if self.last_liquidity_position_idx == 0 {
            self.active=false;
        }

        
        Ok(())
   }


   /// Crank for updating rewards. 
   /// Assume that method is called on each change
   /// 
   /// # Arguments
   /// * Tick
   pub fn increase_rewards(&mut self) -> Result<(),TickError> {
    let mut cumulative_liquidity = 0;
    let mut idx = 0;
    while cumulative_liquidity < self.used_liquidity {
        self.liquidity_position_rewards[idx] += self.reward_calculations(self.liquidity_position_size[idx], 1.0)?;
        cumulative_liquidity += self.liquidity_position_size[idx];
        idx+=1;
    }
    let remaining_liquidity = self.used_liquidity-cumulative_liquidity;
    let current_liquidity_position = self.liquidity_position_size[idx] ;
    
    // Since liquidity is in lamports, ratio would be in 
    let last_lp_utilization = remaining_liquidity as f64/current_liquidity_position as f64; 
    self.liquidity_position_rewards[idx] += self.reward_calculations(self.liquidity_position_size[idx], last_lp_utilization)?;
   
    Ok(())
   }

   /// Get Rewards
   /// get rewards allows users to check the current reward 
   /// 
   /// # Arguments
   /// * Tick
   /// * id: The id in the liquidity position seed
   /// 
   pub fn get_rewards(&mut self,id: u64) -> Result<u64,TickError> {
        let idx = self.find_liquidity_position_idx(id);
        
        Ok(self.liquidity_position_rewards[idx])
   }

   pub fn withdraw_rewards(&mut self,id:u64) -> Result<(),TickError> {
        let idx = self.find_liquidity_position_idx(id);
        
        self.liquidity_position_rewards[idx] = 0;
        Ok(())
   }


   // ____________ Internal functions ___________________ // 
   /// Calculate reward for a liquidity position
   /// 
   /// # Arguments
   /// * Liquidity: the amount of liquidity, given with decimals. 1 token = 1*e^dec
   /// * Utilization: percentage the amount of liquidity use, [0,1]
   /// 
   fn reward_calculations(&self,liquidity: u64,utilization: f64) -> Result<u64,TickError> {
    let liquidity_f = liquidity as f64;
    let current_timestamp = self.get_unix_timestamp()?;
    let time_diff_seconds = current_timestamp-self.last_updated;
    let time_factor = time_diff_seconds as f64/SECONDS_IN_A_YEAR as f64;
    let tick_percentage = self.tick as f64/10_000.0;
    let reward = time_factor*tick_percentage*liquidity_f*utilization;
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
   fn find_liquidity_position_idx(&self, id: u64) -> usize {
       self.liquidity_position_idx.iter().position(|&idx| idx == id).unwrap()
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
   pub fn update_callback(&mut self) -> Result<(),TickError>{
       self.last_updated = self.get_unix_timestamp()?;
       Ok(())
   }

   /// Get Unix Timestamp
   /// Simple helper function to the the timestamp 
   /// from the solana runtime
   /// 
   /// # Arguments
   /// * self: Tick account
   /// 
   fn get_unix_timestamp(&self) -> Result<i64,TickError> {
    let last_updated = match Clock::get() {
        Ok(clock) => clock.unix_timestamp,
        Err(_) => return Err(TickError{cause:"could not get tick timestamp".to_string()})
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
   fn percentage_liquidity_used(&self,id: u64) -> Result<f64, TickError> {
       let mut cummulative_liquidity = 0;
       let mut idx = 0;
       while id != self.liquidity_position_idx[idx] {
           cummulative_liquidity+= self.liquidity_position_size[idx];
           idx+=1
       }

       let remaining_liquidity = self.used_liquidity - cummulative_liquidity;
       if remaining_liquidity > self.liquidity_position_size[idx]{
            return Ok(1.0);
       }
       Ok(remaining_liquidity as f64/self.liquidity_position_size[idx]as f64)
   }
}

#[cfg(test)]
mod tests {
    use std::time::{self, SystemTime};

    use super::*;

    #[test]
    fn add_liquidity() {
        // INitialize 
        let time = SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs() as i64;
        let init_liq_idx = [0;(MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize)];
        let init_liq_size =[0;(MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize)];
        let init_liq_rewards = [0;(MAX_NUMBER_OF_LIQUIDITY_POSITIONS as usize)];
        let last_liq= 0;
        let mut tick = Tick {
            bump: 1,
            liquidity: 0,
            used_liquidity: 0,
            last_updated: time,
            tick: 300,
            active: false,
            liquidity_position_idx: init_liq_idx,
            liquidity_position_rewards: init_liq_rewards,
            liquidity_position_size: init_liq_size,
            last_liquidity_position_idx: last_liq,
        };

        // Add liquidity 
        tick.add_liquidity(356, 1_000).unwrap();
        println!("liquidity pos: {:?}",tick.liquidity_position_size);
        assert_eq!(tick.last_liquidity_position_idx,1);
        assert_eq!(tick.liquidity,1_000);
        assert_eq!(tick.liquidity_position_size[0],1_000);
        assert_eq!(tick.liquidity_position_idx[0],356);
        assert_eq!(tick.liquidity_position_rewards[0],0);

        // Remove liquidity
        tick.remove_liquidity(356).unwrap();
        assert_eq!(tick.last_liquidity_position_idx,0);
        assert_eq!(tick.liquidity,0);
        assert_eq!(tick.liquidity_position_size[0],0);
        assert_eq!(tick.liquidity_position_idx[0],0);
        assert_eq!(tick.liquidity_position_rewards[0],0);
        assert_eq!(tick.active,false);


    }
}