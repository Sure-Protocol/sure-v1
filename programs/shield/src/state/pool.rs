use crate::utils::SURE_SHIELD;
use anchor_lang::prelude::*;

#[account]
pub struct Pool {
    pub bump: u8,
    pub bump_array: [u8; 1],

    pub name: String,

    pub founder: Pubkey,

    // fee rates in basis points
    pub fee_rate: u16,
    /// (1/x)% of fee_rate
    pub protocol_fee: u16,
    pub founders_fee: u16,

    /// Fee paid when buying insurance.
    /// in basis points
    pub insurance_fee: u16, // 4 bytes

    /// The public key of the smart contract that is
    /// insured
    pub smart_contract: Pubkey, // 32 bytes

    // mint of vault holding collateral
    pub token_mint: Pubkey, // 32 bytes
    pub vault: Pubkey,      //32 bytes

    // serum data
    pub market_orderbook: Pubkey,
    pub event_queue: Pubkey,
    pub asks: Pubkey,
    pub bids: Pubkey,
}

impl Pool {
    pub const SPACE: usize = 0;

    pub fn seeds(&self) -> [&[u8]; 3] {
        [
            &SURE_SHIELD.as_bytes() as &[u8],
            self.smart_contract.as_ref(),
            self.bump_array.as_ref(),
        ]
    }

    /// Initilize pool state
    ///
    pub fn initialize(
        &mut self,
        bump: u8,
        name: &str,
        founder: &Pubkey,
        smart_contract: &Pubkey,
        token_mint: &Pubkey,
        vault: &Pubkey,
        market_orderbook: &Pubkey,
        event_queue: &Pubkey,
        asks: &Pubkey,
        bids: &Pubkey,
    ) {
        self.bump = bump;
        self.bump_array = [bump; 1];

        self.name = String::from(name);
        self.founder = *founder;
        self.fee_rate = 100;
        self.protocol_fee = 2;
        self.founders_fee = 2;
        self.insurance_fee = 50;

        self.smart_contract = *smart_contract;
        self.token_mint = *token_mint;
        self.vault = *vault;

        // set serum data
        self.market_orderbook = *market_orderbook;
        self.event_queue = *event_queue;
        self.bids = *bids;
        self.asks = *asks;
    }
}
