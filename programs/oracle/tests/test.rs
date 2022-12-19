use anchor_client::{solana_sdk::signer::Signer, *};
use anchor_lang::prelude::*;
use anchor_lang::*;
use oracle::id;
use oracle::utils::SURE_ORACLE_CONFIG_SEED;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::*;
use spl_token;

#[tokio::test]
async fn create_and_init() {
    let mut program_test = ProgramTest::new("oracle", id(), processor!(oracle::entry));

    // create program oracle owner
    let oracle_owner = signature::Keypair::new();
    program_test.add_account(
        oracle_owner.pubkey(),
        account::Account {
            lamports: 1_000_000_000,
            ..account::Account::default()
        },
    );
    // create minter
    let minter = signature::Keypair::new();
    program_test.add_account(
        minter.pubkey(),
        account::Account {
            lamports: 1_000_000_000,
            ..account::Account::default()
        },
    );
    let mut program_test_context = program_test.start_with_context().await;

    // create token mint - sure
    let mint_account = signature::Keypair::new();

    let token_program = spl_token::id();
    let rent = program_test_context.banks_client.get_rent().await.unwrap();
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
        program_test_context.last_blockhash,
    );

    // sign and send transaction
    program_test_context
        .banks_client
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
}
