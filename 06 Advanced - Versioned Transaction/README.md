# Part Six - Versioned Transaction

So you have completed the first 5 basic lessons! Congrats! It's time for learning deeper into Solana. Let's start with the first advanced lesson: Versioned Transaction.

This feature enables you to include many more accounts in a single transaction, making it possible to build more complex DeFi protocols, NFT marketplaces, and other applications that require interacting with numerous accounts simultaneously.

In this section, you'll:  
✅ Understand what Versioned Transactions are and why they're important  
✅ Learn about Address Lookup Tables (ALTs) and how they work  
✅ Convert legacy transactions to versioned transactions in your Bank App  
✅ Build a batch deposit feature that uses versioned transactions  

By the end of this part, you'll be able to create and send versioned transactions, unlocking the ability to build more complex and efficient Solana applications.  
Let's dive in! 🚀📦

### 🏦 Extending the Bank App: Batch Operations with Versioned Transactions

In the real world, banks often need to process multiple operations at once — like batch deposits from multiple users, or processing several token transfers in a single transaction. However, legacy Solana transactions have a limitation: they can only include about 35 accounts due to the 1,232-byte transaction size limit.

💡 That's where Versioned Transactions come in!  

In this session, we'll upgrade the Bank App to support batch deposits using versioned transactions. This will allow users to deposit multiple tokens in a single transaction, making the app more efficient and user-friendly.

### 1. What is a Versioned Transaction?

A Versioned Transaction is an enhanced transaction format introduced in Solana that supports Address Lookup Tables (ALTs). This allows transactions to reference up to 256 accounts instead of the ~35 limit in legacy transactions.

#### 🧠 Why Do We Need Versioned Transactions?

**Legacy Transaction Limitations:**
- Maximum transaction size: 1,232 bytes
- Each account address is 32 bytes
- This limits you to approximately 35 accounts per transaction
- This becomes a problem when you need to:
  - Process multiple token transfers
  - Interact with many accounts in a single operation
  - Build complex DeFi protocols that require many accounts

**Versioned Transaction Benefits:**
- Uses Address Lookup Tables (ALTs) to compress account references
- Can include up to 256 accounts in a single transaction
- More efficient for complex operations
- Essential for modern Solana applications

#### 🧩 How Do Address Lookup Tables Work?

An Address Lookup Table (ALT) is an on-chain account that stores a list of public keys. Instead of including full 32-byte addresses in your transaction, you can reference accounts by their index in the ALT (just 1 byte for indices 0-255).

**The Process:**
1. **Create an ALT**: Deploy a lookup table containing the public keys you'll frequently use
2. **Reference by Index**: In your transaction, reference accounts by their ALT index instead of full addresses
3. **Compress Transaction**: This dramatically reduces transaction size, allowing more accounts

Think of it like a phone book — instead of writing out full addresses every time, you just reference an entry number.

### 2. Understanding Transaction Versions

Solana supports two transaction versions:

**Legacy Transactions (Version 0x00):**
- The original transaction format
- Accounts are included directly in the transaction
- Limited to ~35 accounts
- Still widely used and fully supported

**Versioned Transactions (Version 0x01):**
- The new transaction format
- Uses Address Lookup Tables
- Supports up to 256 accounts
- Recommended for complex applications

### 3. Creating and Using Versioned Transactions

Let's see how to create and send versioned transactions in your TypeScript client code.

#### 🛠️ Step 1: Create an Address Lookup Table

First, you need to create an ALT that will store the accounts you want to reference:

```typescript
import { 
  AddressLookupTableProgram, 
  TransactionMessage,
  VersionedTransaction,
  Connection,
  Keypair,
  PublicKey
} from "@solana/web3.js";

// Create a new Address Lookup Table
async function createLookupTable(
  connection: Connection,
  payer: Keypair
): Promise<PublicKey> {
  const [lookupTableInst, lookupTableAddress] = 
    AddressLookupTableProgram.createLookupTable({
      authority: payer.publicKey,
      payer: payer.publicKey,
      recentSlot: await connection.getSlot(),
    });

  const transaction = new Transaction().add(lookupTableInst);
  await connection.sendTransaction(transaction, [payer]);

  return lookupTableAddress;
}
```

#### 🛠️ Step 2: Add Accounts to the Lookup Table

Once created, you need to add accounts to your ALT:

```typescript
async function addAccountsToLookupTable(
  connection: Connection,
  payer: Keypair,
  lookupTable: PublicKey,
  accounts: PublicKey[]
): Promise<string> {
  const extendInstruction = AddressLookupTableProgram.extendLookupTable({
    payer: payer.publicKey,
    authority: payer.publicKey,
    lookupTable: lookupTable,
    addresses: accounts,
  });

  const transaction = new Transaction().add(extendInstruction);
  const signature = await connection.sendTransaction(transaction, [payer]);
  await connection.confirmTransaction(signature);
  return signature;
}
```

#### 🛠️ Step 3: Create a Versioned Transaction

Now you can create a versioned transaction that uses the ALT:

```typescript
async function createVersionedTransaction(
  connection: Connection,
  payer: Keypair,
  instructions: TransactionInstruction[],
  lookupTable: PublicKey
): Promise<VersionedTransaction> {
  const { blockhash, lastValidBlockHeight } = 
    await connection.getLatestBlockhash();
  
  const lookupTableAccount = await connection.getAddressLookupTable(lookupTable);
  if (!lookupTableAccount || !lookupTableAccount.value) {
    throw new Error("Lookup table not found or not activated yet");
  }
  // Compile the message to v0 format with the lookup table
  const messageV0 = new TransactionMessage({
    payerKey: payer.publicKey,
    recentBlockhash: blockhash,
    instructions: instructions,
  }).compileToV0Message([lookupTableAccount.value]);
  
  // Create a versioned transaction
  const transaction = new VersionedTransaction(messageV0);
  transaction.sign([payer]);

  return transaction;
}
```

#### 🛠️ Step 4: Send the Versioned Transaction

Finally, send the transaction:

```typescript
async function sendVersionedTransaction(
  connection: Connection,
  transaction: VersionedTransaction
): Promise<string> {
  const signature = await connection.sendTransaction(transaction);
  await connection.confirmTransaction(signature);
  return signature;
}
```

### 4.Example: Batch Token Deposits

Let's apply this to the Bank App. We'll create a feature that allows users to deposit multiple different tokens in a single transaction.

#### 🧱 Overview

Instead of making separate transactions for each token deposit, users can:
- Deposit multiple SPL tokens at once
- Save on transaction fees
- Improve user experience

#### 🛠️ Implementation in bank-app.ts

Here's how you would implement batch token deposits using versioned transactions:

```typescript
import {
  AddressLookupTableProgram,
  TransactionMessage,
  VersionedTransaction,
  Transaction,
  TransactionInstruction,
  Connection,
  SystemProgram,
  PublicKey,
} from "@solana/web3.js";
import { AnchorProvider, Program } from "@coral-xyz/anchor";
import { 
  getAssociatedTokenAddressSync, 
  createAssociatedTokenAccountInstruction,
  TOKEN_PROGRAM_ID 
} from "@solana/spl-token";
import { BN } from "bn.js";

async function batchDepositTokens(
  connection: Connection,
  provider: AnchorProvider,
  program: Program<BankApp>,
  tokenMints: PublicKey[],
  amounts: InstanceType<typeof BN>[],
  BANK_APP_ACCOUNTS: {
    bankInfo: PublicKey;
    bankVault: PublicKey;
    userReserve: (pubkey: PublicKey, tokenMint?: PublicKey) => PublicKey;
  }
) {
  // Step 1: Prepare all accounts we'll need
  const accounts: PublicKey[] = [];
  const instructions: TransactionInstruction[] = [];
  
  // Add common accounts first
  accounts.push(program.programId);
  accounts.push(SystemProgram.programId);
  accounts.push(TOKEN_PROGRAM_ID);
  accounts.push(BANK_APP_ACCOUNTS.bankInfo);
  accounts.push(BANK_APP_ACCOUNTS.bankVault);
  accounts.push(provider.publicKey);

  // For each token, prepare accounts and instructions
  for (let i = 0; i < tokenMints.length; i++) {
    const tokenMint = tokenMints[i];
    const amount = amounts[i];
    
    const userAta = getAssociatedTokenAddressSync(
      tokenMint,
      provider.publicKey
    );
    const bankAta = getAssociatedTokenAddressSync(
      tokenMint,
      BANK_APP_ACCOUNTS.bankVault,
      true
    );
    const userReserve = BANK_APP_ACCOUNTS.userReserve(
      provider.publicKey,
      tokenMint
    );

    // Add accounts to our list (avoid duplicates)
    const newAccounts = [tokenMint, userAta, bankAta, userReserve];
    for (const account of newAccounts) {
      if (!accounts.find(a => a.equals(account))) {
        accounts.push(account);
      }
    }

    // Check if bank ATA exists, create if needed
    if (await connection.getAccountInfo(bankAta) == null) {
      instructions.push(
        createAssociatedTokenAccountInstruction(
          provider.publicKey,
          bankAta,
          BANK_APP_ACCOUNTS.bankVault,
          tokenMint
        )
      );
    }

    // Add deposit instruction
    instructions.push(
      await program.methods
        .depositToken(amount)
        .accounts({
          bankInfo: BANK_APP_ACCOUNTS.bankInfo,
          bankVault: BANK_APP_ACCOUNTS.bankVault,
          tokenMint: tokenMint,
          userAta: userAta,
          bankAta: bankAta,
          userReserve: userReserve,
          user: provider.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .instruction()
    );
  }

  // Step 2: Create or get existing lookup table
  let lookupTable: PublicKey;
  // In production, you'd want to reuse an existing lookup table
  // For this example, we'll create a new one
  const [lookupTableInst, lookupTableAddress] =
    AddressLookupTableProgram.createLookupTable({
      authority: provider.publicKey,
      payer: provider.publicKey,
      recentSlot: await connection.getSlot(),
    });

  const createTableTx = new Transaction().add(lookupTableInst);
  const createTableSig = await connection.sendTransaction(createTableTx, [provider.wallet.payer]);
  await connection.confirmTransaction(createTableSig);

  lookupTable = lookupTableAddress;

  // Wait for lookup table to be ready
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Step 3: Add accounts to lookup table
  const extendInstruction = AddressLookupTableProgram.extendLookupTable({
    payer: provider.publicKey,
    authority: provider.publicKey,
    lookupTable: lookupTable,
    addresses: accounts,
  });

  const extendTx = new Transaction().add(extendInstruction);
  const extendSig = await connection.sendTransaction(extendTx, [provider.wallet.payer]);
  await connection.confirmTransaction(extendSig);

  // Wait for extension to be ready
  await new Promise(resolve => setTimeout(resolve, 1000));

  // Step 4: Create versioned transaction
  const { blockhash, lastValidBlockHeight } =
    await connection.getLatestBlockhash();

  // Fetch the lookup table account (required for versioned transactions)
  const lookupTableAccount = await connection.getAddressLookupTable(lookupTable);
  if (!lookupTableAccount || !lookupTableAccount.value) {
    throw new Error("Lookup table not found or not activated yet");
  }

  const messageV0 = new TransactionMessage({
    payerKey: provider.publicKey,
    recentBlockhash: blockhash,
    instructions: instructions,
  }).compileToV0Message([lookupTableAccount.value]);

  const versionedTransaction = new VersionedTransaction(messageV0);
  versionedTransaction.sign([provider.wallet.payer]);

  // Step 5: Send transaction
  const signature = await connection.sendTransaction(versionedTransaction);
  await connection.confirmTransaction(signature);

  console.log("Batch deposit signature:", signature);
  return signature;
}
```

#### ✅ What's Happening Here?

1. **Account Collection**: We gather all unique accounts needed for all token deposits
2. **Instruction Building**: We create instructions for each token deposit, including ATA creation if needed
3. **Lookup Table Creation**: We create an ALT to store all these accounts
4. **Account Extension**: We add all accounts to the ALT
5. **Versioned Transaction**: We compile the transaction to v0 format using the ALT
6. **Execution**: We send and confirm the transaction

This approach allows us to deposit many tokens in a single transaction, which would be impossible with legacy transactions if we exceed the account limit.

### 5. When to Use Versioned Transactions

Use versioned transactions when:
- ✅ You need to interact with more than ~35 accounts in a single transaction
- ✅ You're building complex DeFi protocols
- ✅ You want to batch multiple operations efficiently
- ✅ You're building NFT marketplaces or other multi-account applications

Stick with legacy transactions when:
- ✅ Your transaction has fewer than ~35 accounts
- ✅ You want maximum compatibility
- ✅ You're building simple applications

### 6. Important Considerations

#### ⚠️ Lookup Table Lifecycle

- **Creation**: Creating an ALT requires a transaction and takes time to become active
- **Extension**: You can add accounts to an ALT later, but it also requires a transaction
- **Deactivation**: ALTs can be deactivated (but not deleted) by the authority
- **Reusability**: In production, reuse existing ALTs rather than creating new ones for each transaction

#### ⚠️ Transaction Confirmation

Versioned transactions work the same as legacy transactions for confirmation:
- Always wait for confirmation before assuming success
- Use `confirmTransaction` or `getSignatureStatus` to verify
- Handle errors appropriately

### 7. Time to Build 💪

Now it's time to apply everything you've learned! You'll complete a set of guided exercises to add versioned transaction support to your Bank App. You will use the existing bank-app program in lessons to do it.
🛠️ Your Tasks:

1. **Create a Helper Function for Lookup Tables**  
   Write a reusable function that:
   - Creates an ALT if it doesn't exist
   - Extends it with new accounts if needed
   - Returns the lookup table address
   - Handles the timing/waiting for ALT activation

2. **Implement Batch Token Deposit**  
   Create a function that allows users to deposit multiple tokens in a single versioned transaction:
   - Accept an array of token mints and amounts
   - Create all necessary instructions
   - Use a lookup table to compress the transaction
   - Send as a versioned transaction

3. **Add Batch SOL Deposit**  
   Extend the batch functionality to support multiple SOL deposits from different users (if you have a multi-user scenario) or batch operations for the same user.

4. **Write Tests**  
   As always, write comprehensive tests:
   - Test batch token deposits with 2-3 different tokens
   - Verify all deposits succeed
   - Test error handling (e.g., insufficient balance)
   - Test with legacy transactions for comparison

5. **Optimize for Production**  
   Consider improvements:
   - Reuse lookup tables instead of creating new ones
   - Cache lookup table addresses
   - Handle lookup table extension more efficiently

Once you've completed these tasks, your Bank App will support efficient batch operations using versioned transactions — a key feature for production-ready Solana applications! 🚀

### 🎓 Key Takeaways

- Versioned transactions use Address Lookup Tables to support up to 256 accounts
- ALTs compress account references, reducing transaction size
- Versioned transactions are essential for complex DeFi and multi-account applications
- Always wait for ALT activation before using it in transactions
- Reuse lookup tables in production for efficiency

