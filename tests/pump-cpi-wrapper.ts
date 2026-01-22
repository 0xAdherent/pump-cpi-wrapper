
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createMint,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";
import { expect } from "chai";
import { PumpCpiWrapper } from "../target/types/pump_cpi_wrapper";

describe("pump_cpi_wrapper", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.PumpCpiWrapper as Program<PumpCpiWrapper>;
  const admin = provider.wallet.publicKey;
  const pumpProgramId =  new PublicKey("7ybnARN6UmPDpV4T3BTcvkS7Nc6vtaQLHXQHFxnXuUNd");
  const METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
  const test_mint = new PublicKey("E4yxBZpFQwVsm8hvPV8NSiqaLSSFJGRuLCrakUkx9f9C");

  // Derive global config PDA (adjust seed if your program uses different)
  const [globalConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("global-config")],
    pumpProgramId
  );

  let tokenMint: PublicKey;
  let bondingCurve: PublicKey;
  let curveTokenAccount: PublicKey;

  it("Swaps SOL for tokens via cpi_swap (buy)", async () => {
    const user = admin;

    const amount = new anchor.BN(Math.floor(0.01 * LAMPORTS_PER_SOL));
    const minOut = new anchor.BN(1);


    tokenMint = test_mint;

    const userTokenAccount = getAssociatedTokenAddressSync(tokenMint, user);

    const [bondingCurvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding-curve"), tokenMint.toBuffer()],
      pumpProgramId
    );
    bondingCurve = bondingCurvePda;


    const [metadataPda] = PublicKey.findProgramAddressSync(
        [
        Buffer.from("metadata"),
        METADATA_PROGRAM_ID.toBuffer(),
        tokenMint.toBuffer(),
        ],
        METADATA_PROGRAM_ID
    );

    curveTokenAccount = getAssociatedTokenAddressSync(
      tokenMint,
      bondingCurve,
      true // allow owner off curve
    );

    // Ensure ATA exists (may be created by swap if using CPI with ATA program)
    try {
      await getOrCreateAssociatedTokenAccount(
        provider.connection,
        // @ts-ignore
        provider.wallet.payer,
        tokenMint,
        user
      );
    } catch (e) {
      // Might already exist; ignore
    }

    await program.methods
      .swap(amount, 0, minOut) // direction 0 = buy
      .accounts({
        user,
        globalConfig,
        feeRecipient: admin,
        bondingCurve,
        tokenMint,
        curveTokenAccount,
        userTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        pumpProgram: pumpProgramId,
      })
      .rpc({ skipPreflight: true });

    const userAta = await provider.connection.getTokenAccountBalance(userTokenAccount);
    expect(Number(userAta.value.amount)).to.be.greaterThan(0);
    console.log("token amount:", Number(userAta.value.amount));
  });

  it("Swaps tokens for SOL via cpi_swap (sell)", async () => {
    const user = admin;

    const userTokenAccount = getAssociatedTokenAddressSync(tokenMint, user);

    const balance = await provider.connection.getTokenAccountBalance(userTokenAccount);
    const amount = new anchor.BN(balance.value.amount);
    const minOut = new anchor.BN(1);

    console.log("before token amount:", Number(balance.value.amount));


    tokenMint = test_mint;

    const [bondingCurvePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("bonding-curve"), tokenMint.toBuffer()],
      pumpProgramId
    );
    bondingCurve = bondingCurvePda;

    curveTokenAccount = getAssociatedTokenAddressSync(
      tokenMint,
      bondingCurve,
      true // allow owner off curve
    );

    await program.methods
      .swap(amount, 1, minOut) // direction 1 = sell
      .accounts({
        user,
        globalConfig,
        feeRecipient: admin,
        bondingCurve,
        tokenMint,
        curveTokenAccount,
        userTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        pumpProgram: pumpProgramId,
      })
      .rpc({ skipPreflight: true });

    // Balance should decrease
    const userAta = await provider.connection.getTokenAccountBalance(userTokenAccount);
    expect(Number(userAta.value.amount)).to.be.equal(0);

    console.log("after token amount:", Number(userAta.value.amount));
    // You can add more precise checks based on expected slippage
  });

});
