import { SurePool } from "../../target/types/sure_pool";
import {PublicKey} from "@solana/web3.js"
import * as anchor from "@project-serum/anchor"
import { Program } from "@project-serum/anchor";
import { token } from "@project-serum/anchor/dist/cjs/utils";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {TokenAccount} from "./types"
import {  metaplex } from "@metaplex/js/lib/programs";
import { TokenMetadataProgram, TokenProgram } from "@metaplex-foundation/js-next";
import { metadata } from "@metaplex/js/lib/programs";
import { assert } from "chai";
import { Connection } from "@metaplex/js";

const {SystemProgram} =anchor.web3;
const program = anchor.workspace.SurePool as Program<SurePool>

export const POOL_SEED =anchor.utils.bytes.utf8.encode("sure-insurance-pool")
export const TOKEN_VAULT_SEED = anchor.utils.bytes.utf8.encode("sure-ata")
export const SURE_BITMAP = anchor.utils.bytes.utf8.encode("sure-bitmap")
export const SURE_LIQUIDITY_POSITION = anchor.utils.bytes.utf8.encode("sure-lp");
export const SURE_TICK_SEED = anchor.utils.bytes.utf8.encode("sure-tick")
export const SURE_VAULT_POOL_SEED = anchor.utils.bytes.utf8.encode("sure-liquidity-vault");
export const SURE_PREMIUM_POOL_SEED = anchor.utils.bytes.utf8.encode("sure-premium-vault");
export const SURE_NFT_MINT_SEED = anchor.utils.bytes.utf8.encode("sure-nft");
export const SURE_TOKEN_ACCOUNT_SEED = anchor.utils.bytes.utf8.encode("sure-token-account");
export const SURE_MP_METADATA_SEED = anchor.utils.bytes.utf8.encode("metadata")
export const SURE_INSURANCE_CONTRACT = anchor.utils.bytes.utf8.encode("sure-insurance-contract")
// Export sub libs
export * from "./nft"


export const getLiquidityVaultPDA = async (pool: PublicKey,tokenMint: PublicKey): Promise<PublicKey> => {
    const [liquidityVaultPDA,liquidityVaultBump] = await PublicKey.findProgramAddress(
        [
            SURE_VAULT_POOL_SEED,
            pool.toBytes(),
            tokenMint.toBytes()
        ],
        program.programId
    )
    return liquidityVaultPDA
}

export const getPremiumVaultPDA = async (pool: PublicKey,tokenMint: PublicKey): Promise<PublicKey> => {
    const [premiumVaultPDA,premiumVaultBump] = await PublicKey.findProgramAddress(
        [
            SURE_PREMIUM_POOL_SEED,
            pool.toBytes(),
            tokenMint.toBytes()
        ],
        program.programId
    )
    return premiumVaultPDA
}

export const getPoolPDA = async (smartContractToInsure: PublicKey,program: anchor.Program<SurePool>): Promise<anchor.web3.PublicKey> => {
   
    const [poolPDA,poolBump] = await PublicKey.findProgramAddress(
        [
            POOL_SEED,
            smartContractToInsure.toBytes()
        ],
        program.programId
    )
    return poolPDA
}

export const getBitmapPDA = async (pool: PublicKey,tokenMint: PublicKey,program: anchor.Program<SurePool>): Promise<anchor.web3.PublicKey> => {
    const [bitmapPDA,bitmapBump] =  await PublicKey.findProgramAddress(
        [
            SURE_BITMAP,
            pool.toBytes(),
            tokenMint.toBytes()
        ],
        program.programId,
    )
    return bitmapPDA
    
}

export const getTickAccountPDA = async (poolPDA: PublicKey,tokenMint: PublicKey,tick:number): Promise<PublicKey> =>{
    let tickBN = new anchor.BN(tick)
    const [tickAccountPDA,tickAccountBump] = await PublicKey.findProgramAddress(
        [
            SURE_TICK_SEED,
            poolPDA.toBytes(),
            tokenMint.toBytes(),
            tickBN.toArrayLike(Buffer,"le",2)
        ],
        program.programId,
    )
    return tickAccountPDA
}

/**
     * Current tick position in tick pool
     *
     * @param poolPDA PDA for pool 
     * @param tick Tick in basis points to supply liquidity to
     * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
     * @return Nothing
     */
export const getCurrentTickPosition = async (poolPDA: PublicKey,tokenMint: PublicKey,tick:number): Promise<number> => {
    const tickPDA = await getTickAccountPDA(poolPDA,tokenMint,tick);
    try {
        const tickAccount = await program.account.tick.fetch(tickPDA);
        return tickAccount.lastLiquidityPositionIdx
    }catch(e){
        throw new Error("Tick account does not exist. Cause: "+e)
    }
}

/**
     * Current tick position in tick pool
     * get the lowest tick pool with available liquidity
     *
     * @param poolPDA PDA for pool 
     * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
     * @return lowest bit
     */
export const getLowestBit = async (poolPDA:PublicKey,tokenMint: PublicKey): Promise<number> => {
    const bitmapPDA = await getBitmapPDA(poolPDA,tokenMint,program)
    const bitmap =await program.account.bitMap.fetch(bitmapPDA)

    const u256 = bitmap.word.flatMap((word) => {
        return word.toString(2,64).split("").reverse().join("")
    })[0]
    const firstBit = u256.indexOf("1")
   
    return firstBit
}

export const getTickBasisPoint = (bitPosition: number, tickSpacing: number): number => {
    return 0 + tickSpacing*bitPosition
}

export const getBitFromTick = (tick: number, tickSpacing: number): number => {
    return tick/tickSpacing
}

/**
     * Get the next tick position in tick pool
     *
     * @param poolPDA PDA for pool 
     * @param tokenMint The mint of the token to be supplied to the pool. This could be USDC
     * @return Next tick position
     */
export const getNextBit = async (poolPK: PublicKey,tokenMint: PublicKey,prevBit:number): Promise<number> => {
    const bitmapPDA = await getBitmapPDA(poolPK,tokenMint,program)
    const bitmap =await program.account.bitMap.fetch(bitmapPDA)

    const u256 = bitmap.word.flatMap((word) => {
        return word.toString(2,64).split("").reverse().join("")
    })[0]

    console.log("u256: ",u256)
    return 0
}

export const getNextTick = async (prevTick: number,poolPK: PublicKey, tokenMint: PublicKey,tickSpacing: number ): Promise<number> => {
    //console.log("// Get next tick")
    const prevBit = getBitFromTick(prevTick,tickSpacing);
    //console.log("previous bit: ",prevBit)
    const bitmapPDA = await getBitmapPDA(poolPK,tokenMint,program)
    const bitmap =await program.account.bitMap.fetch(bitmapPDA)

    const u256 = bitmap.word.flatMap((word) => {
        return word.toString(2,64).split("").reverse().join("")
    })[0]

    //console.log("u256: ",u256)
    const remainingBitmap = u256.slice(prevBit+1)
    const subBit = remainingBitmap.indexOf("1")
    if (subBit === -1){
        return -1
    }
    const nextBit = subBit + prevBit+1

    return getTickBasisPoint(nextBit,tickSpacing)

}

/// Check if tick account exists for the pool, 
/// if not, create the account. 
export const createTickAccount = async (poolPDA: PublicKey,tokenMint: PublicKey,tick: number,creator: PublicKey): Promise<PublicKey> => {
    
    const tickAccountPDA = await getTickAccountPDA(poolPDA,tokenMint,tick);

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

    const tickAccountPDA = await getTickAccountPDA(poolPDA,tokenMint,tick);
    
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


export const getLPMintPDA = async (nftAccountPDA: PublicKey): Promise<PublicKey> => {

    const [nftMintPDA,nftMintBump] = await PublicKey.findProgramAddress(
        [
            SURE_NFT_MINT_SEED,
            nftAccountPDA.toBytes(),
        ],
        program.programId
    )
    return nftMintPDA
}

export const getMpMetadataPDA = async (nftMintPDA: PublicKey): Promise<PublicKey> => {
    const [mpMetadataPDA, mpMetadataBump] = await PublicKey.findProgramAddress(
        [
            SURE_MP_METADATA_SEED,
            TokenMetadataProgram.publicKey.toBuffer(),
            nftMintPDA.toBytes()
        ],
        TokenMetadataProgram.publicKey
    )
    return mpMetadataPDA
}

export const getInsuranceContractPDA = async (pool: PublicKey,owner: PublicKey):Promise<PublicKey> => {

    const [insuranceContractPDA,insuranceContractBump] = await PublicKey.findProgramAddress(
       [
        SURE_INSURANCE_CONTRACT,
        owner.toBuffer(),
        pool.toBuffer()
       ],
       program.programId
    )
   
    return insuranceContractPDA
}

export const getLPTokenAccountPDA = async (poolPDA:PublicKey,vaultPDA: PublicKey,tickBN: anchor.BN,nextTickPositionBN: anchor.BN): Promise<PublicKey> => {

    const [nftAccountPDA,nftAccountBump] = await PublicKey.findProgramAddress(
        [
            SURE_TOKEN_ACCOUNT_SEED,
            poolPDA.toBytes(),
            vaultPDA.toBytes(),
            tickBN.toArrayLike(Buffer,"le",2),
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
    connection: anchor.web3.Connection, 
    liquidityAmount: number,
    tick:number,
    liquidityProvider: PublicKey,
    liquidityProviderATA:PublicKey,
    protocolToInsure: PublicKey,
    tokenMint:PublicKey
    ) => {
    // Liquidity Pool PDA
    const poolPDA = await getPoolPDA(protocolToInsure,program);
    try{
        await program.account.poolAccount.fetch(poolPDA)
    }catch(err){
        throw new Error("Pool does not exist. Cause: "+err)
    }

    // Protocol Owner 
    let [protocolOwnerPDA,_] = await getProtocolOwner();
    try{
        await program.account.protocolOwner.fetch(protocolOwnerPDA)
    }catch(err){
        throw new Error("Protocol owner does not exist. Cause: "+err)
    }
    // Liquidity Pool Vault
    const vaultPDA = await getLiquidityVaultPDA(poolPDA,tokenMint);
    try {
        await getAccount(
            connection,
            vaultPDA,
        )
    }catch(err){
        throw new Error("Vault does not exist. Cause: "+err)
    }


    // Get tick account
    const tickAccountPDA = await createTickAccount(poolPDA,tokenMint,tick,liquidityProvider);
    try{
        await program.account.tick.fetch(tickAccountPDA)
    }catch(err){
        throw new Error("Tick account does not exist. Cause: "+err)
    }
    //  Generate tick

    const tickBN = new anchor.BN(tick)
    const tickPosition = await getCurrentTickPosition(poolPDA,tokenMint,tick);
    const nextTickPositionBN = new anchor.BN(tickPosition+1)

    // Generate nft accounts
    const nftAccount = await getLPTokenAccountPDA(poolPDA,vaultPDA,tickBN,nextTickPositionBN)
    const nftMint = await getLPMintPDA(nftAccount);
  
    let liquidityPositionPDA = await getLiquidityPositionPDA(nftAccount);

    // Get bitmap 
    const bitmapPDA = await getBitmapPDA(poolPDA,tokenMint,program)
    try{
        await program.account.bitMap.fetch(bitmapPDA)
    }catch(err){
        throw new Error("Bitmap does not exist. Cause: "+err)
    }

    const mpMetadataAccountPDA = await getMpMetadataPDA(nftMint)

    /// Deposit liquidity Instruction 
    try {
        const amountBN = new anchor.BN(liquidityAmount)
        await program.methods.depositLiquidity(tick,nextTickPositionBN,amountBN).accounts({
            liquidityProvider: liquidityProvider,
            protocolOwner: protocolOwnerPDA,
            liquidityProviderAccount: liquidityProviderATA,
            pool: poolPDA,
            vault: vaultPDA,
            nftMint: nftMint,
            metadataAccount:mpMetadataAccountPDA,
            metadataProgram:TokenMetadataProgram.publicKey,
            liquidityPosition: liquidityPositionPDA,
            nftAccount: nftAccount,
            bitmap: bitmapPDA,
            tickAccount: tickAccountPDA,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        }).rpc()

    } catch(e){
        console.log(e)
        throw new Error("sure.error! Could not deposit liqudity. Cause: "+e)
    }
}

/**
 * Redeem liquidity based on ownership of NFT
 * 
 * @param Wallet: the publickey of the signer and payer
 * @param walletATA: Associated token account for the token to be redeemed
 * @param nftAccount: The NFT (account) that should be used to redeem 
 * 
 */
export const redeemLiquidity = async (wallet: PublicKey,walletATA: PublicKey,nftAccount:PublicKey,insuredTokenAccount:PublicKey) => {

    const liquidityPositionPDA = await getLiquidityPositionPDA(nftAccount);
    let liquidityPosition;
    try{
        liquidityPosition = await program.account.liquidityPosition.fetch(liquidityPositionPDA);
    }catch(e){
        throw new Error("could not get liquidity position: "+e)
    }
  
    const poolPDA = liquidityPosition.pool;
    const pool = await program.account.poolAccount.fetch(poolPDA)
    const tokenMint = liquidityPosition.tokenMint
    const tick = liquidityPosition.tick
    const nftMint = liquidityPosition.nftMint
    

    // Protocol Owner 
    let [protocolOwnerPDA,_] = await getProtocolOwner();

    let vaultAccountPDA = await getLiquidityVaultPDA(poolPDA,tokenMint);
    let tickAccount = await getTickAccountPDA(poolPDA,tokenMint,tick)
    let metadataAccountPDA = await getMpMetadataPDA(nftMint)
    try {
        await program.rpc.redeemLiquidity({
            accounts:{
                nftHolder: wallet,
                nftAccount: nftAccount,
                protocolOwner: protocolOwnerPDA,
                liquidityPosition: liquidityPositionPDA,
                tokenAccount: walletATA,
                vault: vaultAccountPDA,
                tickAccount: tickAccount,
                metadataAccount: metadataAccountPDA,
                metadataProgram: TokenMetadataProgram.publicKey,
                pool: poolPDA,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            }
        })
    }catch(err){
        throw new Error("sure.reedemLiquidity.error. cause: "+err)
    }
   
}



export const estimateYearlyPremium = async (amount: number,tokenMint:PublicKey,pool: PublicKey,owner:PublicKey): Promise<[amountCovered:anchor.BN,insurancePrice:anchor.BN]> => {
    const poolAccount = await program.account.poolAccount.fetch(pool);
    const insuranceFee = poolAccount.insuranceFee;

    /// Estimate premium 
    const tickSpacing = poolAccount.tickSpacing
    const firstBit = await getLowestBit(pool,tokenMint);
    const firstTick = getTickBasisPoint(firstBit,tickSpacing)

  
    console.log("available liquidity in pool: ",poolAccount.liquidity)
    
    
    // Check if there is enough
    let totalPremium = new anchor.BN(0);
    let amountToPay = new anchor.BN(0);
    let amountToCover = new anchor.BN(amount);
    let amountCovered = new anchor.BN(0)
    let tick = firstTick;

    // Get tick account
    let tickAccountPDA = await getTickAccountPDA(pool,tokenMint,firstTick)
    let tickAccount = await program.account.tick.fetch(tickAccountPDA)
    let availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);

    while (amountToCover.gt(new anchor.BN(0)) && tick !== -1 ) {
        console.log("tick tick")
        if (availableLiquidity.gte(new anchor.BN(amountToCover))){
            // Enough liquidity for one tick
            amountToPay = amountToCover;
        }else {
            // Buy all the liquidity for one tick
            amountToPay = availableLiquidity
        }

        // Buy insurance for tick
        totalPremium = totalPremium.add(amountToPay.muln(tick/10000))

        amountToCover = amountToCover.sub(amountToPay)

        // find next liquidity
        tick = await getNextTick(tick,pool,tokenMint,10)
        tickAccountPDA = await getTickAccountPDA(pool,tokenMint,tick)
        tickAccount = await program.account.tick.fetch(tickAccountPDA)
        availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);

        amountCovered = amountCovered.add(amountToPay)
        console.log("next tick: ",tick)
        console.log("amountToCover: ",amountToCover.toString())
    }

    // Add insurance fee 
    const sureFee = totalPremium.muln(insuranceFee/10000);
    const insurancePrice = totalPremium.add(sureFee)
    return [amountCovered,insurancePrice];
}

/**
 * Buy Insurance from the Liquidity pool
 * It's important that we can buy in as few transactions 
 * as possible
 * This means we have to do optimistic buying of insurance i.e. find how many 
 * ticks needs to be filled and then prepare the correct instructions
 * 
 * @param amount<number>: the amount of insurance to buy
 * @param pool<publickey>: the pool to buy from 
 * 
 */

export const buyInsurance = async (connection: anchor.web3.Connection,amount: number,tokenMint: PublicKey,pool: PublicKey,owner: PublicKey) => {
    
    const poolAccount = await program.account.poolAccount.fetch(pool);
    
    // Get insurance contract with pool
    const insuranceContractPDA = await getInsuranceContractPDA(pool,owner);

    try {
        await program.methods.initializeInsuranceContract().accounts({
            owner: owner,
            pool: pool,
            insuranceContract: insuranceContractPDA,
            systemProgram: SystemProgram.programId,
        }).rpc()
        const insuranceContract = await program.account.insuranceContract.fetch(insuranceContractPDA);

        assert.equal(insuranceContract.active,true);
        assert.equal(insuranceContract.amount.toString(),"0");
        assert.equal(insuranceContract.pool.toBase58(),pool.toBase58())
    } catch(err) {
        throw new Error("Could not initialize Insurance contract. Cause: " + err)
    }

   

    // ================= buy insurance by traversing ticks ==============
    const tickSpacing = poolAccount.tickSpacing
    const firstBit = await getLowestBit(pool,tokenMint);
    const firstTick = getTickBasisPoint(firstBit,tickSpacing)

   
    console.log("available liquidity in pool: ",poolAccount.liquidity)


    // Check if there is enough
    let amountToPay = new anchor.BN(0);
    let amountToCover = new anchor.BN(amount);
    let amountCovered = new anchor.BN(0)
    let tick = firstTick;
    let txs = new anchor.web3.Transaction()

    // Get tick account
    let tickAccountPDA = await getTickAccountPDA(pool,tokenMint,firstTick)
    let tickAccount = await program.account.tick.fetch(tickAccountPDA)
    let availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);

    while (amountToCover.gt(new anchor.BN(0)) && tick !== -1 ) {
        if (availableLiquidity.gte(new anchor.BN(amountToCover))){
            // Enough liquidity for one tick
            amountToPay = amountToCover;
        }else {
            // Buy all the liquidity for one tick
            amountToPay = availableLiquidity
        }

        // Buy insurance for tick
        txs.add(program.instruction.buyInsuranceForTick(amountToPay,{
            accounts:{
                buyer: owner,
                pool: pool,
                tickAccount:tickAccountPDA,
                insuranceContract: insuranceContractPDA,
                systemProgram: SystemProgram.programId,
            },
        }
        ))

        amountToCover = amountToCover.sub(amountToPay)

        // find next liquidity
        tick = await getNextTick(tick,pool,tokenMint,10)
        tickAccountPDA = await getTickAccountPDA(pool,tokenMint,tick)
        tickAccount = await program.account.tick.fetch(tickAccountPDA)
        availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);
        amountCovered = amountCovered.add(amountToPay)
        console.log("next tick: ",tick)
        console.log("amountToCover: ",amountToCover)
    }
    txs.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
    txs.feePayer = owner;
    
    try{
        await anchor.getProvider().send(txs)
    }catch(err){
        throw new Error("Sure.buyInsurance. Could not buy insurance. Cause: "+err)
    }


    console.log("Amount not covered: ",amountToCover)
}
