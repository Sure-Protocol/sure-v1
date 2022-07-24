use crate::common::errors::SureError;
use anchor_lang::{prelude::*, Result};

#[account]
pub struct FeePackage {
    pub bump: u8, // 1 byte

    // Owner of fee package
    pub owner: Pubkey, //32

    /// Total fee the protocol will charge per tx as
    /// 100th of a basis point i.e 0.0001%
    pub fee_rate: u16, // 2 bytes

    /// Fees that accrue the protocol
    /// (1/x) of the total fee will go to the protocol
    pub protocol_fee_rate: u16, // 2 bytes

    /// Fees that accrue the founder of the pool
    /// (1/x) of the total fee will go to the protocol
    pub founders_fee_rate: u16, // 2 bytes
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
        self.validate_fee_rates()?;

        // Validate owner

        self.update_fee_package(fee_rate, protocol_fee_rate, founders_fee_rate);
        self.owner = owner.key.clone();

        Ok(())
    }

    pub fn update_fee_package(
        &mut self,
        fee_rate: u16,
        protocol_fee_rate: u16,
        founders_fee_rate: u16,
    ) {
        self.fee_rate = fee_rate;
        self.protocol_fee_rate = protocol_fee_rate;
        self.founders_fee_rate = founders_fee_rate;
    }

    pub fn validate_fee_rates(&self) -> Result<()> {
        if self.fee_rate > FeePackage::MAX_FEE_RATE_BP {
            return Err(SureError::MaxFeeRateExceeded.into());
        }

        // 1/(x+y) <= 1 <=> x+y >= 1
        if self.protocol_fee_rate + self.founders_fee_rate >= 1 {
            return Err(SureError::InvalidSubFeeRates.into());
        }

        Ok(())
    }
}

#[cfg(tess)]
mod fee_state_test {
    use super::*;
}
