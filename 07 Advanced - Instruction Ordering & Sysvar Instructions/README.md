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

### Example
When an instruction is hitting the stack limit, then you can simply split one instruction into two, with sysvar as the constraint. Here say you force a deposit function before a withdraw function:

```rust
use anchor_lang::prelude::*;
use solana_program::sysvar::instructions as ix_sysvar;

declare_id!("Your11111111111111111111111111111111111111");

// In real code, compute this once off-chain via Anchor and paste it here.
const DEPOSIT_DISCRIMINATOR: [u8; 8] = [0; 8]; // TODO: fill with real bytes

#[program]
pub mod simple_flow {
    use super::*;

    pub fn deposit(_ctx: Context<Deposit>, _amount: u64) -> Result<()> {
        // ... your normal deposit logic ...
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, _amount: u64) -> Result<()> {
        // --- 1) Read current instruction index ---
        let ix_acc = &ctx.accounts.instructions.to_account_info();
        let current_index = ix_sysvar::load_current_index_checked(ix_acc)?;

        // We need at least one instruction before this one
        require!(current_index > 0, ErrorCode::MustDepositFirst);

        // --- 2) Load previous instruction ---
        let prev_ix = ix_sysvar::load_instruction_at_checked(
            (current_index - 1) as usize,
            ix_acc,
        )?;

        // Must be from this program
        require!(prev_ix.program_id == crate::ID, ErrorCode::MustDepositFirst);

        // --- 3) Check discriminator = `deposit` ---
        require!(prev_ix.data.len() >= 8, ErrorCode::MustDepositFirst);
        let disc = &prev_ix.data[0..8];
        require!(disc == DEPOSIT_DISCRIMINATOR, ErrorCode::MustDepositFirst);

        // --- 4) Now do normal withdraw logic ---
        // ... your withdrawal logic here ...

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    // whatever accounts you need
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    // whatever accounts you need
    pub user: Signer<'info>,

    /// CHECK: instructions sysvar
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Transaction must have a deposit immediately before withdraw")]
    MustDepositFirst,
}
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

#### Zero-Copy
When an account becomes very large (often after being grown over multiple transactions), loading it normally with `Account<T>` or even `Box<Account<T>>` can cause out-of-memory or stack violations, because Anchor tries to deserialize the whole account into your program’s memory.

Zero-copy avoids that. Instead of copying the data into stack or heap, your program reads the bytes directly from the account’s memory buffer. Basically, you don't move the mountain, you work where the mountain directly is.

Normally, when you write:
```rust
let vault = &mut ctx.accounts.vault;
```
Anchor gives a Rust struct that lives on the stack.
If the account struct is small, then no problem. But what if it becomes large:
```rust
#[account]
pub struct BigData {
    pub owner: Pubkey,
    pub values: [u64; 2000],   // <- big!
}
```
then each time an instruction loads it, Rust tries to copy all that on to the stack.

With zero-copy, you point directly to the account data:
```rust
#[account(zero_copy)]
#[repr(packed)]
pub struct BigData {
    pub owner: Pubkey,
    pub values: [u64; 2000],
}
```
And instead of `Account<T>`, we use, `AccountLoader<BigData>`.

Then we access it like this:
```rust
let mut data = ctx.accounts.big_data.load_mut()?;
```

So why don't we use Zero-Copy all the time?
We only use when storing fixed-size structured data like: 
- positions
- order books
- large arrays
- history buffer

We don't use if:
- you expect the layout to change later
- you want simpler, safer code
- you need dynamic fields:
  - Vec<T>
  - String
  - optional / variable-length data

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





## Exercise
In this exercise, you'll implement a **sequential approval system** that requires two instructions in order, then optimize it with zero-copy to handle large data.

### Part 1: Sequential Approval
Build a system where `execute` can only run if `approve` ran immediately before it in the same transaction.
1. Create an `approve` instruction that records approval
2. Create an `execute` instruction that:
   - Checks if previous instruction was `approve`
   - Verifies it's from the same program
   - Only executes if approval is valid




### Part 2: Optimize with Zero-Copy
Now create a version that handles large data efficiently using zero-copy.

Requirements:
- Add a LargeApprovalData struct with array of 512 u64 values
- Compare regular Account<T> vs AccountLoader<T>
- Measure which approach avoids stack overflow. Observe how a large regular Account<T> runs into BPF stack limit errors at build time, while a zero-copy AccountLoader<T> can handle the same-sized data safely.
