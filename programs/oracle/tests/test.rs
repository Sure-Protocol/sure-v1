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

use anchor_client::solana_sdk::timing::SECONDS_PER_YEAR;
use anchor_client::solana_sdk::transaction::Transaction;
use anchor_client::{solana_sdk::signer::Signer, *};
use anchor_lang::prelude::*;
use anchor_lang::*;
use anchor_spl::associated_token;
use anchor_spl::token::Mint;
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
use spl_associated_token_account::get_associated_token_address;
use spl_token;
use std::collections::HashMap;
use std::ops::{Div, Index};
use std::time::Duration;
use utils::locker::*;
use utils::tokens::*;

/// initialize_account tops up the given address with enough sol to
/// work in the context of the integration test
fn add_account(address: &Pubkey, program_test: &mut ProgramTest) {
    program_test.add_account(
        *address,
        account::Account {
            lamports: 1_000_000_000,
            ..account::Account::default()
        },
    );
}

fn add_accounts(addresses: Vec<Pubkey>, program_test: &mut ProgramTest) {
    addresses
        .iter()
        .for_each(|&address| add_account(&address, program_test))
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
fn get_submit_vote_transaction(
    vote_hash: Vec<u8>,
    proposal: &Pubkey,
    proposal_vault: &Pubkey,
    voter: &Keypair,
    voter_ata: &Pubkey,
    mint: &Pubkey,
    locker: &Pubkey,
    last_blockhash: Hash,
) -> transaction::Transaction {
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

    solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[solana_program::instruction::Instruction {
            program_id: oracle::ID,
            accounts: voter1_vote_accounts.to_account_metas(None),
            data: voter1_vote_data.data(),
        }],
        Some(&voter.pubkey()),
        &[voter],
        last_blockhash,
    )
}

/// get_reveal_vote_transaction
pub fn get_reveal_vote_transaction(
    salt: &str,
    vote: i64,
    proposal_id: &Vec<u8>,
    voter: &Keypair,
    recent_blockhash: Hash,
) -> transaction::Transaction {
    let (proposal_pda, proposal_bump) = get_proposal_pda(proposal_id);
    let (voter_account_pda, voter_account_bump) =
        get_voter_account_pda(&proposal_pda, &voter.pubkey()).unwrap();
    let (reveal_vote_array_pda, reveal_vote_array_bump) = get_reveal_vote_array_pda(proposal_id);

    let reveal_vote_accounts = oracle::accounts::RevealVote {
        voter: voter.pubkey(),
        proposal: proposal_pda,
        reveal_vote_array: reveal_vote_array_pda,
        vote_account: voter_account_pda,
        system_program: anchor_lang::system_program::ID,
    };

    let reveal_vote_data = oracle::instruction::RevealVote {
        salt: salt.to_string(),
        vote,
    };

    let reveal_vote_ix = solana_sdk::instruction::Instruction {
        program_id: oracle::id(),
        accounts: reveal_vote_accounts.to_account_metas(None),
        data: reveal_vote_data.data(),
    };

    solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[solana_sdk::instruction::Instruction {
            program_id: oracle::id(),
            accounts: reveal_vote_accounts.to_account_metas(None),
            data: reveal_vote_data.data(),
        }],
        Some(&voter.pubkey()),
        &[voter],
        recent_blockhash,
    )
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

pub struct Minter {
    keypair: Keypair,
    mint: Keypair,
}

impl Minter {
    pub fn init() -> Self {
        return Minter {
            keypair: signature::Keypair::new(),
            mint: signature::Keypair::new(),
        };
    }

    pub fn get_mint_pubkey(&self) -> Pubkey {
        self.mint.pubkey()
    }
    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    pub fn get_associated_token_address(&self) -> Pubkey {
        get_associated_token_address(&self.keypair.pubkey(), &self.mint.pubkey())
    }

    pub fn get_create_mint_transaction(
        &self,
        decimals: u8,
        rent: Rent,
        last_blockhash: Hash,
    ) -> Result<Transaction> {
        create_mint(&self.keypair, &self.mint, rent, last_blockhash, decimals)
    }

    pub fn get_genesis_mint_tx(
        &self,
        amount: u64,
        last_blockhash: Hash,
    ) -> Result<transaction::Transaction> {
        let minter_ata = anchor_spl::associated_token::get_associated_token_address(
            &self.keypair.pubkey(),
            &self.mint.pubkey(),
        );
        let create_ata_ix = spl_associated_token_account::create_associated_token_account(
            &self.keypair.pubkey(),
            &self.keypair.pubkey(),
            &self.mint.pubkey(),
        );

        let mint_tokens_ix = spl_token::instruction::mint_to(
            &spl_token::ID,
            &self.mint.pubkey(),
            &minter_ata,
            &self.keypair.pubkey(),
            &[&self.keypair.pubkey(), &self.mint.pubkey()],
            amount,
        )?;

        let create_ata_tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[create_ata_ix, mint_tokens_ix],
            Some(&self.keypair.pubkey()),
            &[&self.keypair, &self.mint],
            last_blockhash,
        );

        Ok(create_ata_tx)
    }

    pub fn get_batch_transfer_tx(
        &self,
        pubkeys: Vec<Pubkey>,
        amount: u64,
        blockhash: Hash,
    ) -> solana_sdk::transaction::Transaction {
        let ixs: Vec<instruction::Instruction>;
        let minter_ata = anchor_spl::associated_token::get_associated_token_address(
            &self.keypair.pubkey(),
            &self.mint.pubkey(),
        );
        let amount_per_account = amount.div(pubkeys.len() as u64);

        let v2: Vec<instruction::Instruction> = pubkeys
            .iter()
            .map(|&x| {
                let ata = get_associated_token_address(&x, &self.mint.pubkey());
                spl_token::instruction::transfer(
                    &spl_token::ID,
                    &minter_ata,
                    &ata,
                    &self.keypair.pubkey(),
                    &[&self.keypair.pubkey()],
                    amount_per_account,
                )
                .unwrap()
            })
            .collect();

        solana_sdk::transaction::Transaction::new_signed_with_payer(
            &v2,
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            blockhash,
        )
    }

    pub fn create_ata_and_mint_tokens_tx(
        &self,
        keypair: &Keypair,
        amount: u64,
        last_blockhash: Hash,
    ) -> Transaction {
        // transfer token to proposer
        let ata = anchor_spl::associated_token::get_associated_token_address(
            &keypair.pubkey(),
            &self.mint.pubkey(),
        );

        let create_ata_ix = spl_associated_token_account::create_associated_token_account(
            &keypair.pubkey(),
            &keypair.pubkey(),
            &self.mint.pubkey(),
        );

        let transfer_ix = spl_token::instruction::transfer(
            &spl_token::ID,
            &self.get_associated_token_address(),
            &ata,
            &self.pubkey(),
            &[&self.pubkey()],
            amount,
        )
        .unwrap();

        solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[create_ata_ix, transfer_ix],
            Some(&self.keypair.pubkey()),
            &[&self.keypair, &keypair],
            last_blockhash,
        )
    }
}

pub struct Voters {
    keys: Vec<Keypair>,
    votes: HashMap<Pubkey, u64>,
    salt: String,
}

impl Voters {
    pub fn init(num_voters: u16, program_test: &mut ProgramTest) -> Voters {
        let mut keys: Vec<Keypair> = Vec::new();
        for _ in 0..num_voters {
            let voter = signature::Keypair::new();
            add_account(&voter.pubkey(), program_test);
            keys.push(voter)
        }
        let votes = HashMap::with_capacity(num_voters as usize);
        Voters {
            keys,
            votes,
            salt: "".to_string(),
        }
    }

    pub fn get_pubkeys(&self) -> Vec<Pubkey> {
        self.keys.iter().map(|x| x.pubkey()).collect()
    }

    pub async fn get_token_account_balance(
        &self,
        ctx: &mut ProgramTestContext,
        mint: &Pubkey,
        index: u16,
    ) -> Result<u64> {
        let voter = self.keys.get(index as usize).unwrap();
        get_token_account_balance(ctx, &voter.pubkey(), mint).await
    }

    pub fn initialize_voter(
        &self,
        voter: &Keypair,
        mint: &Pubkey,
        last_blockhash: Hash,
    ) -> transaction::Transaction {
        let create_ata_ix = spl_associated_token_account::create_associated_token_account(
            &voter.pubkey(),
            &voter.pubkey(),
            mint,
        );
        solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[create_ata_ix],
            Some(&voter.pubkey()),
            &[voter],
            last_blockhash,
        )
    }

    pub fn get_initialize_voters_txs(
        &self,
        mint: &Pubkey,
        last_blockhash: Hash,
    ) -> Vec<transaction::Transaction> {
        self.keys
            .iter()
            .map(|x| self.initialize_voter(&x, mint, last_blockhash))
            .collect()
    }

    pub fn get_create_escrow_tx(
        &self,
        locker: &Pubkey,
        mint: &Pubkey,
        last_blockhash: Hash,
    ) -> Vec<Transaction> {
        self.keys
            .iter()
            .map(|user| {
                return get_create_escrow_transaction(&user, locker, mint, last_blockhash);
            })
            .collect()
    }

    pub fn lock_tokens(
        &self,
        locker: &Pubkey,
        mint: &Pubkey,
        amount: u64,
        duration: i64,
        last_blockhash: Hash,
    ) -> Vec<Transaction> {
        self.keys
            .iter()
            .map(|user| {
                return get_lock_tokens_transaction(
                    &user,
                    locker,
                    mint,
                    amount,
                    duration,
                    last_blockhash,
                );
            })
            .collect()
    }

    pub fn set_votes(&mut self, votes: Vec<u32>) {
        let mut i = 0;
        for key in self.keys.iter() {
            self.votes.insert(key.pubkey(), (votes[i] as u64) << 32);
            i += 1
        }
    }
    pub fn set_vote(&mut self, vote: u32, voter: &Pubkey) {
        // convert vote Q32.0 -> Q32.32
        self.votes.insert(*voter, (vote as u64) << 32).unwrap();
    }

    pub fn get_vote(&self, voter: &Pubkey) -> u64 {
        self.votes.get(voter).unwrap().clone()
    }

    pub fn create_vote_hash(&self, voter: &Keypair) -> Vec<u8> {
        let mut hasher = Sha3_256::new();
        hasher.update(
            self.votes
                .get(&voter.pubkey())
                .unwrap()
                .to_string()
                .as_bytes(),
        );
        hasher.finalize().to_vec()
    }

    pub async fn get_vote_account(
        &self,
        ctx: &mut ProgramTestContext,
        voter: &Pubkey,
        proposal: &Pubkey,
    ) -> oracle::states::VoteAccount {
        let (voter_account_pda, _) = get_voter_account_pda(proposal, voter).unwrap();
        let vote_account = ctx
            .banks_client
            .get_account(voter_account_pda)
            .await
            .unwrap()
            .unwrap();

        println!(
            "[get_vote_account] Vote account data length: {}",
            vote_account.data.len()
        );

        oracle::states::VoteAccount::try_deserialize(&mut vote_account.data.as_slice()).unwrap()
    }

    pub fn get_submit_vote_txs(
        &self,
        proposal: &Pubkey,
        proposal_vault: &Pubkey,
        locker: &Pubkey,
        mint: &Pubkey,
        last_blockhash: Hash,
    ) -> Vec<transaction::Transaction> {
        self.keys
            .iter()
            .map(|voter| {
                let vote_hash = self.create_vote_hash(voter);
                let ata = anchor_spl::associated_token::get_associated_token_address(
                    &voter.pubkey(),
                    mint,
                );
                return get_submit_vote_transaction(
                    vote_hash,
                    proposal,
                    proposal_vault,
                    voter,
                    &ata,
                    mint,
                    locker,
                    last_blockhash,
                );
            })
            .collect()
    }

    /// get_reveal_vote_txs
    pub fn get_reveal_vote_txs(
        &self,
        proposal_id: &Vec<u8>,
        recent_blockhash: Hash,
    ) -> Vec<transaction::Transaction> {
        self.keys
            .iter()
            .map(|voter| {
                let voter_vote = self.get_vote(&voter.pubkey());
                return get_reveal_vote_transaction(
                    &self.salt,
                    voter_vote as i64,
                    proposal_id,
                    voter,
                    recent_blockhash,
                );
            })
            .collect()
    }
}

/// Main integration test for the oracle / prediction market
#[tokio::test]
async fn create_and_init() {
    // CONFIG
    let SURE_MINT_AMOUNT: i64 = 100_000_000_000_000; // 100 000 000 sures
    let SURE_MINT_AMOUNT_VOTERS: i64 = 50_000_000_000_000; // 100 000 000 sures
    let SURE_AMOUNT_PROPOSER: i64 = 10_000_000_000_000;
    let ORACLE_VOTING_LENGTH = SECONDS_PER_DAY as i64;
    let ORACLE_REVEAL_LENGTH = SECONDS_PER_DAY as i64;
    let REQUIRED_VE_VOTES: i64 = 1_000_000_000_000; // 1 000 000 ve sure
    let ORACLE_MIN_PROPOSER_STAKE: i64 = 100_000_000; // 100 sures
    let ORACLE_VOTE_STAKE_RATE: i64 = 10; // need to stake 10% of all veSures

    let mut program_test = ProgramTest::new("oracle", id(), processor!(oracle::entry));

    let protocol_owner = signature::Keypair::new();
    add_account(&protocol_owner.pubkey(), &mut program_test);
    // create program oracle owner
    let oracle_owner = signature::Keypair::new();
    add_account(&oracle_owner.pubkey(), &mut program_test);
    // create minter

    let proposer = signature::Keypair::new();
    add_account(&proposer.pubkey(), &mut program_test);

    let voter1 = signature::Keypair::new();
    add_account(&voter1.pubkey(), &mut program_test);

    let mut voters = Voters::init(10, &mut program_test);

    let minter = Minter::init();
    add_account(&minter.keypair.pubkey(), &mut program_test);

    // add_necessary_programs
    add_necessary_programs(&mut program_test);

    // start program
    let mut program_test_context = program_test.start_with_context().await;
    let rent = program_test_context.banks_client.get_rent().await.unwrap();

    // create SURE mint
    let create_mint_tx = minter
        .get_create_mint_transaction(6, rent, program_test_context.last_blockhash)
        .unwrap();

    program_test_context
        .banks_client
        .process_transaction(create_mint_tx)
        .await
        .unwrap();

    // mint the genesis amount
    let genesis_mint_tx = minter
        .get_genesis_mint_tx(SURE_MINT_AMOUNT as u64, program_test_context.last_blockhash)
        .unwrap();

    program_test_context
        .banks_client
        .process_transaction(genesis_mint_tx)
        .await
        .unwrap();

    // initialize voters
    let initialize_voter_txs = voters
        .get_initialize_voters_txs(&minter.mint.pubkey(), program_test_context.last_blockhash);

    program_test_context
        .banks_client
        .process_transactions(initialize_voter_txs)
        .await
        .unwrap();

    //  setup locker
    let locker_result = setup_sure_locker(
        &mut program_test_context,
        &protocol_owner,
        &minter.mint.pubkey(),
    )
    .await
    .unwrap();

    // batch transfer sures
    let batch_transfer_tx = minter.get_batch_transfer_tx(
        voters.get_pubkeys(),
        SURE_MINT_AMOUNT_VOTERS as u64,
        program_test_context.last_blockhash,
    );

    program_test_context
        .banks_client
        .process_transaction(batch_transfer_tx)
        .await
        .unwrap();
    let token_balance = voters
        .get_token_account_balance(&mut program_test_context, &minter.mint.pubkey(), 1)
        .await
        .unwrap();

    assert!(
        token_balance == SURE_MINT_AMOUNT_VOTERS.div(voters.keys.len() as i64) as u64,
        "[main] token account balance: {} != {}",
        token_balance,
        SURE_MINT_AMOUNT_VOTERS
    );

    let transfer_proposer_tx = minter.create_ata_and_mint_tokens_tx(
        &proposer,
        SURE_AMOUNT_PROPOSER as u64,
        program_test_context.last_blockhash,
    );

    program_test_context
        .banks_client
        .process_transaction(transfer_proposer_tx)
        .await
        .unwrap();

    test_amount_balance(
        &mut program_test_context,
        &proposer.pubkey(),
        &minter.mint.pubkey(),
        SURE_AMOUNT_PROPOSER as u64,
        "main",
    )
    .await;

    // // ======== ORACLE - INITIALIZE ORACLE =============
    let (config_account_pda, config_account_bump) = Pubkey::find_program_address(
        &[
            SURE_ORACLE_CONFIG_SEED.as_bytes().as_ref(),
            minter.mint.pubkey().as_ref(),
        ],
        &oracle::id(),
    );
    let config = oracle::accounts::InitializeConfig {
        signer: oracle_owner.pubkey(),
        config: config_account_pda,
        token_mint: minter.mint.pubkey(),
        system_program: anchor_lang::system_program::ID,
    };

    // create initialize config instruction
    let initialie_config_args = oracle::instruction::InitializeConfig {
        protocol_authority: oracle_owner.pubkey(),
        required_votes_fraction: 10000, // 1/100 = 1% of all tokens
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

    let create_proposer_escrow_tx = get_create_escrow_transaction(
        &proposer,
        &locker_result.locker,
        &minter.mint.pubkey(),
        program_test_context.last_blockhash,
    );

    let lock_proposer_tokens_tx = get_lock_tokens_transaction(
        &proposer,
        &locker_result.locker,
        &minter.mint.pubkey(),
        1_000_000_000,
        (SECONDS_PER_DAY * 365) as i64, // lockup for a year
        program_test_context.last_blockhash,
    );
    program_test_context
        .banks_client
        .process_transactions(Vec::from([
            create_proposer_escrow_tx,
            lock_proposer_tokens_tx,
        ]))
        .await
        .unwrap();

    // create proposal id
    let mut hasher = Sha3_256::new();
    hasher.update("a".as_bytes());
    let hasher_final = hasher.finalize();
    let proposal_id = hasher_final.as_slice().to_vec();
    propose_vote(
        &mut program_test_context,
        &proposal_id,
        &proposer,
        &minter.mint.pubkey(),
        &config_account_pda,
    )
    .await;

    // ====== LOCK VOTER TOKENS ======
    let create_escrow_txs = voters.get_create_escrow_tx(
        &locker_result.locker,
        &minter.mint.pubkey(),
        program_test_context.last_blockhash,
    );
    program_test_context
        .banks_client
        .process_transactions(create_escrow_txs)
        .await
        .unwrap();

    let lock_voters_tokens_txs = voters.lock_tokens(
        &locker_result.locker,
        &minter.mint.pubkey(),
        1_000_000_000,
        (SECONDS_PER_DAY * 4 * 365) as i64, // lock in days
        program_test_context.last_blockhash,
    );
    for tx in &lock_voters_tokens_txs {
        program_test_context
            .banks_client
            .process_transaction(tx.clone())
            .await
            .unwrap();
    }

    // VOTERS vote on proposal
    let number_of_voters = voters.keys.len() as u32;
    let votes: Vec<u32> = (0..number_of_voters).collect();
    voters.set_votes(votes);
    let submit_vote_txs = voters.get_submit_vote_txs(
        &get_proposal_pda(&proposal_id).0,
        &get_proposal_vault_pda(&proposal_id).0,
        &locker_result.locker,
        &minter.get_mint_pubkey(),
        program_test_context.last_blockhash,
    );
    for tx in &submit_vote_txs {
        program_test_context
            .banks_client
            .process_transaction(tx.clone())
            .await
            .unwrap();
    }
    let vote_account_data = voters
        .get_vote_account(
            &mut program_test_context,
            &voters.keys[0].pubkey(),
            &get_proposal_pda(&proposal_id).0,
        )
        .await;

    assert!(
        vote_account_data.vote_power != 0,
        "[main] vote power for {} is {}",
        vote_account_data.owner.to_string(),
        vote_account_data.staked,
    );

    let proposal =
        get_oracle_proposal(&mut program_test_context, &get_proposal_pda(&proposal_id).0)
            .await
            .unwrap();
    assert!(
        proposal.status == 3,
        "[main] Assert failed. Proposal status is not 3 but {}",
        proposal.status
    );
    assert!(
        proposal.votes != 0,
        "[main] suspicious. Number of votes is {}",
        proposal.votes
    );
    println!(
        "[main] number of votes: {} / {}",
        proposal.votes, proposal.required_votes
    );

    // fast forward time beyond voting period
    test_foward_time(&mut program_test_context, proposal.vote_end_at - 1).await;
    let proposal_account =
        get_oracle_proposal(&mut program_test_context, &get_proposal_pda(&proposal_id).0)
            .await
            .unwrap();
    let proposal_status = proposal_account.get_status(proposal.vote_end_at - 1);
    let can_reveal_vote = proposal_account.can_reveal_vote(proposal.vote_end_at - 1);
    println!(
        "[Proposal status] Has reached quorum {:?}, is blind voting {}",
        proposal.has_reached_quorum(),
        proposal.is_blind_vote_ongoing_at_time(proposal.vote_end_at - 1)
    );
    println!(
        "[Proposal status] Current status is {:?}",
        &proposal_status.get_id()
    );
    println!(
        "[Proposal status] Can reveal vote: {:?}",
        can_reveal_vote.unwrap()
    );

    // fast forward to
    // try to reveal vote
    let reveal_vote_txs =
        voters.get_reveal_vote_txs(&proposal_id, program_test_context.last_blockhash);

    for tx in &reveal_vote_txs {
        program_test_context
            .banks_client
            .process_transaction(tx.clone())
            .await
            .unwrap();
    }

    // Finalize the vote
    let finalize_vote_accounts = oracle::accounts::FinalizeVoteResults {
        finalizer: proposer.pubkey(),
        proposal: get_proposal_pda(&proposal_id).0,
        revealed_votes: get_reveal_vote_array_pda(&proposal_id).0,
        system_program: anchor_lang::system_program::ID,
    };

    let finalize_vote_data = oracle::instruction::FinalizeVoteResults {};
    let finalize_vote_tx = transaction::Transaction::new_signed_with_payer(
        &[instruction::Instruction {
            program_id: oracle::id(),
            accounts: finalize_vote_accounts.to_account_metas(None),
            data: finalize_vote_data.data(),
        }],
        Some(&proposer.pubkey()),
        &[&proposer],
        program_test_context.last_blockhash,
    );

    program_test_context
        .banks_client
        .process_transaction(finalize_vote_tx)
        .await
        .unwrap();
}
