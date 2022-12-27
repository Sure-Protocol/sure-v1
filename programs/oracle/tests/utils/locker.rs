use anchor_client::{solana_sdk::signer::Signer, *};
use anchor_lang::prelude::*;
use anchor_lang::*;
use govern;
use locked_voter;
use smart_wallet;
use solana_program::clock::SECONDS_PER_DAY;
use solana_program_test::*;
use solana_sdk::*;
use spl_associated_token_account::create_associated_token_account;

use super::test_amount_balance;
pub struct SetupLockerResult {
    pub locker: Pubkey,
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
