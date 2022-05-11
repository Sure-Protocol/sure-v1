import { SurePool } from "../../target/types/sure_pool";
import {PublicKey} from "@solana/web3.js"
import * as anchor from "@project-serum/anchor"
import { Program } from "@project-serum/anchor";
import { token } from "@project-serum/anchor/dist/cjs/utils";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";
const {SystemProgram} =anchor.web3;

const program = anchor.workspace.SurePool as Program<SurePool>
export const POOL_SEED =anchor.utils.bytes.utf8.encode("sure-insurance-pool")
export const TOKEN_VAULT_SEED = anchor.utils.bytes.utf8.encode("sure-ata")
export const SURE_BITMAP = anchor.utils.bytes.utf8.encode("sure-bitmap")
export const SURE_LIQUIDITY_POSITION = anchor.utils.bytes.utf8.encode("sure-lp");
export const SURE_TICK_SEED = anchor.utils.bytes.utf8.encode("sure-tick")
export const SURE_NFT_MINT_SEED = anchor.utils.bytes.utf8.encode("sure-nft");
export const SURE_TOKEN_ACCOUNT_SEED = anchor.utils.bytes.utf8.encode("sure-token-account");


export const getPoolPDA = async (smartContractToInsure: PublicKey,token_mint:PublicKey,program: anchor.Program<SurePool>): Promise<anchor.web3.PublicKey> => {
    const POOL_SEED =anchor.utils.bytes.utf8.encode("sure-insurance-pool")
    const [poolPDA,poolBump] = await PublicKey.findProgramAddress(
        [
            POOL_SEED,
            token_mint.toBytes(),
            smartContractToInsure.toBytes()
        ],
        program.programId
    )
    return poolPDA
}

export const getBitmapPDA = async (poolPDA: PublicKey,token_mint: PublicKey,program: anchor.Program<SurePool>): Promise<[pda: anchor.web3.PublicKey,bump:number]> => {
    return await PublicKey.findProgramAddress(
        [
            SURE_BITMAP,
            poolPDA.toBytes(),
            token_mint.toBytes(),
        ],
        program.programId,
    )
    
}

export const getTickPDA = async (poolPDA: PublicKey,tokenMint: PublicKey,tick:number): Promise<PublicKey> =>{
    let tickBN = new anchor.BN(tick)
    const [tickAccountPDA,tickAccountBump] = await PublicKey.findProgramAddress(
        [
            SURE_TICK_SEED,
            poolPDA.toBytes(),
            tokenMint.toBytes(),
            tickBN.toArrayLike(Buffer,"le",8)
        ],
        program.programId,
    )
    return tickAccountPDA
}

export const getTickPosition = async (poolPDA: PublicKey,tokenMint: PublicKey,tick:number): Promise<number> => {
    const tickPDA = await getTickPDA(poolPDA,tokenMint,tick);
    try {
        const tickAccount = await program.account.tick.fetch(tickPDA);
        return tickAccount.lastLiquidityPositionIdx
    }catch(e){
        throw new Error("Tick account does not exist. Cause: "+e)
    }
}

/// Check if tick account exists for the pool, 
/// if not, create the account. 
export const createTickAccount = async (poolPDA: PublicKey,tokenMint: PublicKey,tick: number,creator: PublicKey): Promise<PublicKey> => {
    
    const tickAccountPDA = await getTickPDA(poolPDA,tokenMint,tick);

    try{
        let tickBN = new anchor.BN(tick)
        await program.rpc.initializeTick(poolPDA,tokenMint,tickBN, {
            accounts: {
                creator:creator,
                tickAccount: tickAccountPDA,
                systemProgram: SystemProgram.programId,
            },
        })
    } catch(e){
        console.log("errors: ",e)
        throw new Error("Could not create tick account: " +e)
    }

   return tickAccountPDA
}

export const getOrCreateTickAccount = async (poolPDA: PublicKey,tokenMint: PublicKey,tick: number,owner: PublicKey): Promise<anchor.web3.PublicKey> => {

    const tickAccountPDA = await getTickPDA(poolPDA,tokenMint,tick);
    
    try {
        await program.account.tick.fetch(tickAccountPDA)
    } catch (e){
        console.log("sure.getTickAccount.error Could not fetch tick account. Cause: "+e)
        // create account
        try {
            await createTickAccount(poolPDA,tokenMint,tick,owner);
        }catch (e){
            throw new Error("sure.createTickAccount.error. could not create tick account. cause: "+e)
        }
    }   
    return tickAccountPDA
}

export const getProtocolOwner = async (): Promise<[PublicKey,number]> => {
   return await PublicKey.findProgramAddress(
        [],
        program.programId,
    )
}

export const getVaultPDA = async (poolPDA: PublicKey,tokenMint:PublicKey):Promise<PublicKey> => {
    const [vaultPDA,vaultBump] = await PublicKey.findProgramAddress(
        [
            TOKEN_VAULT_SEED,
            poolPDA.toBytes(),
            tokenMint.toBytes(),
        ],
        program.programId
    )
    return vaultPDA
}


export const getLPMintPDA = async (poolPDA:PublicKey,vaultPDA: PublicKey,tickBN: anchor.BN,nextTickPositionBN: anchor.BN): Promise<PublicKey> => {

    const [nftAccountPDA,nftAccountBump] = await PublicKey.findProgramAddress(
        [
            SURE_NFT_MINT_SEED,
            poolPDA.toBytes(),
            vaultPDA.toBytes(),
            tickBN.toArrayLike(Buffer,"le",8),
            nextTickPositionBN.toArrayLike(Buffer,"le",8)
        ],
        program.programId
    )
    return nftAccountPDA
}

export const getLPTokenAccountPDA = async (poolPDA:PublicKey,vaultPDA: PublicKey,tickBN: anchor.BN,nextTickPositionBN: anchor.BN): Promise<PublicKey> => {

    const [nftAccountPDA,nftAccountBump] = await PublicKey.findProgramAddress(
        [
            SURE_TOKEN_ACCOUNT_SEED,
            poolPDA.toBytes(),
            vaultPDA.toBytes(),
            tickBN.toArrayLike(Buffer,"le",8),
            nextTickPositionBN.toArrayLike(Buffer,"le",8)
        ],
        program.programId
    )
    return nftAccountPDA
}

export const getLiquidityPositionPDA = async (nftAccountPDA: PublicKey): Promise<PublicKey> => {
    const [liquidityPositionPDA,liquidityPositionBump] = await PublicKey.findProgramAddress(
        [
            SURE_LIQUIDITY_POSITION,
            nftAccountPDA.toBytes(),
        ],
        program.programId,
    )
    return liquidityPositionPDA
}
/**
     * Deposit liquidity into a Sure pool
     *
     * @param liquidityAmount Amount of liquidity to be transferred 
     * @param tick Tick in basis points to supply liquidity to
     * @param liquidityProvider The signer of the transaction
     * @param liquidityProviderATA Associated Token Account for the tokens to be supplied to the pool
     * @param protocolToInsure The Public Key of the program to insure
     * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
     * @return Nothing
     */
export const depositLiquidity = async (
    liquidityAmount: number,
    tick:number,
    liquidityProvider: PublicKey,
    liquidityProviderATA:PublicKey,
    protocolToInsure: PublicKey,
    tokenMint:PublicKey
    ) => {
    // Liquidity Pool PDA
    const poolPDA = await getPoolPDA(protocolToInsure,tokenMint,program);

    // Protocol Owner 
    let [protocolOwnerPDA,_] = await getProtocolOwner();
    // Liquidity Pool Vault
    const vaultPDA = await getVaultPDA(poolPDA,tokenMint);

    // Get tick account
    //const tickAccount = await getOrCreateTickAccount(poolPDA,tokenMint,tick,liquidityProvider)
    
    const tickAccount = await  createTickAccount(poolPDA,tokenMint,tick,liquidityProvider);
    //  Generate tick

    const tickBN = new anchor.BN(tick)
    const tickPosition = await getTickPosition(poolPDA,tokenMint,tick);
    const nextTickPositionBN = new anchor.BN(tickPosition+1)

    // Generate nft accounts
    const nftMint = await getLPMintPDA(poolPDA,vaultPDA,tickBN,nextTickPositionBN);
    const nftAccount = await getLPTokenAccountPDA(poolPDA,vaultPDA,tickBN,nextTickPositionBN)

    let liquidityPositionPDA = await getLiquidityPositionPDA(nftAccount);

    // Get bitmap 
    const [bitmapPDA,bitmapBum] = await getBitmapPDA(poolPDA,tokenMint,program)

   

    /// Deposit liquidity Instruction 
    try {
        await program.rpc.depositLiquidity(tickBN,nextTickPositionBN,(new anchor.BN(liquidityAmount)),{
            accounts:{
                liquidityProvider: liquidityProvider,
                protocolOwner: protocolOwnerPDA,
                liquidityProviderAccount: liquidityProviderATA,
                pool: poolPDA,
                tokenVault: vaultPDA,
                nftMint: nftMint,
                liquidityPosition: liquidityPositionPDA,
                nftAccount: nftAccount,
                bitmap: bitmapPDA,
                tickAccount: tickAccount,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            }
        })
    } catch(e){
        throw new Error("sure.error! Could not deposit liqudity. Cause: "+e)
    }
}