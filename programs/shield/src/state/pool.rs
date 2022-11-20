use crate::utils::SURE_SHIELD;
use anchor_lang::prelude::*;
use sure_common::token::Seeds;

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

    // mint of vault holding collateral
    //pub token_mint: Pubkey, // 32 bytes
    pub vault: Pubkey, //32 bytes

    // serum data
    pub orderbook_market: Pubkey,
    pub event_queue: Pubkey,
    pub asks: Pubkey,
    pub bids: Pubkey,

    /// Sure underlying prediction market
    pub sure_market: u64,
}

impl Seeds for Pool {
    fn seeds(&self) -> Box<[&[u8]]> {
        Box::new([
            &SURE_SHIELD.as_bytes() as &[u8],
            self.orderbook_market.as_ref(),
            self.bump_array.as_ref(),
        ])
    }
}

impl Pool {
    pub const SPACE: usize = 0;

    /// Initilize pool state
    ///
    pub fn initialize(
        &mut self,
        bump: &u8,
        name: &str,
        founder: &Pubkey,
        vault: &Pubkey,
        orderbook_market: &Pubkey,
        event_queue: &Pubkey,
        asks: &Pubkey,
        bids: &Pubkey,
        sure_market: &u64,
    ) {
        self.bump = *bump;
        self.bump_array = [*bump; 1];

        self.name = String::from(name);
        self.founder = *founder;
        self.fee_rate = 100;
        self.protocol_fee = 2;
        self.founders_fee = 2;
        self.insurance_fee = 50;

        self.vault = *vault;

        // set serum data
        self.orderbook_market = *orderbook_market;
        self.event_queue = *event_queue;
        self.bids = *bids;
        self.asks = *asks;
        self.sure_market = *sure_market
    }
}
