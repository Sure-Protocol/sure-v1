import {assert} from "chai"
import * as chai from 'chai'
import * as anchor from "@project-serum/anchor";
import {mintTo, createMint,TOKEN_PROGRAM_ID} from "@solana/spl-token"

import { Program } from "@project-serum/anchor";
import { SurePool } from "../target/types/sure_pool";
import {PublicKey,LAMPORTS_PER_SOL} from "@solana/web3.js"
const {SystemProgram} =anchor.web3;

describe("Initialize Sure Pool",() => {
    const provider = anchor.Provider.env()
    const {connection,wallet} = anchor.getProvider()
    anchor.setProvider(provider)

    const program = anchor.workspace.SurePool as Program<SurePool>
    const POOL_SEED =anchor.utils.bytes.utf8.encode("sure-insurance-pool")
    const TOKEN_VAULT_SEED = anchor.utils.bytes.utf8.encode("sure-ata")
    const SURE_BITMAP = anchor.utils.bytes.utf8.encode("sure-bitmap")
   

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
         // Create Minter wallet 
        const minterWallet = anchor.web3.Keypair.generate();

        // Set initial variables
        const fromAirdropSig = await connection.requestAirdrop(minterWallet.publicKey,LAMPORTS_PER_SOL);
        await connection.confirmTransaction(fromAirdropSig)
        const insuranceFee = 0
        const tick_spacing= 1 // tick size in basispoints
        const name = "my awesome sure pool"

        // Create a random mint for testing
        // TODO: The mint should have the same pubkey as USDC
        const tokenMint = await createMint(
            connection,
            minterWallet,
            minterWallet.publicKey,
            minterWallet.publicKey,
            8,
        )
        
        // Smart contract that sure should insure. 
        const smartContractToInsure = anchor.web3.Keypair.generate()

        // Generate PDA for Sure Pool
        const [poolPDA,poolBump] = await PublicKey.findProgramAddress(
            [
                POOL_SEED,
                tokenMint.toBytes(),
                smartContractToInsure.publicKey.toBytes()
            ],
            program.programId
        )

        // Generate PDA for token vault
        const [vaultPDA,vaultBump] = await PublicKey.findProgramAddress(
            [
                TOKEN_VAULT_SEED,
                poolPDA.toBytes(),
                tokenMint.toBytes(),
            ],
            program.programId
        )

        const [bitmapPDA,bitmapBum] = await PublicKey.findProgramAddress(
            [
                SURE_BITMAP,
                poolPDA.toBytes(),
                tokenMint.toBytes(),
            ],
            program.programId,
        )

        // Create Poool
        await program.rpc.createPool(insuranceFee,tick_spacing,name,{
            accounts:{
                pool:poolPDA,
                protocolOwner: provider.wallet.publicKey,
                vault: vaultPDA,
                poolCreator:provider.wallet.publicKey,
                token: tokenMint,
                insuredTokenAccount: smartContractToInsure.publicKey,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                bitmap:bitmapPDA,
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
            

        await program.rpc.depositLiquidity(premium_rate,amount,{
            accounts: {
                protocolOwner: provider.wallet.publicKey,

            }
        })
    })
})
