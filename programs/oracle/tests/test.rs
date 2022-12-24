use anchor_client::solana_sdk::signers::Signers;
use anchor_client::{solana_sdk::signer::Signer, *};
use anchor_lang::prelude::*;
use anchor_lang::*;
use oracle::id;
use oracle::utils::SURE_ORACLE_CONFIG_SEED;
use solana_program::hash::Hash;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::*;
use spl_token;

fn initialize_account(address: &Pubkey, program_test: &mut ProgramTest) {
    program_test.add_account(
        *address,
        account::Account {
            lamports: 1_000_000_000,
            ..account::Account::default()
        },
    );
}

fn create_mint<T: Signer>(
    minter: &T,
    mint: &T,
    rent: Rent,
    recent_blockhash: Hash,
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
        6,
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

fn mint_amount<T: Signer>(
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

#[tokio::test]
async fn create_veSure() {}

#[tokio::test]
async fn create_and_init() {
    let mut program_test = ProgramTest::new("oracle", id(), processor!(oracle::entry));

    // create program oracle owner
    let oracle_owner = signature::Keypair::new();
    initialize_account(&oracle_owner.pubkey(), &mut program_test);
    // create minter
    let minter = signature::Keypair::new();
    initialize_account(&minter.pubkey(), &mut program_test);

    let proposer = signature::Keypair::new();
    initialize_account(&proposer.pubkey(), &mut program_test);

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
        &minter.pubkey(),
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
        1_000_000_000,
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
}
