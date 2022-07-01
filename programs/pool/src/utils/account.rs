use super::errors::SureError;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::TokenAccount;

/// Validate Token Account ownership
///
/// Check if the expected owner has admin rights over the
/// token account. It checks if the owner is a delegate for the
/// account then if the owner is token owner.
pub fn validate_token_account_ownership<'info>(
    token_account: &Account<'info, TokenAccount>,
    expected_owner: &Signer<'info>,
) -> Result<()> {
    match token_account.delegate {
        COption::Some(ref delegate) if expected_owner.key == delegate => {
            validate_owner(delegate, &expected_owner.to_account_info())?
        }
        _ => validate_owner(&token_account.owner, &token_account.to_account_info())?,
    };
    Ok(())
}

/// Validate owner
///
/// Simple check to see if the owner key in the token
/// account mathces that of the expected_owner
pub fn validate_owner(expected_owner: &Pubkey, owner_account_info: &AccountInfo) -> Result<()> {
    if expected_owner != owner_account_info.key || !owner_account_info.is_signer {
        return Err(SureError::InvalidOwner.into());
    }

    Ok(())
}
