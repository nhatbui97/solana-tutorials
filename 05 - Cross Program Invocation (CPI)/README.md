# Part Five - Cross Program Invocation (CPI)
Now that you know how to work with tokens and ATAs, it's time to explore one of the most powerful features of Solana smart contracts — Cross-Program Invocations (CPIs).

CPIs allow one program to securely call and execute instructions in another program. This is how DeFi apps integrate with token programs, oracles, staking vaults, and more — enabling modular, composable applications across the Solana ecosystem.

In this section, you’ll:  
✅ Understand what CPI is and how it works under the hood in Anchor  
✅ Walk through an example: staking SOL from the Bank App using CPI  
✅ Build a simple spl-token Staking App and integrate it with the Bank App using CPI  

By the end of this part, you’ll be able to interact with external programs and build protocols that can compose with the rest of the Solana ecosystem — a key superpower for serious Solana development.

Let’s dive in! 🔄💡

### 🏦 Extending the Bank App: Investing via CPI
You already know that in the real world, banks don’t just hold users’ deposits — they put that money to work, investing it to earn returns. Our Bank App is about to do the same.  

In this session, we’ll upgrade the Bank App with the ability to invest user funds into external protocols using Cross-Program Invocation (CPI). This is how real DeFi vaults, lending platforms, and DAOs grow capital while keeping everything on-chain and auditable.  

We’ll support two new functions that empower the Bank authority to manage investments:
+ Allows the bank authority to invest SOL from the bank vault into another dApp
+ Lets the bank authority withdraw previously invested SOL from the dApp back into the vault

💡 This pattern forms the foundation of yield strategies, automated vaults, and treasury management systems in Solana DeFi.

### 1. What is CPI?
CPI, or Cross-Program Invocation, is a feature in Solana that allows one program to call and execute instructions in another program — securely and permissionlessly.  

Think of it like calling a function from another module — except both "modules" are on-chain programs. This enables composability, meaning you can build applications that reuse logic from existing programs like the Token Program, staking protocols, lending markets, and more.

#### 🧠 Why is CPI important?
✅ Reusability – No need to reinvent the wheel; just call existing programs.  
✅ Modularity – Build clean, maintainable apps by splitting logic across programs.  
✅ Interoperability – Your program can interact with DeFi protocols, DAOs, or other custom apps.  

#### 🧩 How does CPI work?
When one program wants to call another, it performs a Cross-Program Invocation:
1. It prepares the required accounts and any instruction data.
2. It wraps these into a `CpiContext`, optionally including signer seeds if the calling program is using a PDA as authority.
3. It calls the target program’s CPI helper function.Anchor internally constructs and sends the CPI instruction using Solana’s runtime.
4. Anchor internally constructs and sends the CPI instruction using Solana’s runtime.  

In low-level Rust, this would involve `invoke()` or `invoke_signed()`, but with Anchor, you typically never need to call those directly — Anchor handles it for you under the hood.  
This is a clean, safe, and ergonomic way to perform CPIs in Anchor.  

In this tutorial, your **Bank App** will call your **Staking App** using CPI to stake or withdraw SOL on behalf of users. This is exactly how real DeFi protocols like yield vaults or auto-compounders operate.

### 2. Real-World Example: Investing SOL from the Bank App into the Staking App
Let’s see CPI in action with a real-world use case.  
In this section, we’ll upgrade the Bank App so the bank authority can invest SOL from the Bank Vault into an external Staking App — using a CPI call. This mirrors how real DeFi protocols put idle capital to work to generate yield.  

#### 🧱 Overview of the Staking App

Before we wire up the CPI, let’s first walk through the Staking App — a simple program that allows users (or another program) to stake and unstake SOL.  

Here’s how it works:
+ Users interact with a single `Stake` instruction that supports both staking and unstaking, depending on the `is_stake` boolean.
+ The app pays out a fixed APR of 5% to stakers.
+ The contract uses PDAs to store `UserInfo` accounts to track stake balance

📂 You can find the code in programs/staking-app:
```rust
    pub fn stake(ctx: Context<Stake>, amount: u64, is_stake: bool) -> Result<()> {
        ...
    }

    #[derive(Accounts)]
    pub struct Stake<'info> {
        /// CHECK:
        #[account(
            init_if_needed,
            payer = payer,
            seeds = [b"STAKING_VAULT"],
            bump,
            space = 0,
            owner = system_program::ID
        )]
        pub staking_vault: UncheckedAccount<'info>,

        #[account(
            init_if_needed,
            seeds = [b"USER_INFO", user.key().as_ref()],
            bump,
            payer = payer,
            space = 8 + std::mem::size_of::<UserInfo>(),
        )]
        pub user_info: Box<Account<'info, UserInfo>>,

        #[account(mut)]
        pub user: Signer<'info>,
        #[account(mut)]
        pub payer: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
```
> 📝 Note: There’s only one instruction `Stake` used for both staking and unstaking — controlled by the `is_stake` flag. This keeps the code DRY, since the logic is nearly identical for both actions.  

Notice there are two signer accounts:
+ `user`: The logical owner of the stake (this will be our Bank Vault PDA)
+ `payer`: The signer who pays the account creation fee (rent fee)  

This separation is perfect for our Bank App because the Bank Vault PDA (used as user) won’t have to fund rent fees and the bank authority (a real signer) can cover rent costs during CPI. This makes integration smooth — no need to pre-fund the vault with lamports just to create a new user info account.

#### 🚀 Ready to Integrate
With the Staking App already deployed on devnet, there’s no need to redeploy or modify it. You can simply reuse the code and reference the same program ID when wiring up CPI from the Bank App.  
Now that we’ve chosen our target investment app (the Staking App), let’s integrate it into the Bank App using Cross-Program Invocation (CPI).

First, add the staking-app as a dependency in the Bank App’s `Cargo.toml`:
```toml
[dependencies]
...
staking-app = {  path = "../staking-app", features = ["cpi"] }
```
This gives the Bank App access to the Staking App’s CPI interface, allowing us to call its stake instruction directly from our program.

#### 🧱 Minor Refactor: Organizing Instructions by Role
As our Bank App grows, it’s a good time to do a bit of housekeeping.  
In this part, we’ll refactor the project structure to better organize instructions based on who is allowed to execute them:
+ `instructions/user/` — for instructions that regular users can call (e.g. deposit, withdraw)
+ `instructions/authority/` — for privileged instructions the bank authority can execute  

This makes the codebase more scalable and readable. The new `invest` instruction — where the bank authority stakes or unstakes SOL — will live in:
```bash
instructions/authority/invest.rs
```

#### 🛠️ Writing the `invest` Instruction
Now that our Bank App is CPI-ready, let’s implement the actual `invest` instruction, which allows the Bank authority to stake or unstake SOL into the external Staking App.  
Here's the full code: 
```rust
#[derive(Accounts)]
pub struct Invest<'info> {
    #[account(
        seeds = [BANK_INFO_SEED],
        bump
    )]
    pub bank_info: Box<Account<'info, BankInfo>>,

    /// CHECK: Bank Vault (PDA) that holds SOL deposits
    #[account(
        mut,
        seeds = [BANK_VAULT_SEED],
        bump,
        owner = system_program::ID
    )]
    pub bank_vault: UncheckedAccount<'info>,

    /// CHECK: CPI target staking vault
    #[account(mut)]
    pub staking_vault: UncheckedAccount<'info>,

    /// CHECK: CPI target user staking info (PDA)
    #[account(mut)]
    pub staking_info: UncheckedAccount<'info>,

    /// The Staking App program to invoke via CPI
    pub staking_program: Program<'info, StakingApp>,

    /// Bank authority — only this signer can invest
    #[account(mut, address = bank_info.authority)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}
```
**👇 Let’s break down a few key CPI-related accounts:**
+ `staking_vault`: The vault account owned by the Staking App. This is where SOL is stored once staked.

+ `staking_info`: This is the user's staking metadata account in the Staking App. In our case, the “user” is the Bank Vault PDA — so this acts as a `UserInfo` record tied to the bank’s vault.

+ `staking_program`: A reference to the Staking App itself, so the Bank App can perform a Cross-Program Invocation (CPI)

#### 🧠 Inside the Logic:`process()`
Here’s the most important part — the CPI call itself:
```rust
cpi::stake(
    CpiContext::new_with_signer(
        ctx.accounts.staking_program.to_account_info(),
        cpi::accounts::Stake {
            staking_vault: ctx.accounts.staking_vault.to_account_info(),
            user_info: ctx.accounts.staking_info.to_account_info(),
            user: ctx.accounts.bank_vault.to_account_info(),
            payer: ctx.accounts.authority.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        },
        invest_vault_seeds,
    ),
    amount,
    is_stake,
)?;
```

##### `cpi::stake(...)`
This function is a wrapper generated by Anchor that allows you to call an instruction from another program (in this case, the Staking App):
- The function name `stake` corresponds to the CPI-accessible `stake` instruction in the staking-app program.
- It’s imported through the CPI interface thanks to the dependency setup in `Cargo.toml.`
- Under the hood, Anchor generates a function here that:
  1. Builds the instruction.  
  2. Prepares the account metas.  
  3. Uses `invoke_signed` to make the actual CPI call if signer seeds are provided.

> ✅ Key idea: This looks like a regular Rust function call — but it's actually executing another program on-chain!

This small block is where the actual cross-program interaction happens — using just a few lines, we can safely and securely route funds from our Bank App into a yield-generating staking strategy.

##### `CpiContext::new_with_signer(...)`
This is how you construct the execution context for a CPI call when your program needs to sign on behalf of a PDA.  

Parameters:
+ `program`: The target program’s AccountInfo — in our case, staking_program.
+ `accounts`: The CPI-compatible version of the account struct that the target program expects. Here we're using `cpi::accounts::Stake`, which is a struct matching the one defined in staking-app.
+ `signer_seeds`:A reference to the PDA seeds used to re-derive and sign on behalf of the `bank_vault` PDA.

##### `cpi::accounts::Stake`
This is the CPI version of the `Stake` context defined in the `staking-app`:
```rust
#[derive(Accounts)]
pub struct Stake<'info> {
    pub staking_vault: AccountInfo<'info>,
    pub user_info: AccountInfo<'info>,
    pub user: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}
```
> Anchor automatically generates a mod cpi::accounts for every program you import with features = ["cpi"].  
You're responsible for manually wiring the right accounts here using `.to_account_info()` from your context.

#### ✅ Summary
This block:
```rust
cpi::stake(CpiContext::new_with_signer(...), amount, is_stake)?;
```
is how the Bank App securely calls into the Staking App to stake or unstake SOL. It:
+ Builds a CPI context (`CpiContext`)
+ Provides signer seeds so the `bank_vault` PDA can authorize the transaction
+ Passes structured account data that matches the staking app’s expected inputs
+ Calls the `stake` instruction in the other program — as if it were part of the current one  
This pattern demonstrates the power of composability on Solana — your program can call into any other program and build rich, interconnected logic  

#### 🧑‍💻 What’s Next
So, you've now seen a real-world example of how the Bank App can invest user deposits into another on-chain program — the Staking App — using Cross-Program Invocation (CPI). This mirrors how traditional banks invest idle funds, and shows how powerful Solana’s composability can be.  

Now it’s your turn.

### 3. You Build It: Token Staking with CPI 💼
You've learned the concept of CPI and seen it in action with SOL staking — now it’s time to apply that knowledge and build something on your own.  

In this part, you’ll extend the Bank App to support SPL token investments via CPI. The goal is to mirror the same staking workflow you just learned, but with SPL tokens instead of SOL.  

This will give you real hands-on experience writing CPI integrations, managing token accounts, and building DeFi-style logic on Solana.  

🛠️ Your Tasks: 
1. **Write Tests for Existing SOL CPI Integration**  
The `invest` instruction is already implemented — your first task is to write a test that ensures it correctly stakes and unstakes SOL via the Staking App.  
Test both the stake and unstake flow carefully.  

2. **Build a Token-Based Staking Program**  
Create a new simple staking app that supports any SPL token and gives a fixed 5% APR, similar to the SOL staking version.
+ Support Stake and Unstake via one instruction
+ Use ATAs and PDAs properly for token storage
+ Handle staking rewards logic cleanly  

3. **Extend the Bank App with Token Investing**
Add a new instruction to your Bank App:
+ `InvestToken` — this allows the bank authority to invest deposited SPL tokens into your new staking program using CPI
+ This will be similar to the `invest` instruction you already saw, but for tokens instead of SOL

4. **🧪 Write Tests for Everything**
🔁 And as always — don’t forget to write tests for:
+ Your new SPL Token Staking Program
+ The `InvestToken` instruction in the Bank App

#### 🚀 Ready to Build?
This part is all about applying what you’ve learned — putting together PDAs, ATAs, CPI, and testing to build a fully working end-to-end feature.  

You’re building real DeFi patterns now — and the skills you're using here are exactly what production protocols on Solana are built with.  

Let’s see what you can create. 💪🌐























