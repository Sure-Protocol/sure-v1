import {assert} from "chai"
import * as chai from 'chai'
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
        await program.rpc.initializeProtocol({
            accounts:{
                owner: provider.wallet.publicKey,
                protocolOwner: protocolOwnerPDA,
                systemProgram: SystemProgram.programId,
            }
        })
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
 

        
        // Smart contract that sure should insure. 
        protcolToInsure0 = anchor.web3.Keypair.generate()

        // Generate PDA for Sure Pool
        const poolPDA = await sureUtils.getPoolPDA(protcolToInsure0.publicKey,token0,program);


        // Generate PDA for token vault
        vault0 = await sureUtils.getVaultPDA(poolPDA,token0)

        const bitmapPDA = await sureUtils.getBitmapPDA(poolPDA,token0,program)
        let [protocolOwnerPDA,_] = await sureUtils.getProtocolOwner();
       

        // Create Poool
        await program.rpc.createPool(insuranceFee,tick_spacing,name,{
            accounts:{
                poolCreator:provider.wallet.publicKey,
                protocolOwner:protocolOwnerPDA,
                pool:poolPDA,
                insuredTokenAccount: protcolToInsure0.publicKey,
                vault: vault0,
                token: token0,
                bitmap:bitmapPDA,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            }
        })

        const newPool = await program.account.poolAccount.fetch(poolPDA)
        assert.equal(newPool.tickSpacing,tick_spacing)
    }),
    it("create tick account for pool",async () => {
        const tick = 340;
        const tickBN = new anchor.BN(tick)
        const poolPDA = await sureUtils.getPoolPDA(protcolToInsure0.publicKey,token0,program);
        const tickPDA = await sureUtils.getTickAccountPDA(poolPDA,token0,tick);
        await program.rpc.initializeTick(poolPDA,token0,tickBN,{
            accounts: {
                creator:wallet.publicKey,
                tickAccount: tickPDA,
                systemProgram: SystemProgram.programId,
            }
        })

        const createdTickAccount = await program.account.tick.fetch(tickPDA);
        assert.equal(createdTickAccount.active,true);
        assert.equal(createdTickAccount.liquidity,0)
        assert.equal(createdTickAccount.lastLiquidityPositionIdx,0);
    }),
    it("deposit liquidity into pool at a given tick",async () => {
        let premium_rate = 300; // basis points
        let amount = 1; // amount to draw from account
        let tick = 220; // 300bp tick


    // TODO: Deposit some more liquidity from other LPs
    
    try{
        await sureUtils.depositLiquidity(
        amount,
        tick,
        wallet.publicKey,
        walletATAPubkey,
        protcolToInsure0.publicKey,
        token0
        )
    } catch(err) {
        console.log("deposit liquidity error. Cause: ",err)
    }

    const poolPDA = await sureUtils.getPoolPDA(protcolToInsure0.publicKey,token0,program);
    const vaultPDA = await sureUtils.getVaultPDA(poolPDA,token0);
    const tickPosition = await sureUtils.getCurrentTickPosition(poolPDA,token0,tick);
    const tickAccountPDA = await sureUtils.getTickAccountPDA(poolPDA,token0,tick);
    const tickAccount = await program.account.tick.fetch(tickAccountPDA)
    ///! here   
    metaplex
   

    const nftAccountPDA = await sureUtils.getLPTokenAccountPDA(
        poolPDA,
        vaultPDA,
        new anchor.BN(tick),
        new anchor.BN(tickPosition+1)
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
        // const sureNfts = await sureUtils.getSureNfts(connection,wallet.publicKey)
        
        // /// Select one NFT to redeem 
        // const reedemableNFT = sureNfts[0];
        
        // // Redeem liquidity
        // await sureUtils.redeemLiquidity(
        //     wallet.publicKey,
        //     walletATAPubkey,
        //     reedemableNFT.pubkey
        // )
    })
    it("buy insurance from smart contract pool",async () => {

        // /// Variables
        // const amount = 3000

        // // Find pool to target
        // const poolPDA = await sureUtils.getPoolPDA(
        //     protcolToInsure0.publicKey,
        //     token0,
        //     program
        // )
        
        // let poolAccount
        // try {
        //     poolAccount = await program.account.poolAccount.fetch(poolPDA);
        // }catch(e){
        //     console.log("could not get pool account: cause: ",e )
        //     throw new Error("pool account not found. cause: " + e)
        // }
       
        // const tickSpacing = poolAccount.tickSpacing

        // // 
        // const firstBit = await sureUtils.getLowestBit(poolPDA,token0);
        // const firstTick = 0 + tickSpacing*firstBit;
        // console.log("first tick to buy from: ",firstTick);

        // // get tick account
    
        // const tickAccountPDA = await sureUtils.getTickAccountPDA(poolPDA,token0,firstTick);
        // let tickAccount;
        // try {
        //     tickAccount = await program.account.tick.fetch(tickAccountPDA) 
        // }catch(e){
        //     console.log("could not get tick account: cause: ",e )
        //     throw new Error("Tick account not found. cause: " + e)
        // }


        // console.log("tickAccount.liquidity: ",tickAccount.liquidity.toString())
        // const availableLiquidity = JSBI.BigInt(tickAccount.liquidity)
        // console.log("available liq: ",String(availableLiquidity))
        // console.log("bn to string",)
        // console.log("tick: ",String(JSBI.BigInt(tickAccount.usedLiquidity)))
        // const ten8 = JSBI.BigInt(Math.pow(10,8));
        // const liquidityDecimals = JSBI.divide(availableLiquidity,ten8);
        // console.log("available liquidity: ",JSBI.toNumber(liquidityDecimals))

    })
})
