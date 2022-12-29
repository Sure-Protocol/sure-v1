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

use anchor_client::solana_sdk::signature::Keypair;

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

async fn test_foward_time_delta(ctx: &mut ProgramTestContext, add_time: i64) {
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

async fn test_foward_time(ctx: &mut ProgramTestContext, end_at: i64) {
    let clock_sysvar: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    println!(
        "Original Time: epoch = {}, timestamp = {}",
        clock_sysvar.epoch, clock_sysvar.unix_timestamp
    );
    let mut new_clock = clock_sysvar.clone();
    new_clock.epoch = new_clock.epoch + 30;
    new_clock.unix_timestamp = end_at;

    ctx.set_sysvar(&new_clock);
    let clock_sysvar: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    println!(
        "New Time: epoch = {}, timestamp = {}",
        clock_sysvar.epoch, clock_sysvar.unix_timestamp
    );
}

pub fn get_voter_account_pda(proposal: &Pubkey, voter: &Pubkey) -> Result<(Pubkey, u8)> {
    Result::Ok(Pubkey::find_program_address(
        &[
            oracle::utils::SURE_ORACLE_VOTE_SEED.as_bytes(),
            &proposal.to_bytes(),
            &voter.to_bytes(),
        ],
        &oracle::ID,
    ))
}

pub fn get_proposal_pda(proposal_id: &Vec<u8>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[oracle::utils::SURE_ORACLE_SEED.as_bytes(), proposal_id],
        &oracle::id(),
    )
}

pub fn get_proposal_vault_pda(proposal_id: &Vec<u8>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            oracle::utils::SURE_ORACLE_PROPOSAL_VAULT_SEED.as_bytes(),
            proposal_id,
        ],
        &oracle::id(),
    )
}

pub fn get_reveal_vote_array_pda(proposal_id: &Vec<u8>) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            oracle::utils::SURE_ORACLE_REVEAL_ARRAY_SEED.as_bytes(),
            proposal_id,
        ],
        &oracle::id(),
    )
}

pub fn create_proposal_id(val: &str) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(val.as_bytes());
    let hasher_final = hasher.finalize();
    hasher_final.as_slice().to_vec()
}

async fn propose_vote(
    ctx: &mut ProgramTestContext,
    proposal_id: &Vec<u8>,
    proposer: &Keypair,
    mint: &Pubkey,
    config_account: &Pubkey,
) {
    let proposer_ata =
        anchor_spl::associated_token::get_associated_token_address(&proposer.pubkey(), mint);

    let (reveal_vote_array_pda, reveal_vote_array_bump) = get_reveal_vote_array_pda(proposal_id);
    let (proposal_pda, proposal_bump) = get_proposal_pda(proposal_id);
    let (proposal_vault_pda, proposal_vault_bump) = get_proposal_vault_pda(proposal_id);

    let create_proposal_accounts = oracle::accounts::ProposeVote {
        proposer: proposer.pubkey(),
        config: *config_account,
        proposal: proposal_pda,
        reveal_vote_array: reveal_vote_array_pda,
        proposer_account: proposer_ata,
        proposal_vault_mint: *mint,
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
        &[proposer],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(create_proposal_tx)
        .await
        .unwrap();
}

/// submit_vote submits a vote for the given user
async fn submit_vote(
    ctx: &mut ProgramTestContext,
    vote_hash: Vec<u8>,
    proposal: &Pubkey,
    proposal_vault: &Pubkey,
    voter: &Keypair,
    voter_ata: &Pubkey,
    mint: &Pubkey,
    locker: &Pubkey,
) {
    let (voter_account_pda, voter_account_bump) =
        get_voter_account_pda(proposal, &voter.pubkey()).unwrap();

    let (voter1_escrow_pda, voter1_escrow_bump) = get_user_escrow_pda(locker, &voter.pubkey());
    let voter1_vote_accounts = oracle::accounts::SubmitVote {
        voter: voter.pubkey(),
        voter_account: *voter_ata,
        locker: *locker,
        user_escrow: voter1_escrow_pda,
        proposal: *proposal,
        proposal_vault: *proposal_vault,
        proposal_vault_mint: *mint,
        vote_account: voter_account_pda,
        token_program: spl_token::ID,
        rent: solana_program::sysvar::rent::ID,
        system_program: anchor_lang::system_program::ID,
    };

    let voter1_vote_data = oracle::instruction::SubmitVote {
        vote_hash: vote_hash,
    };

    let voter1_submit_vote_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[solana_program::instruction::Instruction {
            program_id: oracle::ID,
            accounts: voter1_vote_accounts.to_account_metas(None),
            data: voter1_vote_data.data(),
        }],
        Some(&voter.pubkey()),
        &[voter],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(voter1_submit_vote_tx)
        .await
        .unwrap();
}

pub async fn test_create_mint(
    ctx: &mut ProgramTestContext,
    minter: &Keypair,
    mint: &Keypair,
    decimals: u8,
) {
    // create mint instruction
    let rent = ctx.banks_client.get_rent().await.unwrap();

    let mint_tx = create_mint(minter, &mint, rent, ctx.last_blockhash, decimals).unwrap();
    // sign and send transaction
    ctx.banks_client.process_transaction(mint_tx).await.unwrap();

    return;
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

    // CONFIG
    let SURE_MINT_AMOUNT = 100_000_000_000_000; // 100 000 000 sures
    let ORACLE_VOTING_LENGTH = SECONDS_PER_DAY as i64;
    let ORACLE_REVEAL_LENGTH = SECONDS_PER_DAY as i64;
    let REQUIRED_VE_VOTES: i64 = 1_000_000_000_000; // 1 000 000 ve sure
    let ORACLE_MIN_PROPOSER_STAKE = 100_000_000; // 100 sures
    let ORACLE_VOTE_STAKE_RATE = 10; // need to stake 10% of all veSures

    // create sure mint
    let mint = solana_sdk::signature::Keypair::new();
    test_create_mint(&mut program_test_context, &minter, &mint, 6).await;

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
        SURE_MINT_AMOUNT,
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
    let (config_account_pda, config_account_bump) = Pubkey::find_program_address(
        &[
            SURE_ORACLE_CONFIG_SEED.as_bytes().as_ref(),
            mint.pubkey().as_ref(),
        ],
        &oracle::id(),
    );
    let config = oracle::accounts::InitializeConfig {
        signer: oracle_owner.pubkey(),
        config: config_account_pda,
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
        .get_account(config_account_pda)
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
    hasher.update("a".as_bytes());
    let hasher_final = hasher.finalize();
    let proposal_id = hasher_final.as_slice().to_vec();
    propose_vote(
        &mut program_test_context,
        &proposal_id,
        &proposer,
        &mint.pubkey(),
        &config_account_pda,
    )
    .await;

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

    // VOTER1 vote

    let voter1_vote: i64 = 10;
    let mut voter1_hasher = Sha3_256::new();
    voter1_hasher.update(voter1_vote.to_le_bytes());
    let vote_hash = voter1_hasher.finalize().to_vec();
    submit_vote(
        &mut program_test_context,
        vote_hash,
        &get_proposal_pda(&proposal_id).0,
        &get_proposal_vault_pda(&proposal_id).0,
        &voter1,
        &voter1_ata,
        &mint.pubkey(),
        &locker_result.locker,
    )
    .await;

    let proposal =
        get_oracle_proposal(&mut program_test_context, &get_proposal_pda(&proposal_id).0)
            .await
            .unwrap();
    assert!(
        proposal.status == 2,
        "[main] Assert failed. Proposal status is not 1 but {}",
        proposal.status
    );
    assert!(
        proposal.votes != 0,
        "[main] suspicious. Number of votes is {}",
        proposal.votes
    );

    // fast forward time beyond voting period
    test_foward_time(&mut program_test_context, proposal.vote_end_at + 1).await;
    let proposal_account =
        get_oracle_proposal(&mut program_test_context, &get_proposal_pda(&proposal_id).0)
            .await
            .unwrap();
    let proposal_status = proposal_account.get_status(proposal.vote_end_at + 1);
    println!(
        "[Proposal status] Current status is {:?}",
        &proposal_status.get_id()
    );
    // try to reveal vote
    let (voter_account_pda, voter_account_bump) =
        get_voter_account_pda(&get_proposal_pda(&proposal_id).0, &voter1.pubkey()).unwrap();
    let reveal_vote_data = oracle::instruction::RevealVote {
        salt: "".to_string(),
        vote: voter1_vote,
    };
    let reveal_vote_acounts = oracle::accounts::RevealVote {
        voter: voter1.pubkey(),
        proposal: get_proposal_pda(&proposal_id).0,
        reveal_vote_array: get_reveal_vote_array_pda(&proposal_id).0,
        vote_account: voter_account_pda,
        system_program: anchor_lang::system_program::ID,
    };

    let reveal_vote_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[solana_sdk::instruction::Instruction {
            program_id: oracle::id(),
            accounts: reveal_vote_acounts.to_account_metas(None),
            data: reveal_vote_data.data(),
        }],
        Some(&voter1.pubkey()),
        &[&voter1],
        program_test_context.last_blockhash,
    );
    let tx_result = program_test_context
        .banks_client
        .process_transaction(reveal_vote_tx)
        .await;
    assert!(tx_result.is_err(), "[main] suspicious. Should fail");

    // assume test failed since it didn't reach quoroum
    assert!(
        proposal_status.get_id() == 0,
        "[main] Status is not 3 but {} ",
        proposal_status.get_id()
    )
}
