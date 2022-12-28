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

use hex_literal::hex;

use anchor_client::{solana_sdk::signer::Signer, *};
use anchor_lang::prelude::*;
use anchor_lang::*;
use anchor_spl::associated_token;
use govern;
use locked_voter;
use oracle::id;
use oracle::program::Oracle;
use oracle::utils::SURE_ORACLE_CONFIG_SEED;
use sha3::{Digest, Sha3_256};
use smart_wallet;
use solana_program::clock::SECONDS_PER_DAY;
use solana_program::hash::Hash;
use solana_program_test::*;
use solana_sdk::*;
use spl_token;
use std::hash;
use std::time::Duration;
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

async fn get_oracle_proposal(
    ctx: &mut ProgramTestContext,
    proposal: &Pubkey,
) -> Result<oracle::states::Proposal> {
    // check status
    let proposal_data = ctx
        .banks_client
        .get_account(*proposal)
        .await
        .unwrap()
        .unwrap();

    Result::Ok(
        oracle::states::Proposal::try_deserialize(&mut proposal_data.data.as_slice()).unwrap(),
    )
}

async fn test_foward_time(ctx: &mut ProgramTestContext, add_time: i64) {
    let clock_sysvar: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    println!(
        "Original Time: epoch = {}, timestamp = {}",
        clock_sysvar.epoch, clock_sysvar.unix_timestamp
    );
    let mut new_clock = clock_sysvar.clone();
    new_clock.epoch = new_clock.epoch + 30;
    new_clock.unix_timestamp += add_time;

    ctx.set_sysvar(&new_clock);
    let clock_sysvar: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    println!(
        "New Time: epoch = {}, timestamp = {}",
        clock_sysvar.epoch, clock_sysvar.unix_timestamp
    );
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

    let voter1 = signature::Keypair::new();
    initialize_account(&voter1.pubkey(), &mut program_test);

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
    let voter1_ata = anchor_spl::associated_token::get_associated_token_address(
        &voter1.pubkey(),
        &mint.pubkey(),
    );

    let create_proposer_ata_ix = spl_associated_token_account::create_associated_token_account(
        &proposer.pubkey(),
        &proposer.pubkey(),
        &mint.pubkey(),
    );
    let create_voter1_ata_ix = spl_associated_token_account::create_associated_token_account(
        &voter1.pubkey(),
        &voter1.pubkey(),
        &mint.pubkey(),
    );
    let create_ata_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_proposer_ata_ix, create_voter1_ata_ix],
        Some(&proposer.pubkey()),
        &[&proposer, &voter1],
        program_test_context.last_blockhash,
    );

    program_test_context
        .banks_client
        .process_transaction(create_ata_tx)
        .await
        .unwrap();

    let transfer_sure_to_proposer_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &minter_ata,
        &proposer_ata,
        &minter.pubkey(),
        &[&minter.pubkey()],
        500_000_000_000,
    )
    .unwrap();

    let transfer_sure_to_voter1_ix = spl_token::instruction::transfer(
        &spl_token::ID,
        &minter_ata,
        &voter1_ata,
        &minter.pubkey(),
        &[&minter.pubkey()],
        100_000_000_000,
    )
    .unwrap();

    let transfer_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[transfer_sure_to_proposer_ix, transfer_sure_to_voter1_ix],
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
    test_amount_balance(
        &mut program_test_context,
        &voter1.pubkey(),
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
    let config_account_PK = config_account.0.key();
    let config = oracle::accounts::InitializeConfig {
        signer: oracle_owner.pubkey(),
        config: config_account_PK,
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

    // ==== create proposal ====
    lock_tokens(
        &mut program_test_context,
        &proposer,
        &locker_result.locker,
        &mint.pubkey(),
        100_000_000,
        (SECONDS_PER_DAY * 365) as i64, // lockup for a year
    )
    .await;

    // create proposal id
    let mut hasher = Sha3_256::new();
    hasher.update(b"a");
    let proposal_id = hasher.finalize();

    let (proposal_pda, proposal_bump) = Pubkey::find_program_address(
        &[
            oracle::utils::SURE_ORACLE_SEED.as_bytes(),
            proposal_id.as_slice(),
        ],
        &oracle::id(),
    );
    let (reveal_vote_array_pda, reveal_vote_array_bump) = Pubkey::find_program_address(
        &[
            oracle::utils::SURE_ORACLE_REVEAL_ARRAY_SEED.as_bytes(),
            proposal_id.as_slice(),
        ],
        &oracle::id(),
    );
    let (proposal_vault_pda, proposal_vault_bump) = Pubkey::find_program_address(
        &[
            oracle::utils::SURE_ORACLE_PROPOSAL_VAULT_SEED.as_bytes(),
            proposal_id.as_slice(),
        ],
        &oracle::id(),
    );

    let create_proposal_accounts = oracle::accounts::ProposeVote {
        proposer: proposer.pubkey(),
        config: config_account_PK,
        proposal: proposal_pda,
        reveal_vote_array: reveal_vote_array_pda,
        proposer_account: proposer_ata,
        proposal_vault_mint: mint.pubkey(),
        proposal_vault: proposal_vault_pda,
        token_program: spl_token::id(),
        associated_token_program: associated_token::ID,
        rent: solana_program::sysvar::rent::ID,
        system_program: anchor_lang::system_program::ID,
    };

    let create_proposal_data = oracle::instruction::ProposeVote {
        id: proposal_id.to_vec(),
        name: "my test proposal".to_string(),
        description: "hello".to_string(),
        stake: 100_000_000,
    };

    let create_proposal_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[solana_sdk::instruction::Instruction {
            program_id: oracle::ID,
            accounts: create_proposal_accounts.to_account_metas(None),
            data: create_proposal_data.data(),
        }],
        Some(&proposer.pubkey()),
        &[&proposer],
        program_test_context.last_blockhash,
    );
    program_test_context
        .banks_client
        .process_transaction(create_proposal_tx)
        .await
        .unwrap();

    // ====== VOTE ON PROPOSAL ======
    // lock tokens
    lock_tokens(
        &mut program_test_context,
        &voter1,
        &locker_result.locker,
        &mint.pubkey(),
        100_000_000,
        (SECONDS_PER_DAY * 365) as i64, // lockup for a year
    )
    .await;

    let (voter1_account_pda, voter1_account_bump) = Pubkey::find_program_address(
        &[
            oracle::utils::SURE_ORACLE_VOTE_SEED.as_bytes(),
            &proposal_pda.to_bytes(),
            &voter1.pubkey().to_bytes(),
        ],
        &oracle::ID,
    );

    let (voter1_escrow_pda, voter1_escrow_bump) =
        get_user_escrow_pda(&locker_result.locker, &voter1.pubkey());
    let voter1_vote_accounts = oracle::accounts::SubmitVote {
        voter: voter1.pubkey(),
        voter_account: voter1_ata,
        locker: locker_result.locker,
        user_escrow: voter1_escrow_pda,
        proposal: proposal_pda,
        proposal_vault: proposal_vault_pda,
        proposal_vault_mint: mint.pubkey(),
        vote_account: voter1_account_pda,
        token_program: spl_token::ID,
        rent: solana_program::sysvar::rent::ID,
        system_program: anchor_lang::system_program::ID,
    };

    let vote: i32 = 10;
    let mut voter1_hasher = Sha3_256::new();
    voter1_hasher.update(vote.to_le_bytes());
    let vote_hash = voter1_hasher.finalize();
    let voter1_vote_data = oracle::instruction::SubmitVote {
        vote_hash: vote_hash.to_vec(),
    };

    let voter1_submit_vote_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[solana_program::instruction::Instruction {
            program_id: oracle::ID,
            accounts: voter1_vote_accounts.to_account_metas(None),
            data: voter1_vote_data.data(),
        }],
        Some(&voter1.pubkey()),
        &[&voter1],
        program_test_context.last_blockhash,
    );
    program_test_context
        .banks_client
        .process_transaction(voter1_submit_vote_tx)
        .await
        .unwrap();

    let proposal = get_oracle_proposal(&mut program_test_context, &proposal_pda)
        .await
        .unwrap();
    assert!(
        proposal.status == 2,
        "[main] Assert failed. Proposal status is not 1 but {}",
        proposal.status
    );

    // fast forward time
    let time_add = Duration::from_secs(SECONDS_PER_DAY).as_secs() as i64;
    test_foward_time(&mut program_test_context, time_add).await;
}
