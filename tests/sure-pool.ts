import {assert} from "chai"
import * as chai from 'chai'
import * as anchor from "@project-serum/anchor";
import { createMint,TOKEN_PROGRAM_ID, getOrCreateAssociatedTokenAccount, ASSOCIATED_TOKEN_PROGRAM_ID, transfer, mintTo, getAccount, createAssociatedTokenAccount, Account} from "@solana/spl-token"

import { Program } from "@project-serum/anchor";
import { SurePool } from "../target/types/sure_pool";
import {PublicKey,LAMPORTS_PER_SOL} from "@solana/web3.js"
import { token } from "@project-serum/anchor/dist/cjs/utils";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";
const {SystemProgram} =anchor.web3;

import * as sureUtils from "./utils"
/// =============== Variables ==================

// PDA seeds 
const program = anchor.workspace.SurePool as Program<SurePool>


/// Token for Sure Pool
let token0: PublicKey;
let minterWallet: anchor.web3.Keypair;
let liquidityProviderWalletATAPubKey: PublicKey;

let vault0: PublicKey;

const nftMint:anchor.web3.Keypair = new anchor.web3.Keypair();

// PDAs
let smartContractToInsure0: anchor.web3.Keypair;

/// ================ Methods ====================


/// ============== TESTS ===========================

describe("Initialize Sure Pool",() => {
    const provider = anchor.Provider.env()
    const {connection,wallet} = anchor.getProvider()
    anchor.setProvider(provider)

   
    it("initialize",async () => {
        minterWallet = anchor.web3.Keypair.generate();

        // Airdrop 1 SOL into each wallet
        const fromAirdropSig = await connection.requestAirdrop(minterWallet.publicKey,LAMPORTS_PER_SOL);
        await connection.confirmTransaction(fromAirdropSig)
        const airdropLP = await connection.requestAirdrop(wallet.publicKey,LAMPORTS_PER_SOL);
        await connection.confirmTransaction(airdropLP);
        
        
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

        liquidityProviderWalletATAPubKey = await createAssociatedTokenAccount(
            connection,
            (wallet as NodeWallet).payer,
            token0,
            wallet.publicKey,    
        )

    
        // Mint initial supply to mint authority associated wallet account
        await mintTo(
            connection,
            minterWallet,
            token0,
            minterWalletATA,
            minterWallet,
            1_000_000_000_000,
        )

        // Transfer tokens to liqudity provider ATA from Minter
        await transfer(
            connection,
            minterWallet,
            minterWalletATA,
            liquidityProviderWalletATAPubKey,
            minterWallet,
            1_000_000,
        )
        
        // Validate transfer
        const liquidityProviderToken0ATA = await getAccount(
            connection,
            liquidityProviderWalletATAPubKey,
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
         const tick_spacing= 1 // tick size in basispoints
         const name = "my awesome sure pool"
 

        
        // Smart contract that sure should insure. 
        smartContractToInsure0 = anchor.web3.Keypair.generate()

        // Generate PDA for Sure Pool
        const [poolPDA,poolBump] = await sureUtils.getPoolPDA(smartContractToInsure0.publicKey,token0,program);


        // Generate PDA for token vault
        const [vaultPDA,vaultBump] = await PublicKey.findProgramAddress(
            [
                sureUtils.TOKEN_VAULT_SEED,
                poolPDA.toBytes(),
                token0.toBytes(),
            ],
            program.programId
        )
        vault0 = vaultPDA;

        const [bitmapPDA,bitmapBum] = await sureUtils.getBitmapPDA(poolPDA,token0,program)
        let [protocolOwnerPDA,_] = await sureUtils.getProtocolOwner();
       

        // Create Poool
        await program.rpc.createPool(insuranceFee,tick_spacing,name,{
            accounts:{
                poolCreator:provider.wallet.publicKey,
                protocolOwner:protocolOwnerPDA,
                pool:poolPDA,
                insuredTokenAccount: smartContractToInsure0.publicKey,
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
        const [poolPDA,poolBump] = await sureUtils.getPoolPDA(smartContractToInsure0.publicKey,token0,program);
        const tickPDA = await sureUtils.getTickPDA(poolPDA,token0,tick);
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
        let amount = 1_000_000; // amount to draw from account
        let tick = 300; // 300bp tick


        let tickBp = new anchor.BN(tick)
        // Get next tick position
        const [poolPDA,poolBump] = await sureUtils.getPoolPDA(smartContractToInsure0.publicKey,token0,program);
        const tick_account = await sureUtils.createTickAccount(poolPDA,token0,tick,provider.wallet.publicKey)
        
        const nextTickPos = await sureUtils.getNextTickPosition(poolPDA,token0,tick)+1;
        console.log("tick pos: ",nextTickPos)

        const tickPos = new anchor.BN(nextTickPos)
       
        const [bitmapPDA,bitmapBum] = await sureUtils.getBitmapPDA(poolPDA,token0,program)
        let [protocolOwnerPDA,_] = await sureUtils.getProtocolOwner();
       
        const [nftAccountPDA,nftAccountBump] = await PublicKey.findProgramAddress(
                [
                    sureUtils.SURE_TOKEN_ACCOUNT_SEED,
                    poolPDA.toBytes(),
                    vault0.toBytes(),
                    tickBp.toArrayLike(Buffer,"le",8),
                    tickPos.toArrayLike(Buffer,"le",8)
                ],
                program.programId
        )
     
        const [nftMintPDA,nftMintBump] = await PublicKey.findProgramAddress(
            [
                sureUtils.SURE_NFT_MINT_SEED,
                poolPDA.toBytes(),
                vault0.toBytes(),
                tickBp.toArrayLike(Buffer,"le",8),
                tickPos.toArrayLike(Buffer,"le",8)
            ],
            program.programId
        )

        const [liquidityPositionPDA,liquidityPositionBump] = await PublicKey.findProgramAddress(
            [
                sureUtils.SURE_LIQUIDITY_POSITION,
                nftAccountPDA.toBytes(),
            ],
            program.programId,
        )

        
        try{
            await program.rpc.depositLiquidity(tickBp,tickPos,(new anchor.BN(amount)),{
                accounts:{
                    liquidityProvider: wallet.publicKey,
                    protocolOwner: protocolOwnerPDA,
                    liquidityProviderAccount: liquidityProviderWalletATAPubKey,
                    pool: poolPDA,
                    tokenVault: vault0,
                    nftMint: nftMintPDA,
                    liquidityPosition: liquidityPositionPDA,
                    nftAccount: nftAccountPDA,
                    bitmap: bitmapPDA,
                    tickAccount: tick_account,
                    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                }
            })
        } catch(e){
            console.log("res: ",e)
        } 
    

    let nftAccount = await getAccount(
        connection,
        nftAccountPDA,
    )
    assert.equal(nftAccount.amount,1);
    /// Get liquidity position
    let liquidityPosition = await program.account.liquidityPosition.fetch(liquidityPositionPDA)
    console.log("")
    assert.equal(liquidityPosition.nftAccount.toBase58(),nftAccountPDA.toBase58(),"nft account not equal to expected address")
    })
})
