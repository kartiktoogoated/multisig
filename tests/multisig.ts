import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Multisig } from "../target/types/multisig";
import { Keypair, PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

// Helper to create TransactionAccount type
function makeTxAccount(pubkey: PublicKey, isSigner: boolean, isWritable: boolean) {
  return { pubkey, isSigner, isWritable };
}

describe("multisig", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.multisig as Program<Multisig>;
  const provider = anchor.getProvider();

  // Generate 3 owners
  const owners = [Keypair.generate(), Keypair.generate(), Keypair.generate()];
  let multisigPda: PublicKey;
  let multisigBump: number;
  let multisigAccount: PublicKey;
  let nonce: number;
  // txAccount is shared across tests that need to reference the same transaction
  let txAccount: Keypair;

  before(async () => {
    // Derive multisig PDA
    [multisigPda, multisigBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("multisig"), provider.wallet.publicKey.toBuffer()],
      program.programId
    );
    multisigAccount = multisigPda;
    nonce = multisigBump;
  });

  it("Creates a multisig", async () => {
    await program.methods
      .createMultisig(
        owners.map(o => o.publicKey),
        2, // threshold
        nonce
      )
      .accountsStrict({
        multisig: multisigAccount,
        payer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    const multisig = await program.account.multisig.fetch(multisigAccount);
    expect(multisig.owners.map((k: PublicKey) => k.toBase58())).to.deep.equal(owners.map(o => o.publicKey.toBase58()));
    expect(multisig.threshold).to.equal(2);
    expect(multisig.nonce).to.equal(nonce);
  });

  it("Proposes a transaction", async () => {
    const txAccount = Keypair.generate();
    const preInstructions = [
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: txAccount.publicKey,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(1000),
        space: 1000,
        programId: program.programId,
      }),
    ];
    // Create a dummy transaction: send lamports from multisig to owner[0]
    const targetProgram = SystemProgram.programId;
    const txAccounts = [
      makeTxAccount(multisigAccount, true, true),
      makeTxAccount(owners[0].publicKey, false, true),
      makeTxAccount(SystemProgram.programId, false, false),
    ];
    const data = SystemProgram.transfer({
      fromPubkey: multisigAccount,
      toPubkey: owners[0].publicKey,
      lamports: 1_000,
    }).data;
    await program.methods
      .proposeTransaction(targetProgram, txAccounts, data)
      .accountsStrict({
        transaction: txAccount.publicKey,
        multisig: multisigAccount,
        proposer: provider.wallet.publicKey, // use test wallet as proposer
        systemProgram: SystemProgram.programId,
      })
      .signers([txAccount])
      .preInstructions(preInstructions)
      .rpc();
    const tx = await program.account.transaction.fetch(txAccount.publicKey);
    expect(tx).to.exist;
    expect(tx.programId.toBase58()).to.equal(targetProgram.toBase58());
    expect(tx.signers[0]).to.be.true; // proposer signed
  });

  it("Approves a transaction by another owner", async () => {
    // Propose a new transaction first
    const txAccount = Keypair.generate();
    const preInstructions = [
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: txAccount.publicKey,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(1000),
        space: 1000,
        programId: program.programId,
      }),
    ];
    const targetProgram = SystemProgram.programId;
    const txAccounts = [
      makeTxAccount(multisigAccount, true, true),
      makeTxAccount(owners[0].publicKey, false, true),
      makeTxAccount(SystemProgram.programId, false, false),
    ];
    const data = SystemProgram.transfer({
      fromPubkey: multisigAccount,
      toPubkey: owners[0].publicKey,
      lamports: 1_000,
    }).data;
    await program.methods
      .proposeTransaction(targetProgram, txAccounts, data)
      .accountsStrict({
        transaction: txAccount.publicKey,
        multisig: multisigAccount,
        proposer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([txAccount])
      .preInstructions(preInstructions)
      .rpc();
    // Now approve
    await program.methods
      .approveTransaction()
      .accountsStrict({
        transaction: txAccount.publicKey,
        multisig: multisigAccount,
        owner: owners[1].publicKey,
      })
      .signers([owners[1]])
      .rpc();
    const tx = await program.account.transaction.fetch(txAccount.publicKey);
    expect(tx.signers[1]).to.be.true;
  });

  it("Fails to execute transaction with not enough signers", async () => {
    // Propose a new transaction with only one approval
    const txAccount2 = Keypair.generate();
    const preInstructions = [
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: txAccount2.publicKey,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(1000),
        space: 1000,
        programId: program.programId,
      }),
    ];
    const targetProgram = SystemProgram.programId;
    const txAccounts = [
      makeTxAccount(multisigAccount, true, true),
      makeTxAccount(owners[0].publicKey, false, true),
      makeTxAccount(SystemProgram.programId, false, false),
    ];
    const data = SystemProgram.transfer({
      fromPubkey: multisigAccount,
      toPubkey: owners[0].publicKey,
      lamports: 1_000,
    }).data;
    await program.methods
      .proposeTransaction(targetProgram, txAccounts, data)
      .accountsStrict({
        transaction: txAccount2.publicKey,
        multisig: multisigAccount,
        proposer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([txAccount2])
      .preInstructions(preInstructions)
      .rpc();
    try {
      await program.methods
        .executeTransaction()
        .accountsStrict({
          transaction: txAccount2.publicKey,
          multisig: multisigAccount,
          multisigSigner: PublicKey.findProgramAddressSync([
            multisigAccount.toBuffer(),
          ], program.programId)[0],
        })
        .rpc();
      throw new Error("Should have failed");
    } catch (e: any) {
      console.log("Actual error (not enough signers):", e.toString());
      expect(e.message).to.include("Not enough signers");
    }
  });

  it("Executes transaction after enough approvals", async () => {
    // Propose a new transaction
    const txAccount = Keypair.generate();
    const preInstructions = [
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        newAccountPubkey: txAccount.publicKey,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(1000),
        space: 1000,
        programId: program.programId,
      }),
    ];
    const targetProgram = SystemProgram.programId;
    const txAccounts = [
      makeTxAccount(multisigAccount, true, true),
      makeTxAccount(owners[0].publicKey, false, true),
      makeTxAccount(SystemProgram.programId, false, false),
    ];
    const data = SystemProgram.transfer({
      fromPubkey: multisigAccount,
      toPubkey: owners[0].publicKey,
      lamports: 1_000,
    }).data;
    await program.methods
      .proposeTransaction(targetProgram, txAccounts, data)
      .accountsStrict({
        transaction: txAccount.publicKey,
        multisig: multisigAccount,
        proposer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([txAccount])
      .preInstructions(preInstructions)
      .rpc();
    // Approve with third owner
    await program.methods
      .approveTransaction()
      .accountsStrict({
        transaction: txAccount.publicKey,
        multisig: multisigAccount,
        owner: owners[2].publicKey,
      })
      .signers([owners[2]])
      .rpc();
    // Now execute
    await program.methods
      .executeTransaction()
      .accountsStrict({
        transaction: txAccount.publicKey,
        multisig: multisigAccount,
        multisigSigner: PublicKey.findProgramAddressSync([
          multisigAccount.toBuffer(),
        ], program.programId)[0],
      })
      .rpc();
    const tx = await program.account.transaction.fetch(txAccount.publicKey);
    expect(tx.didExecute).to.be.true;
  });

  it("Fails to create multisig with duplicate owners", async () => {
    const payer = Keypair.generate();
    // Airdrop SOL to payer
    const sig = await provider.connection.requestAirdrop(payer.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig);
    const [dupPda, dupBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("multisig"), payer.publicKey.toBuffer()],
      program.programId
    );
    try {
      await program.methods
        .createMultisig(
          [owners[0].publicKey, owners[0].publicKey],
          2,
          dupBump
        )
        .accountsStrict({
          multisig: dupPda,
          payer: payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([payer])
        .rpc();
      throw new Error("Should have failed");
    } catch (e: any) {
      console.log("Actual error (duplicate owners):", e.toString());
      expect(e.toString()).to.include("Owners must be unique.");
    }
  });

  it("Fails to create multisig with invalid threshold", async () => {
    const payer = Keypair.generate();
    // Airdrop SOL to payer
    const sig = await provider.connection.requestAirdrop(payer.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.confirmTransaction(sig);
    const [invPda, invBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("multisig"), payer.publicKey.toBuffer()],
      program.programId
    );
    try {
      await program.methods
        .createMultisig(
          [owners[0].publicKey, owners[1].publicKey],
          3, // threshold > owners
          invBump
        )
        .accountsStrict({
          multisig: invPda,
          payer: payer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([payer])
        .rpc();
      throw new Error("Should have failed");
    } catch (e: any) {
      console.log("Actual error (invalid threshold):", e.toString());
      expect(e.toString()).to.include("Threshold must be valid and ≤ number of owners.");
    }
  });
});
