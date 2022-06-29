use crate::states::*;
use crate::utils::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};
use mpl_token_metadata::{instruction::update_metadata_accounts_v2, state::Creator};
use vipers::*;
/// Redeem liquidity
/// Allow holder of NFT to redeem liquidity from pool
#[derive(Accounts)]
pub struct DecreaseLiquidityPosition<'info> {
    /// Holder of the LP NFT
    pub nft_holder: Signer<'info>,

    /// Sure Protocol Pool Account
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Token pool account which holds overview
    #[account(mut)]
    pub token_pool: Box<Account<'info, TokenPool>>,

    /// NFT that proves ownership of position
    #[account(
        constraint = liquidity_position_nft_account.mint ==liquidity_position.nft_mint
    )]
    pub liquidity_position_nft_account: Box<Account<'info, TokenAccount>>,

    /// Protocol owner as the authority of mints
    pub protocol_owner: Account<'info, ProtocolOwner>,

    /// Liquidity position
    #[account(mut)]
    pub liquidity_position: Box<Account<'info, LiquidityPosition>>,

    /// Token account to recieve the tokens
    #[account(mut)]
    pub liquidity_provider_ata: Box<Account<'info, TokenAccount>>,

    /// Pool Vault to transfer tokens from
    #[account(mut)]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    /// Pool Liquidity Tick Bitmap
    ///
    /// Holds information on which ticks that contains
    /// available liquidity
    #[account(mut)]
    pub pool_liquidity_tick_bitmap: Box<Account<'info, BitMap>>,
    #[account(mut)]
    pub liquidity_tick_info: AccountLoader<'info, Tick>,

    /// CHECK: Account used to hold metadata on the LP NFT
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,

    /// CHECK: Checks that the address is the metadata metaplex program
    #[account(address = mpl_token_metadata::ID)]
    pub metadata_program: AccountInfo<'info>,

    // Token program that executes the transfer
    pub token_program: Program<'info, Token>,

    /// Provide the system program
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for DecreaseLiquidityPosition<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

pub fn handler(ctx: Context<DecreaseLiquidityPosition>) -> Result<()> {
    let liquidity_position = &ctx.accounts.liquidity_position;

    let tick_account_state =
        AccountLoader::<tick::Tick>::try_from(&ctx.accounts.liquidity_tick_info.to_account_info())?;
    let mut liquidity_tick_info = tick_account_state.load_mut()?;
    let pool_liquidity_tick_bitmap = &mut ctx.accounts.pool_liquidity_tick_bitmap;
    let protocol_owner = &ctx.accounts.protocol_owner;
    let pool = &mut ctx.accounts.pool;
    let token_pool = &mut ctx.accounts.token_pool;

    // Available liquidity
    let free_liquidity = liquidity_tick_info.available_liquidity(liquidity_position.tick_id);
    require!(free_liquidity > 0, errors::SureError::LiquidityFilled);

    // _______________ Functionality _______________

    let pool_seeds = [
        &SURE_PRIMARY_POOL_SEED.as_bytes() as &[u8],
        &pool.smart_contract.to_bytes() as &[u8],
        &[pool.bump],
    ];

    // # 1 Transfer excess liquidity back to nft holder
    transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info().clone(),
            Transfer {
                from: ctx.accounts.pool_vault.to_account_info().clone(),
                to: ctx
                    .accounts
                    .liquidity_provider_ata
                    .to_account_info()
                    .clone(),
                authority: ctx.accounts.pool.to_account_info().clone(),
            },
            &[&pool_seeds[..]],
        ),
        free_liquidity,
    )?;

    // # 2. Update nft metadata to reflect token
    let updated_metaplex = mpl_token_metadata::state::DataV2 {
        name: String::from("Sure LP NFT V1"),
        symbol: String::from("SURE-LP"),
        uri: format!("https://sure.claims"),
        seller_fee_basis_points: 0,
        creators: Some(vec![Creator {
            address: ctx.accounts.protocol_owner.key(),
            verified: true,
            share: 100,
        }]),
        collection: None,
        uses: None,
    };
    let update_metaplex_metadata_ix = update_metadata_accounts_v2(
        ctx.accounts.metadata_program.key(),
        ctx.accounts.metadata_account.key(),
        ctx.accounts.protocol_owner.key(),
        None,
        Some(updated_metaplex),
        None,
        None,
    );

    program::invoke_signed(
        &update_metaplex_metadata_ix,
        &[
            ctx.accounts.metadata_account.to_account_info().clone(),
            ctx.accounts.protocol_owner.to_account_info().clone(),
            ctx.accounts.system_program.to_account_info().clone(),
        ],
        &[&[&[protocol_owner.bump]]],
    )?;

    // Update Token Pool
    token_pool.liquidity -= free_liquidity;

    // # 3 Update tick poo
    liquidity_tick_info
        .remove_liquidity(liquidity_position.tick_id)
        .map_err(|e| e.to_anchor_error())?;

    if !liquidity_tick_info.active {
        if pool_liquidity_tick_bitmap.is_initialized(liquidity_position.tick) {
            pool_liquidity_tick_bitmap.flip_bit(liquidity_position.tick);
        }
    }
    Ok(())
}
