use anchor_client::solana_sdk::{program_pack::Pack, signature::Keypair, signer::Signer, *};
use anchor_lang::prelude::*;
use oracle::id;
use solana_program_test::*;
use spl_token;

const SURE_ORACLE_CONFIG_SEED: &str = "";

#[tokio::test]
async fn create_and_init() {
    let mut program_test = ProgramTest::new("oracle", id(), processor!(oracle::entry));

    let oracle_owner = Keypair::new();
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // create token mint - sure
    let mint_account = Keypair::new();
    let minter = Keypair::new();
    let token_program = spl_token::id();
    let rent = banks_client.get_rent().await.unwrap();
    let mint_rent = rent.minimum_balance(spl_token::state::Mint::LEN);

    // create mint instruction
    let create_mint_account_ix = anchor_lang::solana_program::system_instruction::create_account(
        &minter.pubkey(),       // from
        &mint_account.pubkey(), // to account
        mint_rent,
        spl_token::state::Mint::LEN as u64,
        &token_program,
    );
    let create_mint_ix = spl_token::instruction::initialize_mint(
        &token_program.key(),
        &mint_account.pubkey(),
        &minter.pubkey(),
        Some(&minter.pubkey()),
        6,
    )
    .unwrap();

    // create mint transaction
    let token_mint_tx = anchor_client::solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_mint_account_ix, create_mint_ix],
        Some(&minter.pubkey()),
        &[&minter, &mint_account],
        recent_blockhash,
    );

    // sign and send transaction
    banks_client
        .process_transaction(token_mint_tx)
        .await
        .unwrap();

    let config_account = Pubkey::find_program_address(
        &[
            SURE_ORACLE_CONFIG_SEED.as_bytes().as_ref(),
            mint_account.pubkey().as_ref(),
        ],
        &oracle::id(),
    );

    // intialize config
    let config = oracle::accounts::InitializeConfig {
        signer: oracle_owner.pubkey(),
        config: config_account.0.key(),
        token_mint: mint_account.pubkey(),
        system_program: anchor_lang::system_program::ID,
    };

    //
}
