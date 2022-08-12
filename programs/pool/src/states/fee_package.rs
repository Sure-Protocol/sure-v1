use crate::utils::errors::*;
use anchor_lang::prelude::*;

#[account]
pub struct FeePackage {
    pub bump: u8,
    pub owner: Pubkey,
    pub fee_rate: u16,
    pub founders_fee: u16,
    pub protocol_fee: u16,
}

impl FeePackage {
    pub const MAX_FEE_RATE_BP: u16 = 10_000;
    pub const MAX_PROTOCOL_FEE_RATE_BP: u16 = 3_200;
    pub const MAX_FOUNDERS_FEE_RATE_BP: u16 = 1_000;
    pub const MIN_LIQUIDITY_PROVIDER_FEE_RATE_BP: u16 = 1_000;
    pub const SIZE: usize = 1 + 32 + 2 + 2 + 2;

    pub fn initialize<'info>(
        &mut self,
        owner: &Signer<'info>,
        fee_rate: u16,
        protocol_fee_rate: u16,
        founders_fee_rate: u16,
    ) -> Result<()> {
        self.update_fee_package(fee_rate, protocol_fee_rate, founders_fee_rate);
        self.validate_fee_rates()?;
        self.owner = owner.key.clone();

        Ok(())
    }

    pub fn update_fee_package(&mut self, fee_rate: u16, protocol_fee: u16, founders_fee: u16) {
        self.fee_rate = fee_rate;
        self.protocol_fee = protocol_fee;
        self.founders_fee = founders_fee;
    }

    pub fn validate_fee_rates(&self) -> Result<()> {
        if self.fee_rate > FeePackage::MAX_FEE_RATE_BP {
            return Err(SureError::MaxFeeRateExceeded.into());
        }

        // 1/x + 1/y <= 1 <=> (x+y) <= xy
        if self.protocol_fee + self.founders_fee > self.protocol_fee * self.founders_fee {
            return Err(SureError::InvalidSubFeeRates.into());
        }

        Ok(())
    }
}
