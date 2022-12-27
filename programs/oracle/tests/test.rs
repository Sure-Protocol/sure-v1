//! A huge part about the locker has to do with time. The oracle is using external programs like
//! locker_voter where user can deposit sure tokens for a duration. The tuple (duration,amount) decides the
//! voting power.
//! For the oracle itself it is necessary with timeouts that allow governance holders to vote on future or past events.
//! It is therefore necessary in an integration context to be able to control time. Unfortunately, this is not possible
//! through the regular solana web3.js library for typescript. Therefore, we resort to writing integration tests that targets
//! test-bpf.
//!
//! As a side effect, by seperating logic and usage of programs we can create small wrapper rust sdks on top of existing auto
//! generated crates (especially anchor ones).
//!
//! There will mainly be one huge integration test that will follow the lifetime of an entire proposal. This will include multiple
//! actors. Thus, we are able to test the logic properly.

pub mod utils;

use anchor_client::{solana_sdk::signer::Signer, *};
use anchor_lang::prelude::*;
use anchor_lang::*;
use govern;
use locked_voter;
use oracle::id;
use oracle::utils::SURE_ORACLE_CONFIG_SEED;
use smart_wallet;
use solana_program::clock::SECONDS_PER_DAY;
use solana_program_test::*;
use solana_sdk::*;
use spl_token;
use utils::locker::*;
use utils::tokens::*;

/// initialize_account tops up the given address with enough sol to
/// work in the context of the integration test
fn initialize_account(address: &Pubkey, program_test: &mut ProgramTest) {
    program_test.add_account(
        *address,
        account::Account {
            lamports: 1_000_000_000,
            ..account::Account::default()
        },
    );
}

/// add_necessary_programs adds programs to the the test context that the
/// sure program interacts with
pub fn add_necessary_programs(ctx: &mut ProgramTest) {
    ctx.add_program("smart_wallet", smart_wallet::id(), None);
    ctx.add_program("govern", govern::id(), None);
    ctx.add_program("locked_voter", locked_voter::id(), None);
}

/// Main integration test for the oracle / prediction market
#[tokio::test]
async fn create_and_init() {
    let mut program_test = ProgramTest::new("oracle", id(), processor!(oracle::entry));

    let protocol_owner = signature::Keypair::new();
    initialize_account(&protocol_owner.pubkey(), &mut program_test);
    // create program oracle owner
    let oracle_owner = signature::Keypair::new();
    initialize_account(&oracle_owner.pubkey(), &mut program_test);
    // create minter
    let minter = signature::Keypair::new();
    initialize_account(&minter.pubkey(), &mut program_test);

    let proposer = signature::Keypair::new();
    initialize_account(&proposer.pubkey(), &mut program_test);

    // add_necessary_programs
    add_necessary_programs(&mut program_test);

    // start program
    let mut program_test_context = program_test.start_with_context().await;
    let rent = program_test_context.banks_client.get_rent().await.unwrap();

    // create mint instruction
    let mint = solana_sdk::signature::Keypair::new();

    let mint_tx = create_mint(&minter, &mint, rent, program_test_context.last_blockhash).unwrap();
    // sign and send transaction
    program_test_context
        .banks_client
        .process_transaction(mint_tx)
        .await
        .unwrap();

    //  setup locker
    let locker_result =
        setup_sure_locker(&mut program_test_context, &protocol_owner, &mint.pubkey())
            .await
            .unwrap();

    // create ata
    let minter_ata = anchor_spl::associated_token::get_associated_token_address(
        &minter.pubkey(),
        &mint.pubkey(),
    );
    let create_ata_ix = spl_associated_token_account::create_associated_token_account(
        &minter.pubkey(),
        &minter.pubkey(),
        &mint.pubkey(),
    );
    let create_ata_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_ata_ix],
        Some(&minter.pubkey()),
        &[&minter],
        program_test_context.last_blockhash,
    );

    program_test_context
        .banks_client
        .process_transaction(create_ata_tx)
        .await
        .unwrap();

    // mint some sure tokens
    // create associated token address
    let mint_tokens_tx = mint_amount(
        &minter,
        &mint,
        &minter_ata,
        &program_test_context.last_blockhash,
        1_000_000_000_000,
    )
    .unwrap();

    // TRANSACTION
    program_test_context
        .banks_client
        .process_transaction(mint_tokens_tx)
        .await
        .unwrap();

    // transfer token to proposer
    let proposer_ata = anchor_spl::associated_token::get_associated_token_address(
        &proposer.pubkey(),
        &mint.pubkey(),
    );
    let create_ata_ix = spl_associated_token_account::create_associated_token_account(
        &proposer.pubkey(),
        &proposer.pubkey(),
        &mint.pubkey(),
    );
    let create_ata_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_ata_ix],
        Some(&proposer.pubkey()),
        &[&proposer],
        program_test_context.last_blockhash,
    );

    program_test_context
        .banks_client
        .process_transaction(create_ata_tx)
        .await
        .unwrap();

    let transfer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &minter_ata,
        &proposer_ata,
        &minter.pubkey(),
        &[&minter.pubkey()],
        1_000_000_000_000,
    )
    .unwrap();

    let transfer_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&minter.pubkey()),
        &[&minter],
        program_test_context.last_blockhash,
    );
    program_test_context
        .banks_client
        .process_transaction(transfer_tx)
        .await
        .unwrap();

    test_amount_balance(
        &mut program_test_context,
        &proposer.pubkey(),
        &mint.pubkey(),
        1_000_000_000,
        "main",
    )
    .await;
    // ======== ORACLE - INITIALIZE ORACLE =============
    let config_account = Pubkey::find_program_address(
        &[
            SURE_ORACLE_CONFIG_SEED.as_bytes().as_ref(),
            mint.pubkey().as_ref(),
        ],
        &oracle::id(),
    );
    let config = oracle::accounts::InitializeConfig {
        signer: oracle_owner.pubkey(),
        config: config_account.0.key(),
        token_mint: mint.pubkey(),
        system_program: anchor_lang::system_program::ID,
    };

    // create initialize config instruction
    let initialie_config_args = oracle::instruction::InitializeConfig {
        protocol_authority: oracle_owner.pubkey(),
    };
    let intialize_config_ix = solana_sdk::instruction::Instruction {
        program_id: oracle::id(),
        accounts: config.to_account_metas(None),
        data: initialie_config_args.data(),
    };
    let initialize_config_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[intialize_config_ix],
        Some(&oracle_owner.pubkey()),
        &[&oracle_owner],
        program_test_context.last_blockhash,
    );

    // seend program tx
    program_test_context
        .banks_client
        .process_transaction(initialize_config_tx)
        .await
        .unwrap();

    let config_account = program_test_context
        .banks_client
        .get_account(config_account.0.key())
        .await
        .unwrap()
        .unwrap();

    // deserialize account
    let config_account_d =
        oracle::states::Config::try_deserialize(&mut config_account.data.as_slice()).unwrap();
    assert_eq!(config_account_d.initialized, true);

    // create proposa
    lock_tokens(
        &mut program_test_context,
        proposer,
        &locker_result.locker,
        &mint.pubkey(),
        100_000_000,
        (SECONDS_PER_DAY * 365) as i64, // lockup for a year
    )
    .await;
}
