import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InstructionOrdering } from "../target/types/instruction_ordering";
import { expect } from "chai";
import {
  SYSVAR_INSTRUCTIONS_PUBKEY,
  SystemProgram,
  Keypair,
  Transaction
} from "@solana/web3.js";

describe("instruction-ordering", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.InstructionOrdering as Program<InstructionOrdering>;

  // Derive the state PDA
  const [statePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state")],
    program.programId
  );

  it("succeeds when initialize is first", async () => {
    const tx = new Transaction().add(
      await program.methods
        .initialize()
        .accounts({
          state: statePda,
          authority: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .instruction()
    );

    await provider.sendAndConfirm(tx);

    const state = await program.account.state.fetch(statePda);
    expect(state.authority.toBase58()).to.equal(provider.wallet.publicKey.toBase58());
    expect(state.counter.toNumber()).to.equal(0);
    expect(state.isFinalized).to.equal(false);

    console.log("✅ Initialize succeeded as first instruction");
  });

  it("fails when initialize is not first", async () => {
    // Create a dummy transfer instruction to go first
    const dummyKeypair = Keypair.generate();
    const dummyIx = SystemProgram.transfer({
      fromPubkey: provider.wallet.publicKey,
      toPubkey: dummyKeypair.publicKey,
      lamports: 1000,
    });

    // Create a new state account for this test
    const testStatePda = Keypair.generate();

    const initIx = await program.methods
      .initialize()
      .accounts({
        state: testStatePda.publicKey,
        authority: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
        instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .instruction();

    const tx = new Transaction()
      .add(dummyIx)  // This makes initialize NOT first
      .add(initIx);

    try {
      await provider.sendAndConfirm(tx, [testStatePda]);
      expect.fail("Should have failed - initialize was not first");
    } catch (err) {
      expect(err.message).to.include("MustBeFirstInstruction");
      console.log("✅ Initialize correctly failed when not first");
    }
  });

  it("can increment counter", async () => {
    await program.methods
      .increment()
      .accounts({
        state: statePda,
      })
      .rpc();

    const state = await program.account.state.fetch(statePda);
    expect(state.counter.toNumber()).to.equal(1);
    console.log("✅ Counter incremented to:", state.counter.toNumber());
  });

  it("can increment multiple times", async () => {
    // Increment twice more
    await program.methods
      .increment()
      .accounts({ state: statePda })
      .rpc();

    await program.methods
      .increment()
      .accounts({ state: statePda })
      .rpc();

    const state = await program.account.state.fetch(statePda);
    expect(state.counter.toNumber()).to.equal(3);
    console.log("✅ Counter incremented to:", state.counter.toNumber());
  });

  it("succeeds when finalize is last", async () => {
    // Create transaction with increment first, then finalize
    const tx = new Transaction()
      .add(
        await program.methods
          .increment()
          .accounts({ state: statePda })
          .instruction()
      )
      .add(
        await program.methods
          .finalize()
          .accounts({
            state: statePda,
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .instruction()
      );

    await provider.sendAndConfirm(tx);

    const state = await program.account.state.fetch(statePda);
    expect(state.isFinalized).to.equal(true);
    expect(state.counter.toNumber()).to.equal(4);
    console.log("✅ Finalized successfully as last instruction");
  });

  it("fails when finalize is not last", async () => {
    // Create new state for this test
    const testStatePda = Keypair.generate();

    // Initialize first
    const initTx = new Transaction().add(
      await program.methods
        .initialize()
        .accounts({
          state: testStatePda.publicKey,
          authority: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .instruction()
    );

    await provider.sendAndConfirm(initTx, [testStatePda]);

    // Create dummy instruction to go after finalize
    const dummyIx = SystemProgram.transfer({
      fromPubkey: provider.wallet.publicKey,
      toPubkey: Keypair.generate().publicKey,
      lamports: 1000,
    });

    // Try to finalize with another instruction after it
    const tx = new Transaction()
      .add(
        await program.methods
          .finalize()
          .accounts({
            state: testStatePda.publicKey,
            instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
          })
          .instruction()
      )
      .add(dummyIx);  // This makes finalize NOT last

    try {
      await provider.sendAndConfirm(tx);
      expect.fail("Should have failed - finalize was not last");
    } catch (err) {
      expect(err.message).to.include("MustBeLastInstruction");
      console.log("✅ Finalize correctly failed when not last");
    }
  });

  it("finalize works as only instruction", async () => {
    // Create new state
    const testStatePda = Keypair.generate();

    // Initialize first
    const initTx = new Transaction().add(
      await program.methods
        .initialize()
        .accounts({
          state: testStatePda.publicKey,
          authority: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
          instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
        })
        .instruction()
    );

    await provider.sendAndConfirm(initTx, [testStatePda]);

    // Finalize as only instruction in transaction
    await program.methods
      .finalize()
      .accounts({
        state: testStatePda.publicKey,
        instructions: SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .rpc();

    const state = await program.account.state.fetch(testStatePda.publicKey);
    expect(state.isFinalized).to.equal(true);
    console.log("✅ Finalize succeeded as only instruction");
  });
});