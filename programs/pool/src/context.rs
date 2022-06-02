use crate::states::{
    bitmap::BitMap,
    contract::{InsuranceTickContract,PoolInsuranceContract},
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
pub const SURE_INSURANCE_CONTRACTS_BITMAP: &str = "sure-insurance-contracts-bitmap";
pub const SURE_INSURANCE_CONTRACTS_INFO: &str = "sure-insurance-contracts-info";
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
    pub protocol_owner: Account<'info, ProtocolOwner>,

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

    /// System Program to create a new account
    pub system_program: Program<'info, System>,
}


impl<'info> Validate<'info> for InitializeProtocol<'info> {
    fn validate(&self) -> Result<()> {
      //  assert_eq!(self.program.programdata_address()?,Some(self.program_data.key()));
        //assert_eq!(Some(self.owner.key()), self.program_data.upgrade_authority_address);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializePoolManager<'info> {
    // The signer becomes the initial manager
    #[account(mut)]
    pub initial_manager: Signer<'info>,

    // Account for keeping track of the pool manager
    #[account(
        init,
        payer= initial_manager,
        space = 8 + PoolManager::SIZE,
        seeds = [b"sure-pool-manager"],
        bump
    )]
    pub manager: Account<'info, PoolManager>,

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
    pub protocol_owner: Account<'info, ProtocolOwner>,

    #[account(
        init,
        space = 8 + PoolAccount::SPACE,
        payer = pool_creator,
        seeds = [
            SURE_PRIMARY_POOL_SEED.as_bytes(),
            smart_contract.key().to_bytes().as_ref(),
        ],
        bump
    )]
    pub pool: Box<Account<'info, PoolAccount>>,

    /// Sure Pools keeps an overview over
    /// existing pools
    #[account(mut)]
    pub pools: Box<Account<'info, SurePools>>,

    // Assume the contract being hacked is a token account
    /// CHECK: This accounts represents the executable contract
    /// that is to be insured.
    #[account(
        constraint = smart_contract.executable == false
    )]
    pub smart_contract: UncheckedAccount<'info>,

    /// Sysvar for Associated Token Account
    pub rent: Sysvar<'info, Rent>,

    /// Provide the system program
    pub system_program: Program<'info, System>,
}

impl<'info> Validate<'info> for CreatePool<'info> {
    fn validate(&self) -> Result<()> {
        assert_keys_eq!(self.pool.smart_contract, self.smart_contract);

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
    pub pool_vault_token_mint: Box<Account<'info, Mint>>,

    // Pool Vault used to hold tokens from token_mint
    #[account(
        init,
        payer = creator,
        seeds = [
            SURE_VAULT_POOL_SEED.as_bytes(),
            pool.key().as_ref(),
            pool_vault_token_mint.key().as_ref(),
        ],
        bump,
        token::mint = pool_vault_token_mint,
        token::authority = pool,
    )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    // Premium Vault holding all future premiums
    #[account(
        init,
        payer = creator,
        seeds = [
            SURE_PREMIUM_POOL_SEED.as_bytes(),
            pool.key().as_ref(),
            pool_vault_token_mint.key().as_ref()
        ],
        bump,
        token::mint = pool_vault_token_mint,
        token::authority = pool
    )]
    pub premium_vault: Box<Account<'info, TokenAccount>>,

    /// Pool Tick Accounts
    /// Keep track of tick accounts that has
    /// liquidity in a pool
    #[account(
        init,
        space = 8 + BitMap::SPACE,
        payer = creator,
        seeds = [
            SURE_BITMAP.as_bytes(),
            pool.key().as_ref(),
            pool_vault_token_mint.key().as_ref()
        ],
        bump,
    )]
    pub pool_liquidity_tick_bitmap: Box<Account<'info, BitMap>>,

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
            self.pool_vault_token_mint.key(),
            mint::USDC,
            "Vaults can only have mint USDC"
        );

        Ok(())
    }
}



#[derive(Accounts)]
pub struct UpdateTickPosition<'info> {
    /// Pays for the update
    pub signer: Signer<'info>,

    /// Tick account
    #[account(mut)]
    pub liquidity_tick_info: AccountLoader<'info, Tick>,

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
    pub liquidity_tick_info: AccountLoader<'info, Tick>,

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
    pub liquidity_tick_info: AccountLoader<'info, Tick>,
}

