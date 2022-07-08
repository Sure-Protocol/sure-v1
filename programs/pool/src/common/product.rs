use super::errors::SureError;
use anchor_lang::prelude::*;
#[derive(PartialEq, Clone, Copy)]
pub enum ProductType {
    Coverage,
}

impl ProductType {
    pub fn get_product_type(product_id: u8) -> Result<Self> {
        if product_id == 1 {
            return Ok(ProductType::Coverage);
        } else {
            return Err(SureError::InvalidProductTypeId.into());
        }
    }

    pub fn is_smart_contract_coverage(self) -> bool {
        self == ProductType::Coverage
    }
}
