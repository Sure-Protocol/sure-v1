use crate::states::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_lang::solana_program::entrypoint::ProgramResult;
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

/// Deposit into vault
/// Helper to transfer from an origin user account
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

/// Withdraw from a vault
pub fn withdraw_from_vault<'info>(
    pool: &Account<'info, Pool>,
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
                authority: pool.to_account_info(),
            },
            &[&pool.seeds()],
        ),
        amount,
    )
}

/// Burn the NFT and close the nft token account
pub fn burn_liquidity_position_nft<'info>(
    pool: &Account<'info, Pool>,
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
            &[],
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

pub fn mint_nft<'info>(
    pool: &Account<'info, Pool>,
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
                authority: pool.to_account_info(),
            },
            &[&pool.seeds()],
        ),
        1,
    )
}

pub fn create_liquidity_position_with_metadata<'info>(
    metadata_account: &UncheckedAccount<'info>,
    metadata_program: &UncheckedAccount<'info>,
    metadata_update_auth: &UncheckedAccount<'info>,
    pool: &Account<'info, Pool>,
    liquidity_provider: &Signer<'info>,
    position_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
) -> Result<()> {
    let create_metadata_accounts_ix = create_metadata_accounts_v2(
        metadata_program.key(),
        metadata_account.key(),
        position_mint.key(),
        pool.key(),
        liquidity_provider.key(),
        metadata_update_auth.key(),
        String::from("Sure LP NFT V1"),
        String::from("SURE-LP"),
        format!("https://sure.claims"),
        None,
        0,
        true,
        true,
        None,
        None,
    );

    // Protocol owner signs the transaction with seeds
    // and bump
    solana_program::program::invoke_signed(
        &create_metadata_accounts_ix,
        &[
            metadata_account.to_account_info().clone(),
            position_mint.to_account_info().clone(),
            pool.to_account_info().clone(),
            metadata_update_auth.to_account_info().clone(),
            liquidity_provider.to_account_info().clone(),
            metadata_program.to_account_info().clone(),
            system_program.to_account_info().clone(),
            rent.to_account_info().clone(),
        ],
        &[&pool.seeds()],
    )?;

    remove_liquidity_position_authority(pool, position_mint, token_program)
}

/// Update the liquidity position NFT
/// If the user changes the liquidity position the
/// corrensponding NFT should also be updated
pub fn update_liquidity_position_with_metadata<'info>(
    metadata_account: &UncheckedAccount<'info>,
    metadata_program: &UncheckedAccount<'info>,
    metadata_update_auth: &UncheckedAccount<'info>,
    pool: &Account<'info, Pool>,
    liquidity_provider: &Signer<'info>,
    position_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
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
        update_metadata_account,
        metadata_account,
        metadata_program,
        metadata_update_auth,
        pool,
        liquidity_provider,
        position_mint,
        token_program,
        system_program,
        rent,
    )
}

pub fn process_metaplex_update<'info>(
    instruction: Instruction,
    metadata_account: &UncheckedAccount<'info>,
    metadata_program: &UncheckedAccount<'info>,
    metadata_update_auth: &UncheckedAccount<'info>,
    pool: &Account<'info, Pool>,
    liquidity_provider: &Signer<'info>,
    position_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
) -> Result<()> {
    solana_program::program::invoke_signed(
        &instruction,
        &[
            metadata_account.to_account_info().clone(),
            position_mint.to_account_info().clone(),
            pool.to_account_info().clone(),
            metadata_update_auth.to_account_info().clone(),
            liquidity_provider.to_account_info().clone(),
            metadata_program.to_account_info().clone(),
            system_program.to_account_info().clone(),
            rent.to_account_info().clone(),
        ],
        &[&pool.seeds()],
    )?;
    Ok(())
}

pub fn remove_liquidity_position_authority<'info>(
    pool: &Account<'info, Pool>,
    position_mint: &Account<'info, Mint>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    set_authority(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            SetAuthority {
                current_authority: pool.to_account_info(),
                account_or_mint: position_mint.to_account_info(),
            },
            &[&pool.seeds()],
        ),
        AuthorityType::MintTokens,
        Option::None,
    )
}
