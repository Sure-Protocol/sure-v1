import {assert} from 'chai'
import * as anchor from "@project-serum/anchor";
import { createMint,AccountLayout,TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount, ASSOCIATED_TOKEN_PROGRAM_ID, transfer, mintTo, getAccount, createAssociatedTokenAccount, Account, getMint} from "@solana/spl-token"

import { Program } from "@project-serum/anchor";
import { SurePool } from "../target/types/sure_pool";
import {PublicKey,LAMPORTS_PER_SOL,TokenAccountsFilter} from "@solana/web3.js"
import { token } from "@project-serum/anchor/dist/cjs/utils";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
import {u64} from "@solana/buffer-layout-utils"
const {SystemProgram} =anchor.web3;
import JSBI from 'jsbi';


import {Metaplex} from "@metaplex-foundation/js-next"
import { metadata } from "@metaplex/js/lib/programs";

import * as sureUtils from "./utils"

/// =============== Variables ==================

// PDA seeds 
const program = anchor.workspace.SurePool as Program<SurePool>


/// Token for Sure Pool
let token0: PublicKey;
let minterWallet: anchor.web3.Keypair;
let liqudityProviderWallet: anchor.web3.Keypair;
let walletATAPubkey: PublicKey;
let liquidityProviderWalletATA: PublicKey;

let vault0: PublicKey;

const nftMint:anchor.web3.Keypair = new anchor.web3.Keypair();

// PDAs
let protcolToInsure0: anchor.web3.Keypair;


/// ============== TESTS ===========================

describe("Initialize Sure Pool",() => {
    const provider = anchor.Provider.env()
    const {connection,wallet} = anchor.getProvider()
    const metaplex = new Metaplex(connection)
    anchor.setProvider(provider)

   
    it("initialize",async () => {
        minterWallet = anchor.web3.Keypair.generate();
        liqudityProviderWallet = anchor.web3.Keypair.generate();

        // Airdrop 1 SOL into each wallet
        const fromAirdropSig = await connection.requestAirdrop(minterWallet.publicKey,10*LAMPORTS_PER_SOL);
        await connection.confirmTransaction(fromAirdropSig)
        const airdropLP = await connection.requestAirdrop(wallet.publicKey,10*LAMPORTS_PER_SOL);
        await connection.confirmTransaction(airdropLP);
        const lpAirdrop = await connection.requestAirdrop(liqudityProviderWallet.publicKey,10*LAMPORTS_PER_SOL);
        await connection.confirmTransaction(lpAirdrop);
        protcolToInsure0 = anchor.web3.Keypair.generate()
        // Create a random mint for testing
        // TODO: The mint should have the same pubkey as USDC
        token0 = await createMint(
            connection,
            minterWallet,
            minterWallet.publicKey,
            minterWallet.publicKey,
            8,
        )

        // Create associated token accounts for each wallet for the token0 mint
        const minterWalletATA = await createAssociatedTokenAccount(
            connection,
            minterWallet,
            token0,
            minterWallet.publicKey,
        )

        walletATAPubkey = await createAssociatedTokenAccount(
            connection,
            (wallet as NodeWallet).payer,
            token0,
            wallet.publicKey,    
        )

        liquidityProviderWalletATA = await createAssociatedTokenAccount(
            connection,
            liqudityProviderWallet,
            token0,
            liqudityProviderWallet.publicKey,    
        )

    
        // Mint initial supply to mint authority associated wallet account
        await mintTo(
            connection,
            minterWallet,
            token0,
            minterWalletATA,
            minterWallet,
            1_000_000_000_000_000,
        )

        // Transfer tokens to liqudity provider ATA from Minter
        await transfer(
            connection,
            minterWallet,
            minterWalletATA,
            walletATAPubkey,
            minterWallet,
            1_000_000,
        )

        await transfer(
            connection,
            minterWallet,
            minterWalletATA,
            liquidityProviderWalletATA,
            minterWallet,
            1_000_000,
        )
        
        // Validate transfer
        const liquidityProviderToken0ATA = await getAccount(
            connection,
            walletATAPubkey,
        )
        assert.equal(liquidityProviderToken0ATA.owner.toBase58(),wallet.publicKey.toBase58())
        assert.equal(liquidityProviderToken0ATA.amount,1_000_000);
    })
    
    it("create protocol owner ", async () => {
        let [protocolOwnerPDA,_] = await sureUtils.getProtocolOwner();
        await program.methods.initializeProtocol().accounts({
                owner: provider.wallet.publicKey,
                protocolOwner: protocolOwnerPDA,
                systemProgram: SystemProgram.programId,
        }).rpc()
    })

    it("create Sure pool manager",async () => {
        const [managerPDA,_] = await PublicKey.findProgramAddress(
            [
                anchor.utils.bytes.utf8.encode("sure-pool-manager")
            ],
            program.programId
        )
    
        // Create Pool Manager PDA 
        await program.rpc.initializePoolManager({
            accounts:{
                manager: managerPDA,
                initialManager: provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
            }
        })

        const onChainManager = await program.account.poolManager.fetch(managerPDA)
        assert.equal(onChainManager.owner.toBase58(),provider.wallet.publicKey.toBase58())
    }),
    it("create sure pool",async ()=> {
         const insuranceFee = 0
         const tick_spacing= 10 // tick size in basispoints
         const name = "my awesome sure pool"
 

        // Generate PDA for Sure Pool
        const poolPDA = await sureUtils.getPoolPDA(protcolToInsure0.publicKey,program);


        // Generate PDA for token vault
        vault0 = await sureUtils.getLiquidityVaultPDA(poolPDA,token0)

        let [protocolOwnerPDA,_] = await sureUtils.getProtocolOwner();
       

        // Create Poool
        try{
            await program.methods.createPool(insuranceFee,tick_spacing,name).accounts(
                {
                    poolCreator:wallet.publicKey,
                    protocolOwner:protocolOwnerPDA,
                    pool:poolPDA,
                    insuredTokenAccount: protcolToInsure0.publicKey,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    systemProgram: SystemProgram.programId,
                }
            ).rpc()
        } catch (err) {
            throw new Error("Could not create pool. Cause: "+err)
        }

        const newPool = await program.account.poolAccount.fetch(poolPDA)
        assert.equal(newPool.tickSpacing,tick_spacing)
        assert.isAbove(newPool.bump,0)
    }),
    it("create pool vaults -> For a given mint the isolated ",async () => {
        // Smart contract that sure should insure. 

        // Generate PDA for Sure Pool
        const pool= await sureUtils.getPoolPDA(protcolToInsure0.publicKey,program);
        const bitmap = await sureUtils.getBitmapPDA(pool,token0,program)
        const liquidityVault = await sureUtils.getLiquidityVaultPDA(pool,token0);
        const premiumVault = await sureUtils.getPremiumVaultPDA(pool,token0)

        try {
            await program.methods.createPoolVaults().accounts({
                creator: wallet.publicKey,
                pool: pool,
                tokenMint: token0,
                liquidityVault: liquidityVault,
                premiumVault: premiumVault,
                bitmap: bitmap,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            }).rpc()
        }catch(err) {
            throw new Error("could not create Pool vaults. cause: "+err)
        }

        const bitmapAccount = await program.account.bitMap.fetch(bitmap);
        assert.equal(bitmapAccount.spacing,10)
    }),
    it("create tick account for pool",async () => {
        const tick = 440;
        const tickBN = new anchor.BN(tick)
        const poolPDA = await sureUtils.getPoolPDA(protcolToInsure0.publicKey,program);
        const tickPDA = await sureUtils.getTickAccountPDA(poolPDA,token0,tick);
        try{
            await program.methods.initializeTick(poolPDA,token0,tickBN).accounts({
                creator:wallet.publicKey,
                tickAccount: tickPDA,
                systemProgram: SystemProgram.programId,

        }).rpc()
        }catch (err) {
            throw new Error("Could not initialize tick. Cause: " + err)
        }

        const createdTickAccount = await program.account.tick.fetch(tickPDA);
        assert.equal(createdTickAccount.active,true);
        assert.equal(createdTickAccount.liquidity.toString(),"0");
        assert.equal(createdTickAccount.usedLiquidity.toString(),"0");
        assert.equal(createdTickAccount.tick.toString(),tick.toString())
        assert.equal(createdTickAccount.lastLiquidityPositionIdx,0);
    }),
    it("deposit liquidity into pool at a given tick",async () => {
        let amount = 15; // amount to draw from account
        let tick = 210; // 300bp tick


        // TODO: Deposit some more liquidity from other LPs
    
        try{
            await sureUtils.depositLiquidity(
            connection,
            amount,
            tick,
            wallet.publicKey,
            walletATAPubkey,
            protcolToInsure0.publicKey,
            token0
            )
        } catch(err) {
            throw new Error("Deposit liquidity error. Cause:" + err)
        }

    

        const poolPDA = await sureUtils.getPoolPDA(protcolToInsure0.publicKey,program);
        const vaultPDA = await sureUtils.getLiquidityVaultPDA(poolPDA,token0);
        const tickPosition = await sureUtils.getCurrentTickPosition(poolPDA,token0,tick);
        const tickAccountPDA = await sureUtils.getTickAccountPDA(poolPDA,token0,tick);
        const tickAccount = await program.account.tick.fetch(tickAccountPDA)   

        const nftAccountPDA = await sureUtils.getLPTokenAccountPDA(
            poolPDA,
            vaultPDA,
            new anchor.BN(tick),
            new anchor.BN(tickPosition)
        )
        let nftAccount = await getAccount(
            connection,
            nftAccountPDA,
        )
        assert.equal(nftAccount.amount,1);
        /// Get liquidity position
        const liquidityPositionPDA = await sureUtils.getLiquidityPositionPDA(nftAccountPDA);
        let liquidityPosition = await program.account.liquidityPosition.fetch(liquidityPositionPDA)
        assert.equal(liquidityPosition.nftAccount.toBase58(),nftAccountPDA.toBase58(),"nft account not equal to expected address")
        
    }),
    it("redeem liquidity based on NFT",async () => {
        //  Allow user to provide only the NFT to get the 
        // liquidity position and redeem it.
        const sureNfts = await sureUtils.getSureNfts(connection,wallet.publicKey)
        /// Select one NFT to redeem 
        const reedemableNFT = sureNfts[0];
        
        // Redeem liquidity
        try {
            await sureUtils.redeemLiquidity(
                wallet.publicKey,
                walletATAPubkey,
                reedemableNFT.pubkey,
                protcolToInsure0.publicKey,
            )
        }catch(err){
            throw new Error(err)
        }
       
    })
    it("buy insurance from smart contract pool",async () => {

        /// Variables
        const amountToBuy = 15000
        const newLiquidity = 14000
        const tick = 120

       

        // deposit liquidity 
        try{
            await sureUtils.depositLiquidity(
                connection,
            newLiquidity,
            tick,
            wallet.publicKey,
            walletATAPubkey,
            protcolToInsure0.publicKey,
            token0
            )
        } catch(err) {
            throw new Error("deposit liquidity error. Cause:" + err)
        }

        try{
            await sureUtils.depositLiquidity(
                connection,
            10000,
            150,
            wallet.publicKey,
            walletATAPubkey,
            protcolToInsure0.publicKey,
            token0
            )
        } catch(err) {
            throw new Error("deposit liquidity error. Cause:" + err)
        }

       

        // Find pool to target
        const poolPDA = await sureUtils.getPoolPDA(
            protcolToInsure0.publicKey,
            program
        )

        // Calculate cost of insurance 
        const [potentialAmountCovered,price] = await sureUtils.estimateYearlyPremium(amountToBuy,token0,poolPDA,wallet.publicKey)
        console.log("potentialAmountCovered: ",potentialAmountCovered.toString(), " , price: ",price.toString())

        await sureUtils.buyInsurance(connection,amountToBuy,token0,poolPDA,wallet.publicKey);
    })
})
