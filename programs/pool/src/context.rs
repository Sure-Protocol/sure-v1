use crate::states::{
    bitmap::BitMap,
    contract::InsuranceContract,
    liquidity::{self, LiquidityPosition},
    owner::ProtocolOwner,
    pool::{PoolAccount, PoolManager, SurePools},
    tick::{Tick, TickTrait},
};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    mint,
    token::{Mint, Token, TokenAccount},
};

use std::mem::size_of;
use vipers::{assert_is_ata, prelude::*};

pub const SURE_PRIMARY_POOL_SEED: &str = "sure-insurance-pool";
pub const SURE_ASSOCIATED_TOKEN_ACCOUNT_SEED: &str = "sure-ata";
pub const SURE_PREMIUM_POOL_SEED: &str = "sure-premium-vault";
pub const SURE_VAULT_POOL_SEED: &str = "sure-liquidity-vault";
pub const SURE_LIQUIDITY_POSITION: &str = "sure-lp";
pub const SURE_PROTOCOL_OWNER: &str = "sure-protocol-owner";
pub const SURE_INSURANCE_CONTRACT: &str = "sure-insurance-contract";
pub const SURE_INSURANCE_CONTRACTS: &str = "sure-insurance-contracts";
pub const SURE_BITMAP: &str = "sure-bitmap";
pub const SURE_TICK_SEED: &str = "sure-tick";
pub const SURE_NFT_MINT_SEED: &str = "sure-nft";
pub const SURE_TOKEN_ACCOUNT_SEED: &str = "sure-token-account";
pub const SURE_MP_METADATA_SEED: &str = "metadata";
pub const SURE_POOLS_SEED: &str = "sure-pools";

/// Initialize Sure Protocol
/// by setting the owner of the protocol
#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
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

    /// Sure Pools holding information about which protocols are insured
    #[account(
        init,
        payer = owner,
        space = SurePools::SIZE,
        seeds = [
            SURE_POOLS_SEED.as_bytes(),
        ],
        bump
    )]
    pub pools: Box<Account<'info, SurePools>>,

    /// Sure pool program
    pub program: Program<'info,crate::program::SurePool>,

    pub program_data: Account<'info,ProgramData>,

    /// System Program to create a new account
    pub system_program: Program<'info, System>,
}


impl<'info> Validate<'info> for InitializeProtocol<'info> {
    fn validate(&self) -> Result<()> {
        assert_eq!(self.program.programdata_address()?,Some(self.program_data.key()));
        assert_eq!(Some(self.owner.key()), self.program_data.upgrade_authority_address);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePoolManager<'info> {
    // Account for keeping track of the pool manager
    #[account(
        init,
        payer= initial_manager,
        space = 8 + PoolManager::SIZE,
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
    pub protocol_owner: AccountLoader<'info, ProtocolOwner>,

    #[account(
        init,
        space = 8 + PoolAccount::SPACE,
        payer = pool_creator,
        seeds = [
            SURE_PRIMARY_POOL_SEED.as_bytes(),
            insured_token_account.key().to_bytes().as_ref(),
        ],
        bump
    )]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Sure Pools keeps an overview over
    /// existing pools
    #[account(mut)]
    pub sure_pools: Box<Account<'info, SurePools>>,

    // Assume the contract being hacked is a token account
    /// CHECK: This accounts represents the executable contract
    /// that is to be insured.
    #[account(
        constraint = insured_token_account.executable == false
    )]
    pub insured_token_account: UncheckedAccount<'info>,

    /// Sysvar for Associated Token Account
    pub rent: Sysvar<'info, Rent>,

    /// Provide the system program
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for CreatePool<'info> {
    fn validate(&self) -> Result<()> {
        // To start, only the protocol owner can create a pool
        //assert_keys_eq!(self.pool_creator, self.protocol_owner);
        //
        //
        assert_keys_eq!(self.pool.smart_contract, self.insured_token_account);

        Ok(())
    }
}

/// Create Pool Vaults
/// creates the associated pool vault
/// based on token mint
#[derive(Accounts)]
pub struct CreatePoolVaults<'info> {
    // Signer of the creation
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Pool account that the vaults are associated to
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Token mint used for the Vaults
    pub token_mint: Box<Account<'info, Mint>>,

    // Pool Vault used to hold tokens from token_mint
    #[account(
        init,
        payer = creator,
        seeds = [
            SURE_VAULT_POOL_SEED.as_bytes(),
            pool.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        token::mint = token_mint,
        token::authority = pool,
    )]
    pub liquidity_vault: Box<Account<'info, TokenAccount>>,

    // Premium Vault holding all future premiums
    #[account(
        init,
        payer = creator,
        seeds = [
            SURE_PREMIUM_POOL_SEED.as_bytes(),
            pool.key().as_ref(),
            token_mint.key().as_ref()
        ],
        bump,
        token::mint = token_mint,
        token::authority = pool
    )]
    pub premium_vault: Box<Account<'info, TokenAccount>>,

    /// Bitmap
    /// Keep track of ticks used to provide liquidity at
    #[account(
        init,
        space = 8 + BitMap::SPACE,
        payer = creator,
        seeds = [
            SURE_BITMAP.as_bytes(),
            pool.key().as_ref(),
            token_mint.key().as_ref()
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

impl<'info> Validate<'info> for CreatePoolVaults<'info> {
    fn validate(&self) -> Result<()> {
        // Make sure that token is USDC
        assert_eq!(
            self.token_mint.key(),
            mint::USDC,
            "Vaults can only have mint USDC"
        );

        Ok(())
    }
}

/// Deposit Liquidity into an exisitng pool
#[derive(Accounts)]
#[instruction(tick: u16,tick_pos: u64)]
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
    pub vault: Account<'info, TokenAccount>,

    // NFT minting
    #[account(
        init,
        seeds = [
            SURE_NFT_MINT_SEED.as_ref(),
            nft_account.key().as_ref()
            ],
        bump,
        mint::decimals = 0,
        mint::authority = protocol_owner,
        payer = liquidity_provider,
    )]
    pub nft_mint: Box<Account<'info, Mint>>,

    /// CHECK: done in method
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// Program id for metadata program
    /// CHECK: checks that the address matches the mpl token metadata id
    #[account(address =mpl_token_metadata::ID )]
    pub metadata_program: UncheckedAccount<'info>,

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
            vault.key().as_ref(),
            tick.to_le_bytes().as_ref(),
            tick_pos.to_le_bytes().as_ref(),
        ],
        bump,
        token::mint = nft_mint,
        token::authority = liquidity_provider,
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

        // check the same bitmap
        assert_keys_eq!(self.pool.bitmap, self.bitmap);
        Ok(())
    }
}

/// Redeem liquidity
/// Allow holder of NFT to redeem liquidity from pool
#[derive(Accounts)]
pub struct RedeemLiquidity<'info> {
    /// Holder of the LP NFT
    pub nft_holder: Signer<'info>,

    /// NFT that proves ownership of position
    #[account(
        constraint = nft_account.mint ==liquidity_position.nft_mint
    )]
    pub nft_account: Box<Account<'info, TokenAccount>>,

    /// Protocol owner as the authority of mints
    pub protocol_owner: AccountLoader<'info, ProtocolOwner>,

    /// Liquidity position
    #[account(mut)]
    pub liquidity_position: Box<Account<'info, LiquidityPosition>>,

    /// Token account to recieve the tokens
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Pool Vault to transfer tokens from
    #[account(mut)]
    pub vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub tick_account: AccountLoader<'info, Tick>,

    /// CHECK: Account used to hold metadata on the LP NFT
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,

    /// CHECK: Checks that the address is the metadata metaplex program
    #[account(address = mpl_token_metadata::ID)]
    pub metadata_program: AccountInfo<'info>,

    /// Sure Protocol Pool Account
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

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

#[derive(Accounts)]
#[instruction(pool: Pubkey,token: Pubkey,tick: u16)]
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
            tick.to_le_bytes().as_ref()
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
/// Initialize a new insurance contract with pool
#[derive(Accounts)]
pub struct InitializeInsuranceContract<'info> {
    /// Signer of contract
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Pool to buy insurance from
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Token mint used to insure with
    pub token_mint: Box<Account<'info, Mint>>,

    /// Tick account to insure against
    pub tick_account: AccountLoader<'info, Tick>,

    /// Insurance Contract
    #[account(
        init,
        space = 8 + InsuranceContract::SPACE,
        payer = owner,
        seeds = [
            SURE_INSURANCE_CONTRACT.as_bytes(),
            owner.key().as_ref(),
            tick_account.key().as_ref(),
        ],
        bump,
    )]
    pub insurance_contract: Box<Account<'info, InsuranceContract>>,

    /// Insurance positions
    #[account(mut)]
    pub insurance_contracts: Box<Account<'info, BitMap>>,

    /// System Contract used to create accounts
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for InitializeInsuranceContract<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Initialize user insurance contract
/// The account is used to have an overview over the
/// positions hebild by the user
#[derive(Accounts)]
pub struct InitializeUserInsuranceContracts<'info> {
    /// Signer
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Pool associated with the insurance contracts
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Token mint used for the insurance contracts
    pub token_mint: Box<Account<'info, Mint>>,

    ///
    #[account(
        init,
        space = 8 + BitMap::SPACE,
        payer = signer,
        seeds = [
            SURE_INSURANCE_CONTRACTS.as_bytes(),
            signer.key().as_ref(),
            pool.key().as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub insurance_contracts: Box<Account<'info, BitMap>>,

    /// System program
    pub system_program: Program<'info, System>,
}

/// Buy Insurance Request
///
#[derive(Accounts)]
pub struct BuyInsurance<'info> {
    /// Buyer
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// Account to buy tokens with
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Pool to buy from
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Tick account to buy
    #[account(mut)]
    pub tick_account: AccountLoader<'info, Tick>,

    /// Premium Vault
    #[account(
        mut,
        constraint = premium_vault.owner ==  pool.key(),
        constraint = premium_vault.mint == token_account.mint,
    )]
    pub premium_vault: Box<Account<'info, TokenAccount>>,

    /// Insurance Contract
    #[account(mut,
    constraint = insurance_contract.pool == pool.key(),
    constraint = insurance_contract.owner == buyer.key(),
    )]
    pub insurance_contract: Box<Account<'info, InsuranceContract>>,

    /// Token program, needed to transfer tokens
    pub token_program: Program<'info, Token>,

    /// System Contract used to create accounts
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for BuyInsurance<'info> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Cancel insurance contract
///
#[derive(Accounts)]
pub struct ReduceInsuranceAmount<'info> {
    /// Holder of insurance
    #[account(mut)]
    pub holder: Signer<'info>,

    /// Pool to exit
    #[account(mut)]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Tick account to exit from
    #[account(mut)]
    pub tick_account: AccountLoader<'info, Tick>,

    /// Account to deposit tokens into
    #[account(mut)]
    pub token_account: Box<Account<'info, TokenAccount>>,

    /// Premium Vault
    #[account(
        mut,
        constraint = premium_vault.owner ==  pool.key(),
        constraint = premium_vault.mint == insurance_contract.token_mint,
        constraint = premium_vault.mint == token_account.mint,
    )]
    pub premium_vault: Box<Account<'info, TokenAccount>>,

    /// Insurance Contract
    #[account(mut,
        constraint = insurance_contract.pool == pool.key(),
        constraint = insurance_contract.active == true,
        constraint = insurance_contract.owner == holder.key(),
        constraint = insurance_contract.tick_account == tick_account.key(),
        constraint = insurance_contract.owner == holder.key(),
    )]
    pub insurance_contract: Box<Account<'info, InsuranceContract>>,

    /// TOken program used to transfer tokens
    pub token_program: Program<'info, Token>,

    /// System Contract used to create accounts
    pub system_program: Program<'info, System>,
}
