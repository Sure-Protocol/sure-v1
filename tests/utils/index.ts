import { SurePool } from "../../target/types/sure_pool";
import {PublicKey} from "@solana/web3.js"
import {getOrCreateAssociatedTokenAccount} from "@solana/spl-token"
import * as anchor from "@project-serum/anchor"
import { Program, Spl } from "@project-serum/anchor";
import { token } from "@project-serum/anchor/dist/cjs/utils";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAccount, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import {TokenAccount} from "./types"
import {  metaplex } from "@metaplex/js/lib/programs";
import { TokenMetadataProgram, TokenProgram } from "@metaplex-foundation/js-next";
import { metadata } from "@metaplex/js/lib/programs";
import { assert } from "chai";
import NodeWallet from "@project-serum/anchor/dist/cjs/nodewallet";

import { Connection } from "@metaplex/js";
import { publicKey } from "@solana/buffer-layout-utils";
import { Wallet } from "@project-serum/anchor/dist/cjs/provider";

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
export const SURE_INSURANCE_CONTRACTS = anchor.utils.bytes.utf8.encode("sure-insurance-contracts")
export const SURE_POOLS_SEED = anchor.utils.bytes.utf8.encode("sure-pools")
// Export sub libs
export * from "./nft"
export * from "./bitmap"

import * as bitmap from "./bitmap"
import { min } from "bn.js";


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

export const getLiquidityPositionBitmapPDA = async (pool: PublicKey,tokenMint: PublicKey,program: anchor.Program<SurePool>): Promise<anchor.web3.PublicKey> => {
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



/// Check if tick account exists for the pool, 
/// if not, create the account. 
export const createTickAccount = async (poolPDA: PublicKey,tokenMint: PublicKey,tick: number,creator: PublicKey): Promise<PublicKey> => {
    
    const tickAccountPDA = await getTickAccountPDA(poolPDA,tokenMint,tick);

    try{
        await program.rpc.initializeTick(poolPDA,tokenMint,tick, {
            accounts: {
                creator:creator,
                tickAccount: tickAccountPDA,
                systemProgram: SystemProgram.programId,
            },
        })
    } catch(e){
        console.log("logs?: ",e.logs)
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

export const getInsuranceContractPDA = async (tickAccount: PublicKey,owner: PublicKey):Promise<PublicKey> => {

    const [insuranceContractPDA,insuranceContractBump] = await PublicKey.findProgramAddress(
       [
        SURE_INSURANCE_CONTRACT,
        owner.toBytes(),
        tickAccount.toBytes(),
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
    const bitmapPDA = await getLiquidityPositionBitmapPDA(poolPDA,tokenMint,program)
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

export const getSurePools = async (): Promise<PublicKey> => {

    const [surePoolsPDA,surePoolsBump] = await PublicKey.findProgramAddress(
        [
            SURE_POOLS_SEED,
        ],
        program.programId,
    )

    return surePoolsPDA
}


export const estimateYearlyPremium = async (amount: number,tokenMint:PublicKey,pool: PublicKey,owner:PublicKey): Promise<[amountCovered:anchor.BN,insurancePrice:anchor.BN]> => {
    const poolAccount = await program.account.poolAccount.fetch(pool);
    const insuranceFee = poolAccount.insuranceFee;

    /// Estimate premium 
    const tickSpacing = poolAccount.tickSpacing
    let bitmapPDA = await getLiquidityPositionBitmapPDA(pool,tokenMint,program);
    const firstBit = await bitmap.getLowestBit(bitmapPDA);
    const firstTick = bitmap.getTickBasisPoint(firstBit,tickSpacing)

  
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
        
        bitmapPDA = await getLiquidityPositionBitmapPDA(pool,tokenMint,program)
        tick = await bitmap.getNextTick(tick,bitmapPDA,10)
        tickAccountPDA = await getTickAccountPDA(pool,tokenMint,tick)
        tickAccount = await program.account.tick.fetch(tickAccountPDA)
        availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);

        amountCovered = amountCovered.add(amountToPay)
    }

    // Add insurance fee 
    const sureFee = totalPremium.muln(insuranceFee/10000);
    const insurancePrice = totalPremium.add(sureFee)
    return [amountCovered,insurancePrice];
}


/**
 * Create insurance contract for given tick 
 * The insurance contract holds information about 
 * 
 * @param owner<publickey>: Owner of insurance contract
 * @param tickAccount<publickey>: The tick to buy insurance from
 * 
 */
export const createInsuranceContractForTick = async (owner: PublicKey,tickAccount: PublicKey,pool: PublicKey,tokenMint: PublicKey,userInsuranceContractsPDA: PublicKey): Promise<PublicKey> => {
    // Get insurance contract with pool
    const insuranceContractPDA = await getInsuranceContractPDA(tickAccount,owner);

    try {
        await program.methods.initializeInsuranceContract().accounts({
            owner: owner,
            pool: pool,
            tokenMint: tokenMint,
            tickAccount: tickAccount,
            insuranceContract: insuranceContractPDA,
            insuranceContracts: userInsuranceContractsPDA,
            systemProgram: SystemProgram.programId,
        }).rpc()
        const insuranceContract = await program.account.insuranceContract.fetch(insuranceContractPDA);

        assert.equal(insuranceContract.active,true);
        assert.equal(insuranceContract.amount.toString(),"0");
        assert.equal(insuranceContract.pool.toBase58(),pool.toBase58())

    } catch(err) {
        throw new Error("Could not initialize Insurance contract. Cause: " + err)
    }

    return insuranceContractPDA
}

/**
 * Get or create insurance contract for given tick 
 * The insurance contract holds information about 
 * 
 * @param owner<publickey>: Owner of insurance contract
 * @param tickAccount<publickey>: The tick to buy insurance from
 * 
 */
export const getOrCreateInsuranceContractForTick  = async (owner: PublicKey,tickAccount: PublicKey,pool: PublicKey,tokenMint: PublicKey,userInsuranceContractsPDA:PublicKey): Promise<PublicKey> => {
    const insuranceContractPDA = await getInsuranceContractPDA(tickAccount,owner);
    
    try{
        const insuranceContract = await program.account.insuranceContract.getAccountInfo(insuranceContractPDA)
        if (insuranceContract !== null){
            return insuranceContractPDA
        }
        throw new Error()
    }catch(_){
        // Insurance contract does not exist. Create it
        await createInsuranceContractForTick(owner,tickAccount,pool,tokenMint,userInsuranceContractsPDA)
    }

    return insuranceContractPDA
}

export const getInsuranceContractsBitmapPDA = async (owner: PublicKey,pool: PublicKey): Promise<PublicKey> => {

    const [insuranceContractsPDA,insuranceContractsBump] = await PublicKey.findProgramAddress(
        [
            SURE_INSURANCE_CONTRACTS,
            owner.toBytes(),
            pool.toBytes()
        ],
        program.programId,
    )
    return insuranceContractsPDA
}

/**
 * Calculate the amount insured by user
 * 
 * @param owner<publickey>: Owner of insurance contract
 * @param tokenMint<publickey>: the mint account publickkey
 * @param pool<PublicKey>: the pool to buy insurance from
 * 
 * @returns Promise for a Big Number - BN
 */
export const getInsuredAmount = async (owner: PublicKey,tokenMint: PublicKey,pool:PublicKey): Promise<anchor.BN> => {
    const userInsuranceContractsPDA = await getInsuranceContractsBitmapPDA(owner, pool);
    const userInsuranceContracts = await program.account.bitMap.fetch(userInsuranceContractsPDA)
    
    // Create insurance contract bitmap 
    const insuranceContractBitmap = bitmap.Bitmap.new(userInsuranceContracts)

    // Start from right and reduce position
    let currentTick = insuranceContractBitmap.getHighestTick();
    
    let amount = new anchor.BN(0);
    while (currentTick !== -1) {
        const tickAccountPDA = await getTickAccountPDA(pool,tokenMint,currentTick);
        const insuranceContractForTickPDA =await getInsuranceContractPDA(tickAccountPDA,owner);
        const insuranceContractForTick = await program.account.insuranceContract.fetch(insuranceContractForTickPDA)
        amount = amount.add(insuranceContractForTick.amount)
        currentTick = insuranceContractBitmap.getPreviousTick(currentTick)
    }  
    return amount
}


export const createUserInsuranceContracts = async (owner: PublicKey,pool:PublicKey): Promise<PublicKey> => {
    try {
        const insuranceContractPDA = await getInsuranceContractsBitmapPDA(owner,pool)
        await program.methods.initializeUserInsuranceContracts().accounts({
            signer: owner,
            pool: pool,
            insuranceContracts: insuranceContractPDA,
            systemProgram: SystemProgram.programId,
        }).rpc()

        return insuranceContractPDA
    }catch(err){
        throw new Error("Could not initialize Insurance Contracts. Cause: "+err)
    }
}

export const getOrCreateUserInsuranceContracts = async (owner: PublicKey,pool:PublicKey): Promise<PublicKey> => {
    const insuranceContractPDA = await getInsuranceContractsBitmapPDA(owner,pool) 
    try{
       const res = await program.account.bitMap.getAccountInfo(insuranceContractPDA)
       if (res !== null){
           return insuranceContractPDA
       }
       throw new Error()
    }catch(_){
        return createUserInsuranceContracts(owner,pool)
    }
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
export const buyInsurance = async (connection: anchor.web3.Connection,amount: number,endTimestamp: number,tokenMint: PublicKey,pool: PublicKey,owner: Wallet) => {
    
    const poolAccount = await program.account.poolAccount.fetch(pool);
    
    // Get the premium vault for the given pool
    const premiumVaultPDA = await getPremiumVaultPDA(pool,tokenMint);

    // Get correct associated token account 
    const tokenAccount = await getOrCreateAssociatedTokenAccount(
        connection,
        (owner as NodeWallet).payer,
        tokenMint,
        owner.publicKey,    
    )

    // ================= buy insurance by traversing ticks ==============
    const tickSpacing = poolAccount.tickSpacing
    let insurancePositionBitmapPDA = await getLiquidityPositionBitmapPDA(pool,tokenMint,program)
    const firstBit = await bitmap.getLowestBit(insurancePositionBitmapPDA);
    const firstTick = bitmap.getTickBasisPoint(firstBit,tickSpacing)

    // Create insurance overview 
    const userInsuranceContractsPDA = await getOrCreateUserInsuranceContracts(owner.publicKey,pool)

    // Check if there is enough
    let insuranceContractPDA;
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


        // Get or create insurance for tick
        insuranceContractPDA = await getOrCreateInsuranceContractForTick(owner.publicKey,tickAccountPDA,pool,tokenMint,userInsuranceContractsPDA);
       
        // Buy insurance for tick
        txs.add(program.instruction.buyInsuranceForTick(amountToPay,new anchor.BN(endTimestamp),{
            accounts:{
                buyer: owner.publicKey,
                tokenAccount: tokenAccount.address,
                pool: pool,
                tickAccount:tickAccountPDA,
                premiumVault: premiumVaultPDA,
                insuranceContract: insuranceContractPDA,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            },
        }
        ))

        amountToCover = amountToCover.sub(amountToPay)

        // find next liquidity
        insurancePositionBitmapPDA = await getLiquidityPositionBitmapPDA(pool,tokenMint,program)
        tick = await bitmap.getNextTick(tick,insurancePositionBitmapPDA,10)
        tickAccountPDA = await getTickAccountPDA(pool,tokenMint,tick)
        tickAccount = await program.account.tick.fetch(tickAccountPDA)
        availableLiquidity = tickAccount.liquidity.sub(tickAccount.usedLiquidity);
        amountCovered = amountCovered.add(amountToPay)
    }
    txs.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
    txs.feePayer = owner.publicKey;
    
    try{
        await anchor.getProvider().send(txs)
    }catch(err){
        throw new Error("Sure.buyInsurance. Could not buy insurance. Cause: "+err)
    }
}

/**
 * Reduce Insurance Amount 
 * Move from largest tick insurance position and move downwards until 
 * the position is reduced 
 * 
 * @param amount<number>: the amount of insurance to buy
 * @param pool<publickey>: the pool to buy from 
 * 
 */
export const reduceInsuranceAmount = async (connection: Connection,newInsuranceAmount: number,pool:PublicKey,tokenMint: PublicKey,owner: Wallet) => {

    // Load Accounts
    const tokenAccount = await getOrCreateAssociatedTokenAccount(connection,(owner as NodeWallet).payer,tokenMint,owner.publicKey);
    const premiumVaultPDA = await getPremiumVaultPDA(pool,tokenMint);


    // Calculate amount insured 
    const amountInsured = await getInsuredAmount(owner.publicKey,tokenMint,pool)
    const newInsuranceAmountBN = new anchor.BN(newInsuranceAmount)
    let insuranceReductionAmount = amountInsured.sub(newInsuranceAmountBN) 
    if (insuranceReductionAmount.lte(new anchor.BN(0))){
        throw new Error("Amount insured is less than new insured amount")
    }

    // Get Insurance Contract Bitmap
    const userInsuranceContractsPDA = await getInsuranceContractsBitmapPDA(owner.publicKey, pool);
    const userInsuranceContracts = await program.account.bitMap.fetch(userInsuranceContractsPDA)
    
    // Create insurance contract bitmap 
    const insuranceContractBitmap = bitmap.Bitmap.new(userInsuranceContracts)

    // Start from right and reduce position
    let currentTick = insuranceContractBitmap.getHighestTick();

    
    // Create Anchor Transaction
    let txs = new anchor.web3.Transaction()
    // Initialize parameters
    let tickAccountPDA;
    let insuranceContractForTickPDA;
    let insuranceContract;
    let amountToReduceForTick;

    // Reduce position tick for tick 
    while (insuranceReductionAmount.gt(new anchor.BN(0))){
        tickAccountPDA = await getTickAccountPDA(pool,tokenMint,currentTick);
        insuranceContractForTickPDA =await getInsuranceContractPDA(tickAccountPDA,owner.publicKey);
        insuranceContract = await program.account.insuranceContract.fetch(insuranceContractForTickPDA)
        amountToReduceForTick = min(insuranceContract.amount,insuranceReductionAmount);
        txs.add(
            program.instruction.reduceInsuranceAmountForTick(amountToReduceForTick,{
                accounts: {
                    holder: owner.publicKey,
                    pool: pool,
                    tickAccount: tickAccountPDA,
                    tokenAccount: tokenAccount.address,
                    premiumVault: premiumVaultPDA,
                    insuranceContract: insuranceContractForTickPDA,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId
                }
            })
        )

        insuranceReductionAmount = insuranceReductionAmount.sub(amountToReduceForTick)
        // Get the previous tick in the bitmap
        currentTick = insuranceContractBitmap.getPreviousTick(currentTick)
    }
    txs.recentBlockhash = (await connection.getLatestBlockhash()).blockhash
    txs.feePayer = owner.publicKey;
    
    try{
        await anchor.getProvider().send(txs)
    }catch(err){
        console.log("logs?: ",err?.logs)
        throw new Error("Sure.buyInsurance. Could not buy insurance. Cause: "+err)
    }

}
