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

/// =============== Variables ==================

// PDA seeds 
const program = anchor.workspace.SurePool as Program<SurePool>
const POOL_SEED =anchor.utils.bytes.utf8.encode("sure-insurance-pool")
const TOKEN_VAULT_SEED = anchor.utils.bytes.utf8.encode("sure-ata")
const SURE_BITMAP = anchor.utils.bytes.utf8.encode("sure-bitmap")
const SURE_LIQUIDITY_POSITION = anchor.utils.bytes.utf8.encode("sure-lp");
const SURE_TICK_SEED = anchor.utils.bytes.utf8.encode("sure-tick")
const SURE_MINT_SEED = anchor.utils.bytes.utf8.encode("sure-nft");
const SURE_TOKEN_ACCOUNT_SEED = anchor.utils.bytes.utf8.encode("sure-token-account");

/// Token for Sure Pool
let token0: PublicKey;
let minterWallet: anchor.web3.Keypair;
let liquidityProviderWalletATAPubKey: PublicKey;

let vault0: PublicKey;

const nftMint:anchor.web3.Keypair = new anchor.web3.Keypair();

// PDAs
let smartContractToInsure0: anchor.web3.Keypair;

/// ================ Methods ====================
const getPoolPDA = async (smartContractToInsure: PublicKey,program: anchor.Program<SurePool>): Promise<[pda: anchor.web3.PublicKey,bump:number]> => {
    const POOL_SEED =anchor.utils.bytes.utf8.encode("sure-insurance-pool")
    return await PublicKey.findProgramAddress(
        [
            POOL_SEED,
            token0.toBytes(),
            smartContractToInsure.toBytes()
        ],
        program.programId
    )
}

const getBitmapPDA = async (poolPDA: PublicKey,token_mint: PublicKey,program: anchor.Program<SurePool>): Promise<[pda: anchor.web3.PublicKey,bump:number]> => {
    return await PublicKey.findProgramAddress(
        [
            SURE_BITMAP,
            poolPDA.toBytes(),
            token_mint.toBytes(),
        ],
        program.programId,
    )
    
}

/// Check if tick account exists for the pool, 
/// if not, create the account. 

const createTickAccount = async (poolPDA: PublicKey,tokenMint: PublicKey,tickBpn: number,creator: PublicKey): Promise<PublicKey> => {
    let tickBp = new anchor.BN(tickBpn)
    const [tickAccountPDA,tickAccountBump] = await PublicKey.findProgramAddress(
        [
            SURE_TICK_SEED,
            poolPDA.toBytes(),
            tokenMint.toBytes(),
            tickBp.toArrayLike(Buffer,"le",8)
        ],
        program.programId,
    )

   try{
    await program.rpc.initializeTick(poolPDA,tokenMint,tickBp, {
        accounts: {
            creator:creator,
            tickAccount: tickAccountPDA,
            systemProgram: SystemProgram.programId,
        },
    })
   } catch(e){
       throw new Error(e)
   }

   return tickAccountPDA
}

const getOrCreateTickAccount = async (owner: PublicKey,poolPDA: PublicKey, tokenMint: PublicKey, tickBp: number): Promise<anchor.web3.PublicKey> => {
    const [tickAccountPDA,tickAccountBump] = await PublicKey.findProgramAddress(
        [
            SURE_TICK_SEED,
            poolPDA.toBytes(),
            tokenMint.toBytes(),
            anchor.utils.bytes.utf8.encode(tickBp.toString()),
        ],
        program.programId,
    )
    let account;
    try {
        account = await program.account.tick.fetch(tickAccountPDA)
    } catch (e){
        console.log("erere")
        // create account
        try {
            await createTickAccount(poolPDA,tokenMint,tickBp,owner);
        }catch (e){
            throw new Error("could not create tick account. cause: "+e)
        }
       account = await program.account.tick.fetch(tickAccountPDA)
    }   
    return account
}

const getProtocolOwner = async (): Promise<[PublicKey,number]> => {
   return await PublicKey.findProgramAddress(
        [],
        program.programId,
    )
}

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
        let [protocolOwnerPDA,_] = await getProtocolOwner();
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
        const [poolPDA,poolBump] = await getPoolPDA(smartContractToInsure0.publicKey,program);


        // Generate PDA for token vault
        const [vaultPDA,vaultBump] = await PublicKey.findProgramAddress(
            [
                TOKEN_VAULT_SEED,
                poolPDA.toBytes(),
                token0.toBytes(),
            ],
            program.programId
        )
        vault0 = vaultPDA;

        const [bitmapPDA,bitmapBum] = await getBitmapPDA(poolPDA,token0,program)
        let [protocolOwnerPDA,_] = await getProtocolOwner();
       

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
    it("deposit liquidity into pool at a given tick",async () => {
        let premium_rate = 300; // basis points
        let amount = 1_000_000; // amount to draw from account
        let tick = 300; // 300bp tick


        const [poolPDA,poolBump] = await getPoolPDA(smartContractToInsure0.publicKey,program);

        let tickBp = new anchor.BN(tick)
       
        const [bitmapPDA,bitmapBum] = await getBitmapPDA(poolPDA,token0,program)
        let [protocolOwnerPDA,_] = await getProtocolOwner();
        const [nftAccountPDA,nftAccountBump] = await PublicKey.findProgramAddress(
                [
                    SURE_TOKEN_ACCOUNT_SEED
                ],
                program.programId
        )
        const [nftMintPDA,nftMintBump] = await PublicKey.findProgramAddress(
            [
                SURE_MINT_SEED
            ],
            program.programId
        )

        const [liquidityPositionPDA,liquidityPositionBump] = await PublicKey.findProgramAddress(
            [
                SURE_LIQUIDITY_POSITION,
                poolPDA.toBytes(),
                vault0.toBytes(),
                tickBp.toArrayLike(Buffer,"le",8),
                nftMintPDA.toBytes(),
            ],
            program.programId,
        )

        const tick_account = await createTickAccount(poolPDA,token0,tick,provider.wallet.publicKey)
        try{
        await program.rpc.depositLiquidity(tickBp,new anchor.BN(amount),{
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
    })
})
