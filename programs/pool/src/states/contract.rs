///! Insurance contract representing the proof
///! that a user has insurance
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use crate::utils::errors::*;


const SURE_TIME_LOCK_IN_SECONDS: u64 = solana_program::clock::SECONDS_PER_DAY;

/// Pool insurance contract is meant to keep an
/// overview over the relationship a user 
/// has with the pool
#[account]
#[derive(Default)]
pub struct PoolInsuranceContract {
    /// The bump
    pub bump: u8, //1 byte

    /// Contract expiry
    pub end_time: i64, // 8 byte

    /// Owner of contract
    pub owner: Pubkey, // 8 byte

    /// Pool Insurance Tick Contracts
    pub insurance_contracts: Pubkey, // 8 bytes 
}

/// Insurance Contract for each tick
/// The account should be able to be reduced within a tick
#[account]
#[derive(Default)]
pub struct InsuranceContract {
    /// The bump identity of the PDA
    pub bump: u8, // 1 byte

    /// Amount insured
    pub insured_amount: u64, // 8 bytes

    /// Amount to be converted to insured amount
    /// over time
    pub time_locked_insured_amount: u64,

    /// Premium
    pub premium: u64, // 8 bytes

    /// The length of the contract
    pub period_ts: i64, // 8 bytes

    /// The end time of the contract
    pub end_ts: i64, // 8 bytes

    /// Start time of contract
    pub start_ts: i64, // 8 bytes

    /// End of timelocked insured amount
    pub time_lock_end: i64, // 8 bytes

    /// Insured pool
    pub pool: Pubkey, // 32 bytes

    /// Tick Account used to buy from
    pub tick_account: Pubkey, // 32 bytes

    // Token Mint
    pub token_mint: Pubkey, // 32 bytes

    /// Owner of insurance contract
    pub owner: Pubkey, // 32 bytes

    /// Is the insurance contract active
    pub active: bool, // 1 byte

    /// Updated 
    pub updated_ts: i64, // 8 bytes

    /// Created
    pub created_ts: i64, // 8 bytes
}

impl InsuranceContract {
    pub const SPACE: usize = 1 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 32 + 32 + 32 + 32 + 1 + 8+8;

    /// Calculate premium required to cover the 
    /// contract for the time period
    fn calculate_premium(&self,tick: u16,amount: u64,start_ts: i64,end_ts: i64) -> Result<u64>{
        msg!(&format!("Calculate Premium: tick: {}, amount: {}, start_ts:  {}, end_ts:{}",tick,amount,start_ts,end_ts));
        // Get the premium rate in decimal
        let premium_rate = (tick as f64) / 10000.0;

        // Get contract length 
        let contract_length = end_ts-start_ts;
        if end_ts <= start_ts {
            return Ok(0);
        }
        
        let seconds_per_year = (solana_program::clock::SECONDS_PER_DAY * 365) as f64;
        msg!(&format!("contract length {}, seconds per year {} ",contract_length as f64,seconds_per_year));
        let year_fraction = (contract_length as f64) / ((solana_program::clock::SECONDS_PER_DAY * 365) as f64);
        let premium = (amount as f64) * premium_rate * year_fraction;
        msg!(&format!("year fraction: {}, premium_rate: {}, premium: {}",year_fraction,premium_rate,premium));
        msg!(&format!("amount u64: {}",premium as u64));
        Ok(premium as u64)
    }

    /// Update premium
    /// Calculate the premium the user have to pay or get 
    /// refunded when they update their insurance position
    /// 
    /// The calculations assumes the remainder of the contract
    /// is voided and extended with the new one.
    /// 
    /// # Arguments
    /// * tick: the current tick used for the premium rate
    /// * new_insured_amount: The new amount to be insured
    /// * new_end_ts: the updated end time for the contract
    /// 
    /// # Returns Result<increasePremium,premiumIncrease>
    /// * increasePremium<bool>: Should premium be increased?
    /// * premiumDiff<u64>: the premium diff.
    fn increase_premium(&self,tick: u16,new_insured_amount: u64,new_end_ts: i64) -> Result<(bool,u64)> {
        let current_time = Clock::get()?.unix_timestamp;
        let remaining_premium = self.calculate_premium(tick,self.insured_amount,current_time,self.end_ts)?;
        let new_premium = self.calculate_premium(tick,new_insured_amount,current_time,new_end_ts)?;
        msg!(&format!("remaining_premium {}, new premium {}",remaining_premium,new_premium));
        if remaining_premium > new_premium {
            return Ok((false,(remaining_premium - new_premium)))
        }
        Ok((true,(new_premium-remaining_premium)))
    }


    /// Update insured amount and return the premium
    /// 
    /// # Arguments
    /// * tick: the current tick used for the premium rate
    /// * new_insured_amount: The new amount to be insured
    /// * new_end_ts: the updated end time for the contract
    /// 
    /// # Returns Result<increasePremium,premiumIncrease>
    /// * increasePremium<bool>: Should premium be increased?
    /// * premiumDiff<u64>: the premium diff.
    pub fn update_position_and_get_premium(&mut self,tick: u16,new_insured_amount: u64,new_end_ts:i64) -> Result<(bool,u64)> {
        let (increase_premium,premium) = self.increase_premium(tick, new_insured_amount,new_end_ts)?;
        
        let current_time = Clock::get()?.unix_timestamp;
        // Update insurance position
        let time_lock = false;
        if new_insured_amount > self.insured_amount && time_lock {
            // Time-locked insurance amount
            let amount_change = new_insured_amount - self.insured_amount;
            self.time_locked_insured_amount = amount_change;
            self.time_lock_end = current_time + (SURE_TIME_LOCK_IN_SECONDS as i64);
            self.insured_amount = new_insured_amount;
        }else {
            // Reduction happens immidiately
            self.insured_amount = new_insured_amount;
        }

        if increase_premium {
            self.premium = self.premium + premium
        }else {
            self.premium = self.premium - premium
        }
       
        self.end_ts = new_end_ts;
        self.updated_ts =current_time;
        self.period_ts = new_end_ts - current_time;
        
       
        return Ok((increase_premium,premium));
    }

    /// Crank to be used to update the 
    /// insured amount
    /// 
    /// # Arguments
    /// * tick: the current tick used for the premium rate
    /// * new_insured_amount: The new amount to be insured
    /// * new_end_ts: the updated end time for the contract
    /// 
    /// # Returns Result<increasePremium,premiumIncrease>
    /// * increasePremium<bool>: Should premium be increased?
    /// * premiumDiff<u64>: the premium diff.
    pub fn crank(&mut self) -> Result<()>{
        // let current_time = Clock::get()?.unix_timestamp;
        // let time_diff = current_time-self.updated_ts;
        // let elapsed_time 
        // self.updated_ts = current_time;
        Ok(())
    }
}

#[event]
pub struct ReduceInsuredAmountForTick {
    pub owner: Pubkey,
    pub tick: u16,
    pub updated_insured_amount: u64,
}
