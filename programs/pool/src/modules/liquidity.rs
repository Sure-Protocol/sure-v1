use anchor_lang::{prelude::*};
use anchor_spl::*;

/// Basic struct containing all data
/// necessary to create new liquidity position
pub struct ProvideLiquidity<'info> {
    pub token_program: AccountInfo<'info>,
    pub nft_mint: AccountInfo<'info>,
    pub nft_account: AccountInfo<'info>,
    pub nft_owner: AccountInfo<'info>,
    pub protocol_owner_bump: u8,

    pub vault: AccountInfo<'info>,
}

impl ProvideLiquidity<'_> {
    pub fn validate(&self) -> Result<()> {


        Ok(())
    }

    pub fn create_liquidity_position(&self,amount:u64) -> Result<()> {
        self.mint_liquidity_nft()?;
        Ok(())
    }

    pub fn mint_liquidity_nft(&self) -> Result<()> {
        token::mint_to(
            CpiContext::new_with_signer(
                self.token_program.clone()
                , token::MintTo {
                    mint: self.nft_mint.clone(),
                    to: self.nft_account.clone(),
                    authority: self.nft_owner.clone()
                },
                &[&[&[self.protocol_owner_bump] as &[u8]]])
            , 1)?;

            // TODO: Add metaplex data 
            Ok(())
    }

    pub fn supply_liquidity(&self, amount: u64) -> Result<()> {
        token::transfer(
            CpiContext::new(
                self.token_program.clone(),
                 token::Transfer{
                     from:self.nft_owner.clone(),
                     to:   self.vault.clone(),
                     authority: self.nft_owner.clone(),
                 }
                ),
        amount)?;

        Ok(())
    }

    pub fn update_liquidity_state(&self,amount:u64) -> Result<()> {
        Ok(())
    }

    pub fn redeem_liquidity(&self,amount: u64) -> Result<()> {
        Ok(())
    }




}