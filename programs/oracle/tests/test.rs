use std::io::Read;

use anchor_client::solana_sdk::signers::Signers;
use anchor_client::solana_sdk::timing::SECONDS_PER_YEAR;
use anchor_client::{solana_sdk::signer::Signer, *};
use anchor_lang::prelude::*;
use anchor_lang::*;
use govern;
use locked_voter;
use oracle::id;
use oracle::utils::SURE_ORACLE_CONFIG_SEED;
use smart_wallet;
use solana_program::clock::SECONDS_PER_DAY;
use solana_program::hash::Hash;
use solana_program::instruction::Instruction;
use solana_program::program_pack::Pack;
use solana_program_test::*;
use solana_sdk::*;
use spl_associated_token_account::create_associated_token_account;
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

pub fn add_necessary_programs(ctx: &mut ProgramTest) {
    ctx.add_program("smart_wallet", smart_wallet::id(), None);
    ctx.add_program("govern", govern::id(), None);
    ctx.add_program("locked_voter", locked_voter::id(), None);
}

pub async fn test_amount_balance(
    ctx: &mut ProgramTestContext,
    wallet: &Pubkey,
    mint: &Pubkey,
    expected_amount: u64,
    tag: &str,
) {
    let account_balance = get_account_balance(ctx, wallet, mint).await.unwrap();
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
// @ checkpoint - must find mint token balance
pub async fn get_account_balance(
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

/// lock_tokens allows users to lock
/// their tokens int the locker based on the mint
///
pub async fn lock_tokens(
    ctx: &mut ProgramTestContext,
    user: solana_sdk::signature::Keypair,
    locker: &Pubkey,
    mint: &Pubkey,
    amount: u64,
    duration: i64,
) {
    test_amount_balance(ctx, &user.pubkey(), mint, amount, "lock_tokens").await;
    let source_token_account =
        anchor_spl::associated_token::get_associated_token_address(&user.pubkey(), &mint);

    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(
        &[
            "Escrow".as_bytes(),
            &locker.to_bytes(),
            &user.pubkey().to_bytes(),
        ],
        &locked_voter::id(),
    );
    let mut ixs = Vec::new();
    let escrow_token_account =
        anchor_spl::associated_token::get_associated_token_address(&escrow_pda, &mint);
    let create_escrow_token_account_ix =
        create_associated_token_account(&user.pubkey(), &escrow_pda, &mint);
    ixs.push(create_escrow_token_account_ix);

    let escrow_account = ctx.banks_client.get_account(escrow_pda).await.unwrap();

    // if escrow account does not exist - create it
    if (escrow_account.is_none()) {
        let create_escrow_accounts = locked_voter::accounts::NewEscrow {
            locker: *locker,
            escrow: escrow_pda,
            escrow_owner: user.pubkey(),
            payer: user.pubkey(),
            system_program: anchor_lang::system_program::ID,
        };

        let create_escrow_data = locked_voter::instruction::NewEscrow { _bump: escrow_bump };

        let create_escrow_ix = solana_sdk::instruction::Instruction {
            program_id: locked_voter::id(),
            accounts: create_escrow_accounts.to_account_metas(None),
            data: create_escrow_data.data(),
        };
        ixs.push(create_escrow_ix)
    }

    // lock tokens
    let lock_tokens_accounts = locked_voter::accounts::Lock {
        locker: *locker,
        escrow: escrow_pda,
        escrow_owner: user.pubkey(),
        escrow_tokens: escrow_token_account,
        source_tokens: source_token_account,
        token_program: anchor_spl::token::ID,
    };

    let lock_tokens_with_whitelist_accounts = locked_voter::accounts::LockWithWhitelist {
        lock: lock_tokens_accounts,
        instructions_sysvar: solana_sdk::sysvar::instructions::id(),
    };

    let lock_tokens_data = locked_voter::instruction::LockWithWhitelist { amount, duration };

    let lock_tokens_ix = solana_sdk::instruction::Instruction {
        program_id: locked_voter::id(),
        accounts: lock_tokens_with_whitelist_accounts.to_account_metas(None),
        data: lock_tokens_data.data(),
    };
    ixs.append(&mut [lock_tokens_ix].to_vec());

    let lock_tokens_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &ixs,
        Some(&user.pubkey()),
        &[&user],
        ctx.last_blockhash,
    );

    ctx.banks_client
        .process_transaction(lock_tokens_tx)
        .await
        .unwrap();
}

struct SetupLockerResult {
    locker: Pubkey,
}

/// setup_sure_locker
/// set ups
///     - locker
///     - governor
///     - smart walle
pub async fn setup_sure_locker(
    ctx: &mut ProgramTestContext,
    protocol_owner: &solana_sdk::signature::Keypair,
    mint: &Pubkey,
) -> Result<SetupLockerResult> {
    // let mut program_test = ProgramTest::new("oracle", id(), processor!(oracle::entry));
    // let protocol_owner = signature::Keypair::new();
    // initialize_account(&protocol_owner.pubkey(), &mut program_test);

    // let mut test_context = program_test.start_with_context().await;

    //let base = solana_sdk::signature::Keypair::new();
    let (smart_wallet_pda, smart_wallet_bump) = Pubkey::find_program_address(
        &[
            "GokiSmartWallet".as_bytes(),
            &protocol_owner.pubkey().to_bytes(),
        ],
        &smart_wallet::id(),
    );
    // create smart locker - get goki sdk
    // get required data
    let (governor_pda, governor_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[
            "TribecaGovernor".as_bytes(),
            &protocol_owner.pubkey().to_bytes(),
        ],
        &govern::id(),
    );

    // one of the owners of the smart wallet must be the governor
    let create_smart_wallet_data = smart_wallet::instruction::CreateSmartWallet {
        _bump: smart_wallet_bump,
        max_owners: 5,
        owners: [governor_pda].to_vec(),
        threshold: 1,
        minimum_delay: 1,
    };

    // get required accounts
    let create_smart_wallet_account = smart_wallet::accounts::CreateSmartWallet {
        base: protocol_owner.pubkey(),
        smart_wallet: smart_wallet_pda,
        payer: protocol_owner.pubkey(),
        system_program: anchor_lang::system_program::ID,
    };

    let create_smart_wallet_ix = instruction::Instruction {
        program_id: smart_wallet::id(),
        accounts: create_smart_wallet_account.to_account_metas(None),
        data: create_smart_wallet_data.data(),
    };

    let create_smart_wallet_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_smart_wallet_ix],
        Some(&protocol_owner.pubkey()),
        &[protocol_owner],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(create_smart_wallet_tx)
        .await
        .unwrap();

    // use smart wallet to create governor
    let (locker_pda, locker_bump) = Pubkey::find_program_address(
        &["Locker".as_bytes(), &protocol_owner.pubkey().to_bytes()],
        &locked_voter::id(),
    );

    let governor_params = govern::GovernanceParameters {
        voting_delay: 60 * 60 * 24,      // one hour voting delay
        voting_period: 60 * 60 * 24 * 7, // 7 days voting period
        quorum_votes: 100_000_000,
        timelock_delay_seconds: 60 * 60 * 24,
    };
    let governor_data = govern::instruction::CreateGovernor {
        _bump: governor_bump,
        electorate: locker_pda,
        params: governor_params,
    };

    let governor_accounts = govern::accounts::CreateGovernor {
        base: protocol_owner.pubkey(),
        governor: governor_pda,
        smart_wallet: smart_wallet_pda,
        payer: protocol_owner.pubkey(),
        system_program: anchor_lang::system_program::ID,
    };

    let create_governor_ix = solana_sdk::instruction::Instruction {
        program_id: govern::id(),
        accounts: governor_accounts.to_account_metas(None),
        data: governor_data.data(),
    };

    let create_governor_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_governor_ix],
        Some(&protocol_owner.pubkey()),
        &[protocol_owner],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(create_governor_tx)
        .await
        .unwrap();

    //create locker

    let locker_params = locked_voter::LockerParams {
        whitelist_enabled: true,
        max_stake_duration: SECONDS_PER_DAY * 365 * 4, // max 4 years
        max_stake_vote_multiplier: 1,
        min_stake_duration: SECONDS_PER_DAY * 30, // min 30 days
        proposal_activation_min_votes: 100_000_000,
    };
    let create_locker_data = locked_voter::instruction::NewLocker {
        _bump: locker_bump,
        params: locker_params,
    };

    let create_locker_accounts = locked_voter::accounts::NewLocker {
        base: protocol_owner.pubkey(),
        locker: locker_pda,
        token_mint: *mint,
        governor: governor_pda,
        payer: protocol_owner.pubkey(),
        system_program: anchor_lang::system_program::ID,
    };

    let create_locker_ix = solana_sdk::instruction::Instruction {
        program_id: locked_voter::id(),
        accounts: create_locker_accounts.to_account_metas(None),
        data: create_locker_data.data(),
    };

    let create_locker_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[create_locker_ix],
        Some(&protocol_owner.pubkey()),
        &[protocol_owner],
        ctx.last_blockhash,
    );
    ctx.banks_client
        .process_transaction(create_locker_tx)
        .await
        .unwrap();

    return Result::Ok(SetupLockerResult { locker: locker_pda });
}

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
