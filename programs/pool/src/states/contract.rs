///! Insurance contract representing the proof
///! that a user has insurance
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use crate::BitMap;

use crate::states::{
    pool::{PoolAccount,},
    seeds::{SURE_INSURANCE_CONTRACTS_BITMAP,SURE_INSURANCE_CONTRACTS_INFO,SURE_INSURANCE_CONTRACT,SURE_INSURANCE_CONTRACTS},
    tick::Tick,
};

use anchor_spl::{
    token::{Mint, Token, TokenAccount},
};

use vipers::{assert_is_ata, prelude::*};


const SURE_TIME_LOCK_IN_SECONDS: u64 = solana_program::clock::SECONDS_PER_DAY;

/// --- Insurance Contracts ----
/// <POOLS>
/// Holds information about the contracts held by the 
/// user 
#[account]
pub struct InsuranceContracts{
    /// the bump
    pub bump: u8, // 1 byte

    /// owner of account
    pub owner: Pubkey,

    /// Vec of Pool PubKeys
    pub pools: Vec<Pubkey>, // 4 + 32*256 = 8196, 256 insured contracts
}

impl InsuranceContracts {
    pub const SPACE: usize = 1 + 32 + 4 + 32 * 256;
}

/// --- Pool insurance contract ---
/// <POOL>
/// Accumulation of all insurance contracts for a user in  
/// a given pool.
#[account]
pub struct PoolInsuranceContract {
    /// The bump
    pub bump: u8, //1 byte

    /// Contract expiry
    pub expiry_ts: i64, // 8 byte

    /// Contract Amount
    pub insured_amount: u64, // 8 byte

    /// token mint
    pub token_mint: Pubkey, 
    
    /// Owner of contract
    pub owner: Pubkey, // 32 byte
}

impl PoolInsuranceContract{
    pub const SPACE: usize = 1 + 8 + 8 + 32 + 32;
}

/// --- Insurance Contract -- 
/// <TICK>
/// Holds state about an insurance contract for a specific tick
#[account]
#[derive(Default)]
pub struct InsuranceTickContract {
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
    pub liquidity_tick_info: Pubkey, // 32 bytes

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

impl InsuranceTickContract {
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
            
        }else {
            // Reduction happens immidiately
        }

        if increase_premium {
            self.premium = self.premium + premium
        }else {
            self.premium = self.premium - premium
        }
       
        self.end_ts = new_end_ts;
        self.updated_ts =current_time;
        self.period_ts = new_end_ts - current_time;
        self.insured_amount = new_insured_amount;
        
       
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

// ?---------------------------------.----------------- //
// ?%%%%%%%%%%%%%%%% Method Accounts %%%%%%%%%%%%%%%%! //
// ?-------------------------------------------------- //

/// --- Initialize Policy Holder ---
/// 
/// Prepare a new user for being able to buy insurance 
/// 
#[derive(Accounts)]
pub struct InitializePolicyHolder<'info> {
    /// Signer - the new users
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Insurance Contracts
    #[account(
        init,
        space = 8 + InsuranceContracts::SPACE,
        payer = signer,
        seeds = [
            SURE_INSURANCE_CONTRACTS.as_bytes(),
            signer.key().as_ref(),
        ],
        bump,
    )]
    pub insurance_contracts: Box<Account<'info,InsuranceContracts>>,

    /// System program
    pub system_program: Program<'info, System>,
}


/// --- Initialize User Insurance Contract --- 
/// 
/// Accounts used to initialize a user for a new pool
/// 
/// seeds: [
///     signer,
///     pool
///     token_mint
/// ] 
#[derive(Accounts)]
pub struct InitializeUserPoolInsuranceContract<'info> {
    /// Signer
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Pool associated with the insurance contracts
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Token mint used for the insurance contracts
    pub token_mint: Box<Account<'info, Mint>>,

    /// Insurance contracts keeping an overview over 
    /// Pools held by user
    #[account(mut)]
    pub insurance_contracts: Box<Account<'info,InsuranceContracts>>,

    /// Insurance Contracts Bitmap
    /// Bitmap for identifying for which ticks the user's 
    /// insurance contract is located
    #[account(
        init,
        space = 8 + BitMap::SPACE,
        payer = signer,
        seeds = [
            SURE_INSURANCE_CONTRACTS_BITMAP.as_bytes(),
            signer.key().as_ref(),
            pool.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_insurance_contract_bitmap: Box<Account<'info,BitMap>>,
    
    /// Insurance pool contract info
    /// Holds aggregate information on all the 
    /// insurance contracts for a given user 
    #[account(
        init,
        space = 8 + PoolInsuranceContract::SPACE,
        payer = signer,
        seeds = [
            SURE_INSURANCE_CONTRACTS_INFO.as_bytes(),
            signer.key().as_ref(),
            pool.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub pool_insurance_contract_info: Box<Account<'info, PoolInsuranceContract>>,

    /// System program
    pub system_program: Program<'info, System>,
}
/// --- Initialize Insurance Contract ---
/// 
/// Initializes an insurance contract for a specific tick 
/// account. 
/// 
/// side effects: 
/// Pool contracts is updated:
///     - Info
///     - Bitmap overview 
/// 
#[derive(Accounts)]
pub struct InitializeInsuranceContract<'info> {
    /// Signer of contract
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Pool to buy insurance from
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Token mint used to insure with
    pub token_mint: Box<Account<'info, Mint>>,

    /// Tick account to insure against
    pub liquidity_tick_info: AccountLoader<'info, Tick>,

    /// Insurance Contract
    #[account(
        init,
        space = 8 + InsuranceTickContract::SPACE,
        payer = owner,
        seeds = [
            SURE_INSURANCE_CONTRACT.as_bytes(),
            owner.key().as_ref(),
            liquidity_tick_info.key().as_ref(),
        ],
        bump,
    )]
    pub insurance_tick_contract: Box<Account<'info, InsuranceTickContract>>,

    /// Insurance Contracts 
    #[account(mut)]
    pub pool_insurance_contract_info: Box<Account<'info, PoolInsuranceContract>>,

    #[account(mut)]
    pub pool_insurance_contract_bitmap: Box<Account<'info,BitMap>>,

    /// System Contract used to create accounts
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for InitializeInsuranceContract<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}


/// --- Update Insurance Tick Contract --- 
///
/// Updates the insurance contract for the given tick account
/// 
/// Method adjust the position and expiry and calculates the 
/// new premium that has to be refunded or paid
#[derive(Accounts)]
pub struct UpdateInsuranceTickContract<'info> {
    /// Buyer
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// Account to buy tokens with
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Pool to buy from
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Tick account to buy
    #[account(mut)]
    pub liquidity_tick_info: AccountLoader<'info, Tick>,

    /// Premium Vault
    #[account(
        mut,
        constraint = premium_vault.owner ==  pool.key(),
        constraint = premium_vault.mint == token_account.mint,
    )]
    pub premium_vault: Box<Account<'info, TokenAccount>>,

    /// Insurance Contract
    #[account(mut,
    constraint = insurance_tick_contract.pool == pool.key(),
    constraint = insurance_tick_contract.owner == buyer.key(),
    )]
    pub insurance_tick_contract: Box<Account<'info, InsuranceTickContract>>,

    /// Insurance Contracts 
    #[account(mut)]
    pub pool_insurance_contract_info: Box<Account<'info, PoolInsuranceContract>>,

    /// Token program, needed to transfer tokens
    pub token_program: Program<'info, Token>,

    /// System Contract used to create accounts
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for UpdateInsuranceTickContract<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

// ?----------------------------------------- //
// ?%%%%%%%%%%%%%%%% Events %%%%%%%%%%%%%%%%! //
// ?----------------------------------------- //
#[event]
pub struct ReduceInsuredAmountForTick {
    pub owner: Pubkey,
    pub tick: u16,
    pub updated_insured_amount: u64,
}

#[event]
pub struct InitializePolicyHolderEvent{
    pub owner: Pubkey,
}
