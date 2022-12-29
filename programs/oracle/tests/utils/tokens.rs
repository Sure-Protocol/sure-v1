use anchor_client::solana_sdk::{self, signers::Signers};
use anchor_client::{solana_sdk::signer::Signer, *};
use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;
use solana_program::hash::Hash;
use solana_program::program_pack::Pack;
use solana_program::rent::Rent;

use solana_program_test::ProgramTestContext;

pub fn create_mint<T: Signer>(
    minter: &T,
    mint: &T,
    rent: Rent,
    recent_blockhash: Hash,
    decimals: u8,
) -> Result<solana_sdk::transaction::Transaction> {
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);
    let token_program = spl_token::id();
    let create_mint_account_ix = anchor_lang::solana_program::system_instruction::create_account(
        &minter.pubkey(), // from
        &mint.pubkey(),   // to account
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &token_program,
    );
    let create_mint_ix = spl_token::instruction::initialize_mint(
        &token_program.key(),
        &mint.pubkey(),
        &minter.pubkey(),
        Some(&minter.pubkey()),
        decimals,
    )
    .unwrap();

    // create mint transaction
    Ok(
        anchor_client::solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[create_mint_account_ix, create_mint_ix],
            Some(&minter.pubkey()),
            &[minter, mint],
            recent_blockhash,
        ),
    )
}

pub fn mint_amount<T: Signer>(
    minter: &T,
    mint: &T,
    ata: &Pubkey,
    recent_blockhash: &Hash,
    amount: u64,
) -> Result<solana_sdk::transaction::Transaction> {
    let mint_tokens_ix = spl_token::instruction::mint_to(
        &spl_token::ID,
        &mint.pubkey(),
        &ata,
        &minter.pubkey(),
        &[&minter.pubkey(), &mint.pubkey()],
        amount,
    )?;

    Ok(solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[mint_tokens_ix],
        Some(&minter.pubkey()),
        &[minter, mint],
        *recent_blockhash,
    ))
}

pub async fn test_amount_balance(
    ctx: &mut ProgramTestContext,
    wallet: &Pubkey,
    mint: &Pubkey,
    expected_amount: u64,
    tag: &str,
) {
    let account_balance = get_token_account_balance(ctx, wallet, mint).await.unwrap();
    assert!(
        account_balance > expected_amount,
        "[{:?}] account balance is less than amount: balance = {} < {} = amount to be locked",
        tag,
        account_balance,
        expected_amount
    );
}

const SPL_AMOUNT_OFFSET: usize = 32 + 32;
const U64_OFFSET: usize = 8;

/// get_token_account_balance
/// gets the account data from the associated token account and
/// uses offsets to find the account balance
pub async fn get_token_account_balance(
    ctx: &mut ProgramTestContext,
    wallet: &Pubkey,
    mint: &Pubkey,
) -> Result<u64> {
    let account_address = anchor_spl::associated_token::get_associated_token_address(wallet, mint);
    let account = ctx
        .banks_client
        .get_account(account_address)
        .await
        .unwrap()
        .unwrap();

    let account_data = account.data.as_slice();

    let amount: &u64 =
        bytemuck::from_bytes(&account_data[SPL_AMOUNT_OFFSET..SPL_AMOUNT_OFFSET + U64_OFFSET]);
    Ok(*amount)
}
