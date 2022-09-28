use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_spl::token::spl_token::instruction::AuthorityType;
use anchor_spl::token::{
    burn, close_account, mint_to, set_authority, transfer, Burn, CloseAccount, Mint, MintTo,
    SetAuthority, Token, TokenAccount, Transfer,
};
use mpl_token_metadata::{
    instruction::{create_metadata_accounts_v2, update_metadata_accounts_v2},
    state::DataV2,
};

/// Allows for easy generation of seeds
pub trait Seeds {
    fn seeds(&self) -> Box<[&[u8]]>;
}

/// deposit into an vault (account)
///
/// helper to transfer from an origin user account
/// into a vault
pub fn deposit_into_vault<'info>(
    user: &Signer<'info>,
    vault: &Account<'info, TokenAccount>,
    origin_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: origin_account.to_account_info(),
                to: vault.to_account_info(),
                authority: user.to_account_info(),
            },
        ),
        amount,
    )
}

/// withdraw from a vault (account)
///
/// assume the owner is an account
pub fn withdraw_from_vault<
    'info,
    T: AccountSerialize + AccountDeserialize + Seeds + Clone + Owner,
>(
    authority: &Account<'info, T>,
    vault: &Account<'info, TokenAccount>,
    destination_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    amount: u64,
) -> Result<()> {
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: vault.to_account_info(),
                to: destination_account.to_account_info(),
                authority: authority.to_account_info(),
            },
            &[&authority.seeds()],
        ),
        amount,
    )
}

/// burn an nft
///
/// assume nft is owned by an account whic
pub fn burn_nft<'info, T: AccountSerialize + AccountDeserialize + Seeds + Clone + Owner>(
    authority: &Account<'info, T>,
    destination: &UncheckedAccount<'info>,
    position_mint_account: &Account<'info, TokenAccount>,
    position_mint: &Account<'info, Mint>,
    token_authority: &Signer<'info>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    // Burn liquidity Position NFT
    burn(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Burn {
                mint: position_mint.to_account_info(),
                from: position_mint_account.to_account_info(),
                authority: token_authority.to_account_info(),
            },
            &[&authority.seeds()],
        ),
        1,
    )?;

    // Close token account
    close_account(CpiContext::new(
        token_program.to_account_info(),
        CloseAccount {
            account: position_mint_account.to_account_info(),
            destination: destination.to_account_info(),
            authority: token_authority.to_account_info(),
        },
    ))
}

/// mint an nft
///
/// make an account authority
pub fn mint_nft<'info, T: AccountSerialize + AccountDeserialize + Seeds + Clone + Owner>(
    authority: &Account<'info, T>,
    position_mint_account: &Account<'info, TokenAccount>,
    position_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    mint_to(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            MintTo {
                mint: position_mint.to_account_info(),
                to: position_mint_account.to_account_info(),
                authority: authority.to_account_info(),
            },
            &[&authority.seeds()],
        ),
        1,
    )
}

/// attach metaplex date to an NFT
pub fn attach_mp_metadata<
    'info,
    T: AccountSerialize + AccountDeserialize + Seeds + Clone + Owner,
>(
    authority: &Account<'info, T>,
    user: &Signer<'info>,
    name: &str,
    symbol: &str,
    uri: &str,
    metadata_account: &UncheckedAccount<'info>,
    metadata_program: &UncheckedAccount<'info>,
    metadata_update_auth: AccountInfo<'info>,
    position_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
) -> Result<()> {
    let create_metadata_accounts_ix = create_metadata_accounts_v2(
        metadata_program.key(),
        metadata_account.key(),
        position_mint.key(),
        authority.key(),
        user.key(),
        metadata_update_auth.key(),
        String::from(name),
        String::from(symbol),
        String::from(uri),
        None,
        0,
        true,
        true,
        None,
        None,
    );

    // make cpi
    process_metaplex_update(
        authority,
        user,
        create_metadata_accounts_ix,
        metadata_account,
        metadata_program,
        metadata_update_auth,
        position_mint,
        system_program,
        rent,
    )?;

    remove_mint_authority(authority, position_mint, token_program)
}

/// Create Liquidity Position with Metadata
///
/// creates a metadata account for the given
/// position mint
/// TODO: Accept accountInfo instead of UncheckedAccounts
pub fn create_nft_with_metadata<
    'info,
    T: AccountSerialize + AccountDeserialize + Seeds + Clone + Owner,
>(
    authority: &Account<'info, T>,
    user: &Signer<'info>,
    name: &str,
    symbol: &str,
    uri: &str,
    metadata_account: &UncheckedAccount<'info>,
    metadata_program: &UncheckedAccount<'info>,
    metadata_update_auth: AccountInfo<'info>,
    position_mint: &Account<'info, Mint>,
    position_token_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
) -> Result<()> {
    // Mint position
    mint_nft(
        authority,
        position_token_account,
        position_mint,
        token_program,
    )?;

    attach_mp_metadata(
        authority,
        user,
        name,
        symbol,
        uri,
        metadata_account,
        metadata_program,
        metadata_update_auth,
        position_mint,
        token_program,
        system_program,
        rent,
    )
}

/// Update the liquidity position NFT
/// If the user changes the liquidity position the
/// corrensponding NFT should also be updated
pub fn update_nft_metadata<
    'info,
    T: AccountSerialize + AccountDeserialize + Seeds + Clone + Owner,
>(
    authority: &Account<'info, T>,
    user: &Signer<'info>,
    metadata_account: &UncheckedAccount<'info>,
    metadata_program: &UncheckedAccount<'info>,
    metadata_update_auth: AccountInfo<'info>,
    position_mint: &Account<'info, Mint>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
) -> Result<()> {
    let datav2 = DataV2 {
        name: String::from("SURE LP V1 NFT"),
        symbol: String::from("SURE-LP"),
        uri: format!("https://arweave/some_id"),
        creators: None,
        seller_fee_basis_points: 0,
        collection: None,
        uses: None,
    };

    let update_metadata_account = update_metadata_accounts_v2(
        metadata_program.key(),
        metadata_account.key(),
        metadata_update_auth.key(),
        None,
        Some(datav2),
        None,
        None,
    );

    process_metaplex_update(
        authority,
        user,
        update_metadata_account,
        metadata_account,
        metadata_program,
        metadata_update_auth,
        position_mint,
        system_program,
        rent,
    )
}

pub fn process_metaplex_update<
    'info,
    T: AccountSerialize + AccountDeserialize + Seeds + Clone + Owner,
>(
    authority: &Account<'info, T>,
    user: &Signer<'info>,
    instruction: Instruction,
    metadata_account: &UncheckedAccount<'info>,
    metadata_program: &UncheckedAccount<'info>,
    metadata_update_auth: AccountInfo<'info>,
    position_mint: &Account<'info, Mint>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
) -> Result<()> {
    solana_program::program::invoke_signed(
        &instruction,
        &[
            metadata_account.to_account_info().clone(),
            position_mint.to_account_info().clone(),
            authority.to_account_info().clone(),
            metadata_update_auth.clone(),
            user.to_account_info().clone(),
            metadata_program.to_account_info().clone(),
            system_program.to_account_info().clone(),
            rent.to_account_info().clone(),
        ],
        &[&authority.seeds()],
    )?;
    Ok(())
}

/// remove mint authority
///
/// by removing the mint authority nobody can mint new
/// tokens
pub fn remove_mint_authority<
    'info,
    T: AccountSerialize + AccountDeserialize + Seeds + Clone + Owner,
>(
    authority: &Account<'info, T>,
    position_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    set_authority(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            SetAuthority {
                current_authority: authority.to_account_info(),
                account_or_mint: position_mint.to_account_info(),
            },
            &[&authority.seeds()],
        ),
        AuthorityType::MintTokens,
        Option::None,
    )
}
