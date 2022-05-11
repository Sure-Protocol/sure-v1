use crate::states::{
    bitmap::BitMap,
    contract::InsuranceContract,
    liquidity::{self, LiquidityPosition},
    owner::ProtocolOwner,
    pool::{PoolAccount, PoolManager},
    tick::{Tick, TickTrait},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use std::mem::size_of;
use vipers::{assert_is_ata, prelude::*};

pub const SURE_PRIMARY_POOL_SEED: &str = "sure-insurance-pool";
pub const SURE_ASSOCIATED_TOKEN_ACCOUNT_SEED: &str = "sure-ata";
pub const SURE_LIQUIDITY_POSITION: &str = "sure-lp";
pub const SURE_PROTOCOL_OWNER: &str = "sure-protocol-owner";
pub const SURE_INSURANCE_CONTRACT: &str = "sure-insurance-contract";
pub const SURE_BITMAP: &str = "sure-bitmap";
pub const SURE_TICK_SEED: &str = "sure-tick";
pub const SURE_NFT_MINT_SEED: &str = "sure-nft";
pub const SURE_TOKEN_ACCOUNT_SEED: &str = "sure-token-account";

/// Initialize Sure Protocol
/// by setting the owner of the protocol
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Owner of the protocol
    #[account(mut)]
    pub owner: Signer<'info>,

    ///
    #[account(
        init,
        seeds=[],
        bump,
        payer = owner,
        space = 8 + ProtocolOwner::SPACE,
    )]
    pub protocol_owner: AccountLoader<'info, ProtocolOwner>,

    /// System Program to create a new account
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for Initialize<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePoolManager<'info> {
    // Account for keeping track of the pool manager
    #[account(
        init,
        payer= initial_manager,
        space = 8 + PoolManager::POOL_MANAGER_SIZE,
        seeds = [b"sure-pool-manager"],
        bump
    )]
    pub manager: Account<'info, PoolManager>,

    // The signer becomes the initial manager
    #[account(mut)]
    pub initial_manager: Signer<'info>,

    // System program
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for InitializePoolManager<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.manager.owner, self.initial_manager);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePool<'info> {
    /// Pool creator
    #[account(mut)]
    pub pool_creator: Signer<'info>,

    /// Protocol owner
    /// !!!ISSUE
    pub protocol_owner: AccountLoader<'info, ProtocolOwner>,

    #[account(
        init,
        space = 8 + PoolAccount::SPACE,
        payer = pool_creator,
        seeds = [
            SURE_PRIMARY_POOL_SEED.as_bytes(),
            token.key().as_ref(),
            insured_token_account.key().to_bytes().as_ref(),
        ],
        bump
    )]
    pub pool: Box<Account<'info, PoolAccount>>,

    // Assume the contract being hacked is a token account
    /// CHECK: This accounts represents the executable contract
    /// that is to be insured.
    #[account(
        constraint = insured_token_account.executable == false
    )]
    pub insured_token_account: UncheckedAccount<'info>,

    // Initialized Associated token accoun to hold vault tokens
    #[account(
        init,
        payer = pool_creator,
        seeds = [
            SURE_ASSOCIATED_TOKEN_ACCOUNT_SEED.as_bytes(),
            pool.key().as_ref(),
            token.key().as_ref()
        ],
        bump,
        token::mint = token,
        token::authority = vault
    )]
    pub vault: Box<Account<'info, TokenAccount>>,

    /// Token to be deposited into the pool
    pub token: Box<Account<'info, Mint>>,

    /// Bitmap
    /// Keep track of ticks used to provide liquidity at
    #[account(
        init,
        space = 8 + BitMap::SPACE,
        payer = pool_creator,
        seeds = [
            SURE_BITMAP.as_bytes(),
            pool.key().as_ref(),
            token.key().as_ref()
        ],
        bump,
    )]
    pub bitmap: Box<Account<'info, BitMap>>,

    /// Sysvar for Associated Token Account
    pub rent: Sysvar<'info, Rent>,

    // Token program
    pub token_program: Program<'info, Token>,

    /// Provide the system program
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for CreatePool<'info> {
    fn validate(&self) -> Result<()> {
        // To start, only the protocol owner can create a pool
        assert_keys_eq!(self.pool_creator, self.protocol_owner);
        //
        assert_keys_eq!(self.pool.token, self.token);
        //
        assert_keys_eq!(self.pool.bitmap, self.bitmap);
        //
        assert_keys_eq!(self.pool.smart_contract, self.insured_token_account);
        assert_keys_eq!(self.pool.vault, self.vault);

        Ok(())
    }
}

/// Deposit Liquidity into an exisitng pool
#[derive(Accounts)]
#[instruction(tick: u64,tick_pos: u64)]
pub struct DepositLiquidity<'info> {
    /// Liquidity provider
    #[account(mut)]
    pub liquidity_provider: Signer<'info>,

    /// Protocol owner as the authority of mints
    pub protocol_owner: AccountLoader<'info, ProtocolOwner>,

    /// Associated token accoun to credit
    #[account(mut)]
    pub liquidity_provider_account: Box<Account<'info, TokenAccount>>,

    /// Pool to provide liquidity to
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Pool Vault account to deposit liquidity to
    #[account(mut)]
    pub token_vault: Box<Account<'info, TokenAccount>>,

    // NFT minting
    #[account(
        init,
        seeds = [
            SURE_NFT_MINT_SEED.as_ref(),
            pool.key().as_ref(),
            token_vault.key().as_ref(),
            tick.to_le_bytes().as_ref(),
            tick_pos.to_le_bytes().as_ref(),
            ],
        bump,
        mint::decimals = 0,
        mint::authority = protocol_owner,
        payer = liquidity_provider,
    )]
    pub nft_mint: Box<Account<'info, Mint>>,

    /// Create Liquidity position
    /// HASH: [sure-lp,liquidity-provider,pool,token,tick]
    #[account(
        init,
        payer = liquidity_provider,
        seeds = [
            SURE_LIQUIDITY_POSITION.as_bytes(),
            nft_account.key().as_ref()
        ],
        space = 8 + LiquidityPosition::SPACE,
        bump,
    )]
    pub liquidity_position: Box<Account<'info, LiquidityPosition>>,

    /// Account to deposit NFT into
    #[account(
        init,
        seeds =
        [
            SURE_TOKEN_ACCOUNT_SEED.as_bytes().as_ref(),
            pool.key().as_ref(),
            token_vault.key().as_ref(),
            tick.to_le_bytes().as_ref(),
            tick_pos.to_le_bytes().as_ref(),
        ],
        bump,
        token::mint = nft_mint,
        token::authority = liquidity_provider_account,
        payer = liquidity_provider,
    )]
    pub nft_account: Box<Account<'info, TokenAccount>>,

    /// Bitmap representing liquidity at
    /// different ticks
    #[account(mut)]
    pub bitmap: Box<Account<'info, BitMap>>,

    /// Tick contains information on liquidity at
    /// one specific tick
    #[account(mut)]
    pub tick_account: AccountLoader<'info, Tick>,

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
        assert_is_zero_token_account!(self.nft_account);

        // Check correct vault
        assert_keys_eq!(self.pool.vault, self.token_vault);

        // check the same bitmap
        assert_keys_eq!(self.pool.bitmap, self.bitmap);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct UpdateTickPosition<'info> {
    /// Pays for the update
    pub signer: Signer<'info>,

    /// Tick account
    #[account(mut)]
    pub tick: AccountLoader<'info, Tick>,

    /// Liquidity Position
    #[account(mut)]
    pub liquidity_position: Box<Account<'info, LiquidityPosition>>,
}

/// Redeem liquidity
/// Allow holder of NFT to redeem liquidity from pool
#[derive(Accounts)]
pub struct RedeemLiquidity<'info> {
    /// Holder of the LP NFT
    pub nft_holder: Signer<'info>,

    /// NFT that proves ownership of position
    #[account(
        constraint = nft.mint ==liquidity_position.nft_mint
    )]
    pub nft: Box<Account<'info, TokenAccount>>,

    /// Mint of the NFT
    #[account(mut)]
    pub nft_mint: Account<'info, Mint>,

    /// Liquidity position
    #[account(mut)]
    pub liquidity_position: Box<Account<'info, LiquidityPosition>>,

    /// Token account to recieve the tokens at
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Pool Vault to transfer tokens from
    pub vault_account: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub tick_account: AccountLoader<'info, Tick>,

    /// Sure Protocol Pool Account
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Sure owner
    pub protocol_owner: AccountLoader<'info, ProtocolOwner>,
}

impl<'info> Validate<'info> for RedeemLiquidity<'info> {
    fn validate(&self) -> Result<()> {
        assert_is_zero_token_account!(self.nft);

        // Check correct vault
        assert_keys_eq!(self.pool.vault, self.vault_account);

        // check the same bitmap
        //assert_keys_eq!(self.pool.bitmap, self.bitmap);
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(pool: Pubkey,token: Pubkey,tick_bp: u64)]
pub struct InitializeTick<'info> {
    /// Signer of the transaction
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Create tick account
    #[account(
        init,
        payer = creator,
        seeds = [
            SURE_TICK_SEED.as_bytes(),
            pool.key().as_ref(),
            token.key().as_ref(),
            tick_bp.to_le_bytes().as_ref()
        ],
        bump,
        space = 8 + size_of::<Tick>(),
    )]
    pub tick_account: AccountLoader<'info, Tick>,

    /// System program required to make changes
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseTick<'info> {
    // Account to receive remaining rent
    pub recipient: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        close = recipient,
    )]
    pub tick_account: AccountLoader<'info, Tick>,
}

/// Buy Insurance Request
///
#[derive(Accounts)]
pub struct BuyInsurance<'info> {
    /// Buyer
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// Pool to buy from
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Insurance Position
    #[account(
        init,
        space = 8 + InsuranceContract::SPACE,
        payer = buyer,
        seeds = [
            SURE_INSURANCE_CONTRACT.as_bytes(),
            pool.key().as_ref(),
        ],
        bump,
    )]
    pub insurance_contract: Box<Account<'info, InsuranceContract>>,

    /// System Contract used to create accounts
    pub system_program: Program<'info, System>,
}
