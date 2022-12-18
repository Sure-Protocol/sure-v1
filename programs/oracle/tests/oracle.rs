use anchor_client::solana_sdk::*;
use anchor_lang::prelude::*;
use oracle::oracle::id;
use solana_program_test::*;

#[tokio::test]
async fn create_and_init() {
    let program_test = ProgramTest::new("oracle", id(), processor!(oracle::entry));
}
