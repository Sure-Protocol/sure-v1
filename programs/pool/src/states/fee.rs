use crate::utils::errors::SureError;
use anchor_lang::{prelude::*, Result};

#[account]
pub struct FeePackage {
    /// Total fee the protocol will charge per
    /// tx
    pub fee_rate: u16,

    /// Total fee that will accrue the protocol
    /// for every tx
    pub protocol_fee_rate: u16,

    /// Total fee rate that will accrue the founder of
    /// the pool at every tx
    pub founders_fee_rate: u16,
}

impl FeePackage {
    pub const MAX_FEE_RATE_bp: u16 = 10_000;
    pub const MAX_PROTOCOL_FEE_RATE_bp: u16 = 3_200;
    pub const MAX_FOUNDERS_FEE_RATE_bp: u16 = 1_000;
    pub const MIN_LIQUIDITY_PROVIDER_FEE_RATE_bp: u16 = 1_000;
    pub fn initialize(
        &mut self,
        fee_rate: u16,
        protocol_fee_rate: u16,
        founders_fee_rate: u16,
    ) -> Result<()> {
        self.validate_fee_rates()?;
        self.fee_rate = fee_rate;
        self.protocol_fee_rate = protocol_fee_rate;
        self.founders_fee_rate = founders_fee_rate;

        Ok(())
    }

    pub fn validate_fee_rates(&self) -> Result<()> {
        if self.fee_rate > FeePackage::MAX_FEE_RATE_bp {
            return Err(SureError::MaxFeeRateExceeded.into());
        }

        if self.fee_rate > FeePackage::MAX_PROTOCOL_FEE_RATE_bp {
            return Err(SureError::MaxProtocolFeeRateExceeded.into());
        }

        if self.protocol_fee_rate + self.founders_fee_rate > self.fee_rate {
            return Err(SureError::InvalidSubFeeRates.into());
        }
        let liquidity_provider_fee_rate =
            self.fee_rate - self.protocol_fee_rate - self.founders_fee_rate;
        if liquidity_provider_fee_rate < FeePackage::MIN_LIQUIDITY_PROVIDER_FEE_RATE_bp {
            return Err(SureError::TooLowLiquidityProviderFeeRate.into());
        }
        if self.founders_fee_rate > FeePackage::MAX_FOUNDERS_FEE_RATE_bp {
            return Err(SureError::MaxFoundersFeeRateExceeded.into());
        }

        Ok(())
    }
}
