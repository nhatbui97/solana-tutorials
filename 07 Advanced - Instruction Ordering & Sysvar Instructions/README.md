# 07 – Instruction Ordering, Sysvar & Runtime Constraints

In this lesson, we dive deep into how Solana executes instructions within a transaction and why the order of those instructions matters. You'll learn how to use the **Sysvar Instructions** account for instruction introspection, enabling your on-chain programs to inspect and enforce constraints on the transaction they're running in.


---

Before starting this lesson, you should be comfortable with:

- **PDAs (Program Derived Addresses)** – Understanding how programs derive and own accounts
- **ATAs (Associated Token Accounts)** – Token account management patterns
- **CPI (Cross-Program Invocation)** – How programs call other programs
- **Transaction structure** – Accounts, signers, instructions
If you need a refresher, review the earlier lessons (03, 04, 05) in this course.

---


By the end of this lesson, you will:

- How Solana executes instructions: sequential and atomic
- Why instruction order affects security, correctness, and composability
- How to use the Sysvar Instructions account for instruction introspection
- How to enforce instruction ordering inside on-chain programs
- When to use `invoke_signed` instead of CPI to control instruction stack growth
- How Solana’s stack size and account limits affect complex transactions
- Practical strategies for designing transactions that do not exceed runtime limits
- How to test and debug instruction-order–dependent transactions
---

## How Solana Executes Transactions 

A Solana transaction is a batch of instructions executed sequentially by the runtime, it executes them **in order**, **one at a time**:


```
Transaction {
  instructions: [Ix0, Ix1, Ix2, Ix3]
}

Execution:
1. Ix0 executes → modifies accounts → success
2. Ix1 executes → modifies accounts → success
3. Ix2 executes → modifies accounts → success
4. Ix3 executes → modifies accounts → success

Result: Transaction succeeds, all account changes committed.
```
And transaction is **atomic**. If a single instruction fails, the entire transaction will fail and no changes will occur.
Think of a transaction as a single atomic database transaction with multiple operations:

```
BEGIN TRANSACTION;
  operation1();  -- Ix0
  operation2();  -- Ix1
  operation3();  -- Ix2
COMMIT;
```

If `operation2()` fails, the whole thing rolls back.

Example: Sequential State Modification

```rust
// Instruction 0: Initialize counter to 0
counter.value = 0;

// Instruction 1: Increment counter
counter.value += 1;  // Now 1

// Instruction 2: Increment counter
counter.value += 1;  // Now 2

// Transaction succeeds, counter.value = 2
```

The final state depends on the execution order. If Ix1 and Ix2 ran before Ix0, the counter would be 0.

---

## Why Instruction Order Matters

Instruction order affects three critical areas:

### 1. Correctness

Programs often have logical dependencies between operations:

``` RUST
Bad order:
1. Transfer tokens
2. Verify signature  ❌ Too late!

Good order:
1. Verify signature
2. Transfer tokens   ✅ Secure
```

### 2. Security

```rust
// ❌ WRONG: Check after mutate
pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    // Change data first
    ctx.accounts.vault.balance -= amount;
    ctx.accounts.user.balance += amount;
    
    // Check second
    require!(ctx.accounts.authority.is_signer ErrorCode::Unauthorized);
    
    Ok(())
}
```
We always verify preconditions before changing state. Fix:
```rust
// ✅ CORRECT: Check before mutate
pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
    // Check first
    require!(ctx.accounts.authority.is_signer, ErrorCode::Unauthorized);
    require!(ctx.accounts.vault.balance >= amount, ErrorCode::InsufficientFunds);
    
    // Then mutate
    ctx.accounts.vault.balance -= amount;
    ctx.accounts.user.balance += amount;
    
    Ok(())
}
```


### 3. Composability

When your program is called via CPI, you need to ensure:

- **Preconditions** are met before your instruction runs
- **Postconditions** hold after your instruction completes
- **Invariants** are maintained across instruction boundaries

Example: A DEX swap program should verify slippage *before* executing the swap, not after.

---

## Sysvar Instruction Account

 **Sysvar Instruction** is a special account that contains information about all instructions in the current instructions.

 **Program ID**: `Sysvar1nstructions1111111111111111111111111` 

 It provides:
- Current instruction index
- Total number of instructions
- Program ID, accounts, and data for each instruction
- Which programs are being invoked  

So, when to use it?
- You want a instruction must be the first instruction, the last instruction or the only instruction in a transaction.
- You want the signer can only be authorized by directly calling through this program, cannot be called by CPI.
- Verify specific instructions ran before yours
- Force the user to do steps in the right order in one transaction

Here's how you implement it:
```rust
use solana_program::sysvar::instructions;

// Get current instruction index
let current_index = instructions::load_current_index_checked(ix_account)?;

// Load a specific instruction
let instruction = instructions::load_instruction_at_checked(index,ix_account)?;
```

## Implementation example

Here i will implement a simple example: initialization must be first instruction in any transaction.

```rust
use anchor_lang::prelude::*;
use solana_program::sysvar::instructions;

declare_id!("Your11111111111111111111111111111111111111");

#[program]
pub mod instruction_ordering {
    use super::*;
    
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Get current instruction index
        let current_index = instructions::load_current_index_checked(
            &ctx.accounts.instructions.to_account_info()
        )?;
        
        // Must be first (index 0)
        require!(
            current_index == 0,
            ErrorCode::MustBeFirstInstruction
        );
        
        // Initialize state
        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.authority.key();
        state.counter = 0;
        
        msg!("Initialized as first instruction");
        Ok(())
    }
    
    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        ctx.accounts.state.counter += 1;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, State>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    
    /// CHECK: Instructions sysvar
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct Increment<'info> {
    #[account(
        mut,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, State>,
}

#[account]
pub struct State {
    pub authority: Pubkey,
    pub counter: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Initialize must be the first instruction")]
    MustBeFirstInstruction,
}
```

Test file:

```typescript
import * as anchor from "@coral-xyz/anchor";
import { expect } from "chai";
import { SYSVAR_INSTRUCTIONS_PUBKEY, SystemProgram, Keypair } from "@solana/web3.js";

describe("instruction-ordering", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  
  const program = anchor.workspace.InstructionOrdering;
  
  const [statePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state")],
    program.programId
  );
  
  it("succeeds when initialize is first", async () => {
    const tx = new anchor.web3.Transaction().add(
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
    expect(state.counter.toNumber()).to.equal(0);
  });
  
  it("fails when initialize is not first", async () => {
    const dummyIx = SystemProgram.transfer({
      fromPubkey: provider.wallet.publicKey,
      toPubkey: Keypair.generate().publicKey,
      lamports: 1,
    });
    
    const tx = new anchor.web3.Transaction()
      .add(dummyIx)
      .add(
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
    
    try {
      await provider.sendAndConfirm(tx);
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.message).to.include("MustBeFirstInstruction");
    }
  });
  
  it("can increment after initialization", async () => {
    await program.methods
      .increment()
      .accounts({
        state: statePda,
      })
      .rpc();
    
    const state = await program.account.state.fetch(statePda);
    expect(state.counter.toNumber()).to.equal(1);
  });
});
```


## Common Patterns
### Pattern 1: Must Be First

```rust
let current = instructions::load_current_index_checked(ixs)?;
require!(current == 0, ErrorCode::MustBeFirst);
```

### Pattern 2: Must Have Previous Instruction

```rust
let current = instructions::load_current_index_checked(ixs)?;
require!(current > 0, ErrorCode::NoPreviousInstruction);

let prev = instructions::load_instruction_at_checked(current - 1, ixs)?;
// Check prev.program_id, prev.data, etc.
```

### Pattern 3: Count Total Instructions

```rust
let mut total = 0;
loop {
    match instructions::load_instruction_at_checked(total, ixs) {
        Ok(_) => total += 1,
        Err(_) => break,
    }
    if total >= 64 { break; }  // Solana max
}

require!(total <= 5, ErrorCode::TooManyInstructions);
```
## Runtime Constraints: Stack Depth & Account Limits
Once you start enforcing instruction order and composing multiple instructions in a single transaction, you will quickly encounter Solana’s runtime limits:

- **Instruction call stack depth**
- **Maximum number of accounts per instruction**
- **Compute budget limits**
- **Access violation in stack frame**
...
### CPI call depth - CallDepth error
Cross-program invocations allow programs to invoke other programs directly, but the depth is constrained currently to 4
What this means:
```
Transaction instruction (depth 0)
  → Program A (depth 1)
    → CPI to Program B (depth 2)
      → CPI to Program C (depth 3)
        → CPI to Program D (depth 4)
          → ❌ CPI to Program E would fail!
```

### Call stack depth - CallDepthExceeded error
Solana programs are constrained to run quickly, and to facilitate this, the program's call stack is limited to a max depth of 64 frames.

When a program exceeds the allowed call stack depth limit, it will receive the CallDepthExceeded error.



### Account Limit Per Instruction
Maximum **30 accounts** per instruction if using legacy transaction (including duplicates)


### Transaction Size

**Limit**: Maximum transaction size is **1232 bytes** (legacy) or **1280 bytes** (v0)

What this affects:
- Number of instructions
- Number of accounts
- Size of instruction data
- Number of signatures


### "Access Violation in Stack Frame" Errors

Solana’s BPF runtime gives each instruction handler only 4 KiB of stack space. Anchor’s derive macro deserializes all fields of your `Accounts` struct including account data and instruction arguments—onto the stack. If there are too many fields or very large accounts, this can exceed the 4 KiB limit and cause a runtime “Access violation in stack frame” error, equivalent a stack overflow.

It will looks like this, if you ever encountered:
```
Program {program_address} invoke [1]
Program log: Instruction: Foo
Program {program_address} consumed 7000 of 1000000 compute units
Program failed to complete: Access violation in stack frame 7 at address 0x200007fd8 of size 8 by instruction #4170
Program {program_address} failed: Program failed to complete
```



## invoke_signed - CPI
### The Problem with CPI chains 

When you use Anchor's `CpiContext`, you're building a CPI call that adds to the call stack:

```rust
// This is CPI (adds to call stack depth)
let cpi_ctx = CpiContext::new_with_signer(
    ctx.accounts.token_program.to_account_info(),
    Transfer {
        from: ctx.accounts.vault.to_account_info(),
        to: ctx.accounts.user.to_account_info(),
        authority: ctx.accounts.vault_authority.to_account_info(),
    },
    signer_seeds,
);

token::transfer(cpi_ctx, amount)?;
```
When using CPI, the stack is: 
```
Your code
 → Anchor wrapper
   → SPL Token wrapper
     → invoke_signed
       → SPL Token program
```

### The Solution: invoke_signed

`invoke_signed` is a lower-level function that can be more efficient for simple operations, by using `invoke_signed`, the stack is shallower:

```rust
use solana_program::program::invoke_signed;

// This is direct invoke_signed (more efficient)
let ix = spl_token::instruction::transfer(
    &spl_token::ID,
    &ctx.accounts.vault.key(),
    &ctx.accounts.user.key(),
    &ctx.accounts.vault_authority.key(),
    &[],
    amount,
)?;

invoke_signed(
    &ix,
    &[
        ctx.accounts.vault.to_account_info(),
        ctx.accounts.user.to_account_info(),
        ctx.accounts.vault_authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
    ],
    &[signer_seeds],
)?;
```




## Handling the Overflow

### Account Limit
There are more than one solutions for this:
#### Multiple Instructions
Instead of one instruction with 200 accounts, create multiple instructions:

```rust
// ❌ Won't work: Too many accounts
pub fn process_all_users(ctx: Context<ProcessAll>) -> Result<()> {
    // Tries to process 200 user accounts at once
    for user in ctx.remaining_accounts.iter() {
        // process...
    }
    Ok(())
}

// ✅ Works: Process in batches
pub fn process_batch(ctx: Context<ProcessBatch>, start: u32, end: u32) -> Result<()> {
    // Process users from start to end (max 100 per call)
    for i in start..end {
        let user = &ctx.remaining_accounts[(i - start) as usize];
        // process...
    }
    Ok(())
}
```

#### Lookup Tables (Advanced)

Use Address Lookup Tables from versioned transactions (covered in Lesson 06):

```typescript
// Create lookup table with many accounts
const lookupTable = await createLookupTable(connection, authority, [
  account1, account2, /* ... 200 more accounts ... */
]);

// Use in transaction (counts as 1 account reference)
const ix = await program.methods
  .processMany()
  .accounts({ /* ... */ })
  .instruction();

const message = new TransactionMessage({
  payerKey: authority.publicKey,
  recentBlockhash: blockhash,
  instructions: [ix],
}).compileToV0Message([lookupTable]);

const tx = new VersionedTransaction(message);
```

### Stack Frame Optimization
To optimize the memory usage in Solana, you can use these techniques:
#### Allocate Big Account on the Heap
It’s generally a good practice to allocate larger accounts on the heap. While you can’t allocate programs this way and heap accounts need to be deserialized (So you can't do it on Unchecked Accounts), this method can save a significant amount of stack space.
``` rust
Box<Account<'info,BigAccount>>
```
Use this when the account is:
- Large
- Needs to be modified 
- Too big for the stack

But this will cost you an extra amount to compute, so make sure you have a plan in mind.

#### Use Helper function
Wait, does more functions = more stack frames??? Yes, but only one of those frames exists at a time.
The trick is not reducing depth, it’s reducing **frame size**.

```rust
fn monster() {
    let huge_a: [u8; 20_000];
    let huge_b: [u8; 20_000];
    let huge_c: [u8; 20_000];

    do_something(&huge_a);
    do_something_else(&huge_b);
    do_another(&huge_c);
}
```
All three of them live in the same stack frame. Even if they're used at different times, the complier reserve space for all of them **at once**. 


```rust
fn monster() {
    sub_a();
    sub_b();
    sub_c();
}

#[inline(never)]
fn sub_a() {
    let huge_a: [u8; 20_000];
    do_something(&huge_a);
}

#[inline(never)]
fn sub_b() {
    let huge_b: [u8; 20_000];
    do_something_else(&huge_b);
}

#[inline(never)]
fn sub_c() {
    let huge_c: [u8; 20_000];
    do_another(&huge_c);
}
```


Now in the runtime the stack looks like this:
```
monster frame
  -> sub_a frame (huge_a)
monster frame
  -> sub_b frame (huge_b)
monster frame
  -> sub_c frame (huge_c)
```

This is not guaranteed by the Rust complier, you can enforce it using `#[inline(never)]` by strongly suggesting compiler against inlining a specific function, which could potentially crash the program.



#### Use Remaining Accounts
This is not directly a way to handle the stack frame violation, but more like preventing your Context from using huge memory.

You only put the account you **always** need in the `Accounts` struct. Anything optional, or variables, goes in `remaining_accounts`.

The reason we use `remaining_accounts` is that everything you put in `Accounts` struct will be in stack, while `remaining_accounts` will live in the heap.

``` rust
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MyDynamicIx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,

    /// Collect all extra accounts here
    #[account(mut)]
    pub remaining_accounts: Vec<AccountInfo<'info>>,
}

pub fn my_dynamic_ix(ctx: Context<MyDynamicIx>) -> Result<()> {
    for acc in ctx.remaining_accounts.iter() {
        msg!("Extra account: {}", acc.key());
    }
    Ok(())
}
```


```typescript
// extraPubkeys: PublicKey[] from your business logic
const remainingAccounts = extraPubkeys.map(pk => ({
  pubkey: pk,
  isWritable: true,
  isSigner: false,
}));

await program.rpc.myDynamicIx({
  accounts: {
    payer: provider.wallet.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  },
  remainingAccounts,
});
```



#### Don't Use AccountInfo

If, for some reason, you are not validating or checking a specific account (perhaps it’s not accessed directly or it’s being validated by another instruction), you can use `UncheckedAccount<’info>` instead of `AccountInfo<’info>`. It serves the same purpose in this context, is more explicit about not performing checks, and takes up considerably less space.


#### Zero-Copy
When an account becomes very large (often after being grown over multiple transactions), loading it normally with `Account<T>` or even `Box<Account<T>>` can cause out-of-memory or stack violations, because Anchor tries to deserialize the whole account into your program’s memory.

Zero-copy avoids that. Instead of copying the data into stack or heap, your program reads the bytes directly from the account’s memory buffer. Basically, you don't move the mountain, you work where the mountain directly is.


## Debugging 

### 1. Log Instruction Index

```rust
let current = instructions::load_current_index_checked(ixs)?;
msg!("Current instruction index: {}", current);
```

### 2. Simulate Before Sending

```typescript
const simulation = await connection.simulateTransaction(tx, [wallet.payer]);

if (simulation.value.err) {
  console.error("Failed:", simulation.value.err);
  console.log("Logs:", simulation.value.logs);
}
```

### 3. Check Transaction Logs

```
Program YourProgram invoke [1]
Program log: Current instruction index: 0
Program YourProgram success
```


