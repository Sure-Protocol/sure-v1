///! Liquidity positions
///!
use anchor_lang::prelude::*;

use crate::states::{
    bitmap::BitMap,
    owner::ProtocolOwner,
    pool::{PoolAccount,},
    seeds::{SURE_NFT_MINT_SEED,SURE_LIQUIDITY_POSITION,SURE_TOKEN_ACCOUNT_SEED},
    tick::Tick,
};

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use vipers::{assert_is_ata, prelude::*};

use super::pool::TokenPool;

/// -- Liquidity Position --
/// 
/// Holds information about liquidity at a given tick
///
#[account]
#[derive(Default)]
pub struct LiquidityPosition {
    /// Bump Identity
    pub bump: u8, // 1byte

    /// The amount of liquidity provided in lamports
    pub liquidity: u64, // 8 bytes

    /// the amount of liquidity used
    pub used_liquidity: u64, // 8 bytes

    /// Liquidity Pool
    pub pool: Pubkey, // 32 bytes

    // TokenMint representing the position
    pub token_mint: Pubkey,

    /// Mint of token provided
    pub nft_account: Pubkey, // 32 bytes

    /// NFT mint. The mint representing the position
    /// The NFT is the owner of the position.
    pub nft_mint: Pubkey, // 32 bytes

    /// Time at liquidity position creation
    pub created_at: i64, // 8 bytes,

    /// Id in the tick pool
    pub tick_id: u8,

    /// The tick that the liquidity is at
    pub tick: u16, // 8 bytes

    /// Outstanding Rewards
    pub outstanding_rewards: u32, // 4 bytes
}

impl LiquidityPosition {
    pub const SPACE: usize = 1 + 8 + 8 + 32 + 32 + 32 + 32 + 8 + 1 + 8 + 4;
}

// ?---------------------------------.----------------- //
// ?%%%%%%%%%%%%%%%% Method Accounts %%%%%%%%%%%%%%%%! //
// ?-------------------------------------------------- //


/// --- Deposit Liquidity ---
/// 
/// Deposits liquidity into a 
/// 
/// Liquidity Positions on Sure is represented as an NFT. 
/// The holder has the right to manage the liquidity position
/// 
/// The associated method does
///     - Mint a new Liquidity Position NFT 
///     - Transfer capital to pool vault
///     - Creates liquidity position 
///     - Updates 
/// 
/// Initializes:
///     - nft_mint: Mint associated with the liquidity NFT position
///     - liquidity_position: keeps a summary of liquidity position
///     - nft_account: nft account to hold the newly minted Liquidity position NFT
/// 
#[derive(Accounts)]
#[instruction(tick: u16,tick_pos: u64)]
pub struct DepositLiquidity<'info> {
    /// Liquidity provider
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    /// Protocol owner as the authority of mints
    pub protocol_owner: Account<'info, ProtocolOwner>,

    /// Associated token account to credit
    #[account(mut)]
    pub liquidity_provider_ata: Box<Account<'info, TokenAccount>>,

    /// Pool which owns token account
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Token pool account which holds overview
    #[account(mut)]
    pub token_pool: Box<Account<'info,TokenPool>>,

    /// Pool Vault account to deposit liquidity to
    #[account(mut)]
    pub pool_vault: Account<'info, TokenAccount>,

    /// CHECK: done in method
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// Program id for metadata program
    /// CHECK: checks that the address matches the mpl token metadata id
    //#[account(address =mpl_token_metadata::ID )]
    pub metadata_program: UncheckedAccount<'info>,

    /// Create Liquidity position
    /// HASH: [sure-lp,liquidity-provider,pool,token,tick]
    #[account(
        init,
        payer = liquidity_provider,
        seeds = [
            SURE_LIQUIDITY_POSITION.as_bytes(),
            liquidity_position_nft_account.key().as_ref()
        ],
        space = 8 + LiquidityPosition::SPACE,
        bump,
    )]
    pub liquidity_position: Box<Account<'info, LiquidityPosition>>,

     // NFT minting
     #[account(
        init,
        seeds = [
            SURE_NFT_MINT_SEED.as_ref(),
            liquidity_position_nft_account.key().as_ref()
            ],
        bump,
        mint::decimals = 0,
        mint::authority = protocol_owner,
        payer = liquidity_provider,
    )]
    pub liquidity_position_nft_mint: Box<Account<'info, Mint>>,

    /// Account to deposit NFT into
    #[account(
        init,
        seeds =
        [
            SURE_TOKEN_ACCOUNT_SEED.as_bytes().as_ref(),
            pool.key().as_ref(),
            pool_vault.key().as_ref(),
            tick.to_le_bytes().as_ref(),
            tick_pos.to_le_bytes().as_ref(),
        ],
        bump,
        token::mint = liquidity_position_nft_mint,
        token::authority = liquidity_provider,
        payer = liquidity_provider,
    )]
    pub liquidity_position_nft_account: Box<Account<'info, TokenAccount>>,

    /// Pool Liquidity Tick Bitmap
    /// 
    /// Holds information on which ticks that contains 
    /// available liquidity
    #[account(mut)]
    pub pool_liquidity_tick_bitmap: Box<Account<'info, BitMap>>,

    /// Tick contains information on liquidity at
    /// one specific tick
    #[account(mut)]
    pub liquidity_tick_info: AccountLoader<'info, Tick>,

    /// Sysvar for token mint and ATA creation
    pub rent: Sysvar<'info, Rent>,

    // Token program that executes the transfer
    pub token_program: Program<'info, Token>,

    /// Provide the system program
    pub system_program: Program<'info, System>,

    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Validate<'info> for DepositLiquidity<'info> {
    fn validate(&self) -> Result<()> {
        assert_is_zero_token_account!(self.liquidity_position_nft_account);

        // check the same bitmap
        //assert_keys_eq!(self.pool.pool_liquidity_tick_bitmap, self.pool_liquidity_tick_bitmap);
        Ok(())
    }
}

/// Redeem liquidity
/// Allow holder of NFT to redeem liquidity from pool
#[derive(Accounts)]
pub struct RedeemLiquidity<'info> {
    /// Holder of the LP NFT
    pub nft_holder: Signer<'info>,

    /// Sure Protocol Pool Account
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Token pool account which holds overview
    #[account(mut)]
    pub token_pool: Box<Account<'info,TokenPool>>,

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

impl<'info> Validate<'info> for RedeemLiquidity<'info> {
    fn validate(&self) -> Result<()> {
        //assert_is_zero_token_account!(self.nft);

        // Check correct vault

        // check the same bitmap
        //assert_keys_eq!(self.pool.bitmap, self.bitmap);
        Ok(())
    }
}



#[event]
pub struct NewLiquidityPosition {
    pub tick: u16,
    pub liquidity: u64,
}
