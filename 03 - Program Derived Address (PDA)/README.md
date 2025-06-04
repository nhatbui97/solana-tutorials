# Part Three - Program Derived Address (PDA)

Now that you’re comfortable writing basic Solana programs, it’s time to introduce one of the most important concepts in Solana development — Program Derived Addresses (PDAs). These special accounts are the key to building secure, stateful programs that can store user data, manage vaults, control authorities, and more.  

### In this section, you will:
✅ Understand what PDAs are and how they work  
✅ Initialize accounts using PDAs with seeds and bump  
✅ Learn how to derive PDAs in Anchor TS Client.  
✅ Complete the first real-world example: the Bank App  

By the end of this part, you’ll be able to confidently create and manage PDA accounts in your Solana programs, unlocking the ability to build more powerful and complex smart contracts.  
Let’s dive in! 🧠✨

### Let's start with a real-world example: the Bank App 🏦
To understand how PDAs work in practice, let’s look at a simple banking program on Solana.  
In this app:

👤 Users can deposit and withdraw SOL  
🛑 An authority can pause the program to stop all activity during an emergency  
💾 The program should store:
- Global state in a special PDA account called `BankInfo`
- Each user’s deposited balance in individual PDA accounts called `UserReserve`


### 1. What is a PDA?
A Program Derived Address (PDA) is a special type of Solana account that is owned by a program, not by an on-curve wallet (user wallet) with a private key. This makes PDAs the backbone of most Solana smart contracts — they allow your program to safely manage state, assets, and authority without depending on externally owned wallets.  

PDAs are:  
🔐 *Controlled by your program* — no private key, only the program can access its PDAs, and no one can forge its signature.  
🧠 *Deterministic* — they’re generated using fixed inputs (called `seeds`) plus your program ID  
✍️ *Able to sign transactions* — but only by using `invoke_signed()` with the PDA's `seeds` inside your program  

In our bank program, we use two PDAs to store data in `state.rs`:
```rust
#[account]
#[derive(Default)]
pub struct BankInfo {
    pub authority: Pubkey,
    pub is_paused: bool,
    pub bump: u8,
}

#[account]
#[derive(Default)]
pub struct UserReserve {
    pub deposited_amount: u64,
}
```

- `BankInfo` is a global PDA that stores the program’s state: who the `authority` is, whether the program `is_paused`, and the Bank Vault's `bump` value.
- `UserReserve` is a user-specific PDA that tracks how much SOL each user has deposited.

These PDAs are derived using seeds and something called a bump. But what exactly is a bump — and why do we store it?

When generating a PDA, Solana requires that the derived address does not lie on the ed25519 curve (since otherwise someone could potentially find a private key for it). However, not every seed combination produces a valid off-curve address.  

To fix this, they add a small number — the bump (an 8-bit unsigned integer from 0–255) — which is adjusted automatically during PDA creation to ensure a valid address. Anchor handles the bump calculation automatically when you initialize the PDA. But if your program needs to regenerate or sign on behalf of that PDA, you must store the bump so you can reproduce the seeds or the exact address.  

👉 In our example, we store the bump in `BankInfo` because the program will need the Bank Vault PDA to sign instructions later.

### 2. Initializing a PDA
Now that we understand what a PDA is, let’s walk through how to create and initialize a PDA in Anchor.  
In our bank app, we initialize the global `BankInfo` PDA when the program is first set up. Here’s how it looks in `instructions/initialize.rs`:
```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [BANK_INFO_SEED],
        bump,
        payer = authority,
        space = 8 + std::mem::size_of::<BankInfo>(),
    )]
    pub bank_info: Box<Account<'info, BankInfo>>,
}
```

##### 🧪Let’s break it down:
- `init`: Tells Anchor to create a new PDA account. You can only initialize a PDA once — if it already exists, the transaction will fail and revert. If you ever need to reuse the same PDA address, you'll first have to close the existing account.
- `seeds` = [...]: These are the values used to deterministically derive the PDA address. You can include multiple seed values depending on your use case. In this example, we’re initializing a single global state account, so we only use one static seed: `BANK_INFO_SEED`.
- `bump`: Instructs Anchor to automatically calculate a valid bump value for this seed combination (we already covered bumps in the previous section 😄).
- `payer`: Creating a PDA requires storage space, and on Solana, storage comes with rent costs. The payer field specifies which signer will cover the cost of account creation — in this case, the `authority`.
- `space`: How much space (in bytes) to allocate for the account. The more space the PDA needs, the more cost payer have to pay.  

The Bank Vault is initialized right after the `BankInfo` account, but there are a few key differences worth noting:
```rust
    #[account(
        init,
        seeds = [BANK_VAULT_SEED],
        bump,
        payer = authority,
        space = 0,
        owner = system_program::ID
    )]
    pub bank_vault: UncheckedAccount<'info>,
```
##### 🧩 What’s different here?
+ *No data storage*: Unlike `BankInfo`, this PDA doesn’t store any data — hence `space = 0`, and we don’t define a struct for it.
+ *System-owned*: The account is created with `owner = system_program::ID`, meaning it’s owned by the System Program, not your Anchor program. This might seem unusual at first, but it's intentional.
+ *Why create this PDA?*  
This vault acts as a centralized fund holder for your entire app. Since it's a PDA derived using your program ID and a known seed, your program can still sign for it and control its SOL balance.

**⚠️ Important Note**: The reason we use a System Program-owned PDA is because only accounts owned by the System Program can participate in native SOL transfers. When transferring SOL using the transfer instruction, both the sender and receiver must be system-owned accounts. That’s why we structure the vault this way — to serve as a secure, program-controlled SOL pool that users can send funds to or withdraw from. We’ll dive deeper into how this works when we implement the actual SOL transfer logic in the next section.  

Now that both PDAs are created, let’s move on to the process function where we initialize the fields of our `BankInfo` account:
```rust
pub fn process(ctx: Context<Initialize>) -> Result<()> {
    let bank_info = &mut ctx.accounts.bank_info;

    bank_info.authority = ctx.accounts.authority.key();
    bank_info.is_paused = false;
    bank_info.bump = ctx.bumps.bank_vault;

    msg!("Bank initialized!");
    Ok(())
}
```
Here we’re:
- Saving the authority’s public key
- Setting is_paused to false by default
- Storing the bump value for future PDA sign and re-derivation

That wraps up the `Initialize` instruction.  

Now, let’s take a look at how we create user-specific PDA accounts — specifically the `UserReserve` — which is handled in the `instructions/deposit.rs` file:
```rust
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        init_if_needed,
        seeds = [USER_RESERVE_SEED, user.key().as_ref()],
        bump,
        payer = user,
        space = 8 + std::mem::size_of::<UserReserve>(),
    )]
    pub user_reserve: Box<Account<'info, UserReserve>>,
}
```
At first glance, this looks similar to how we initialized `BankInfo` right? But there are some key differences:
- `init_if_needed`: This directive checks if the PDA already exists. If it doesn’t, Anchor will automatically create it; if it does, the existing PDA will be loaded mutably. This is perfect for an instruction like `Deposit`, which might be called many times by the same user - no need to write extra logic to check if the account exists before using it.
- `seeds`: This time, we’re using two seeds - a constant seed `USER_RESERVE_SEED` and the user’s public key `user.key().as_ref()` (converted to `&[u8]`). This pattern ensures that each user gets their own unique PDA — so no two users share the same UserReserve account. It also means each user can only have one UserReserve PDA derived in this way, which helps with consistency and security.

Then, we handle the deposit logic in the `process` function like this:
```rust
pub fn process(ctx: Context<Deposit>, deposit_amount: u64) -> Result<()> {
    if ctx.accounts.bank_info.is_paused {
        return Err(BankAppError::BankAppPaused.into());
    }

    let user_reserve = &mut ctx.accounts.user_reserve;

    sol_transfer_from_user(
        &ctx.accounts.user,
        ctx.accounts.bank_info.to_account_info(),
        &ctx.accounts.system_program,
        deposit_amount,
    )?;

    user_reserve.deposited_amount += deposit_amount;

    Ok(())
}
```

In this function, before allowing any deposits, the program first checks the status of `BankInfo`:
```rust
if ctx.accounts.bank_info.is_paused {
    return Err(BankAppError::BankAppPaused.into());
}
```
If the bank is paused (perhaps due to an emergency or upgrade), the transaction is rejected with an appropriate error.  

We then transfer SOL from the user to the `BankInfo` PDA — which acts as a global vault that holds all deposited funds.  
The actual transfer is handled using a helper function defined in `transfer_helper.rs`:
```rust
//  transfer SOL from user
pub fn sol_transfer_from_user<'info>(
    signer: &Signer<'info>,
    destination: AccountInfo<'info>,
    system_program: &Program<'info, System>,
    amount: u64,
) -> Result<()> {
    let ix = transfer(signer.key, destination.key, amount);
    invoke(
        &ix,
        &[
            signer.to_account_info(),
            destination,
            system_program.to_account_info(),
        ],
    )?;
    Ok(())
}
```
Since the user is the signer in this case, we can simply use `invoke()` to perform the transfer.  
Later, when we implement withdrawals, the program will need to sign on behalf of the Bank Vault PDA — and for that, we’ll use `invoke_signed()`.  

Finally, we update the user’s UserReserve PDA to reflect the new deposited amount:
```rust
user_reserve.deposited_amount += deposit_amount;
```
Now that you know how to create, initialize, and interact with PDAs inside the program, let’s move on to the client side.  
➡️ In the next part, we’ll learn how to derive PDA addresses from the Anchor TypeScript client so we can call these instructions properly from the frontend or scripts.

### 3. Derive PDA on the Client
To interact with your program from the frontend or scripts (like calling `initialize` or `deposit`), you’ll need to derive the same PDA addresses that the program expects. Luckily, Anchor makes this easy on the TypeScript client side.  

Let’s see how to do it using the same logic we used in the program.  
PDA addresses are derived using this formula:
```ts
PublicKey.findProgramAddressSync([SEEDS], PROGRAM_ID)
```
- `SEEDS` is an array of bytes (Buffer) that must match exactly what the program uses.
- `PROGRAM_ID` is the ID of your deployed program.

In our bank-app, We derive two PDAs.  
Here’s how they’re defined in `tests/bank-app.ts`:
```ts
const BANK_APP_ACCOUNTS = {
    bankInfo: PublicKey.findProgramAddressSync(
        [Buffer.from("BANK_INFO_SEED")],
        program.programId
    )[0],
    bankVault: PublicKey.findProgramAddressSync(
        [Buffer.from("BANK_VAULT_SEED")],
        program.programId
    )[0],
    userReserve: (pubkey: PublicKey) => PublicKey.findProgramAddressSync(
        [
            Buffer.from("USER_RESERVE_SEED"),
            pubkey.toBuffer()
        ],
    program.programId
    )[0],
  }
```
Notice that `userReserve` is a function. This lets you dynamically generate a unique PDA for each user based on their public key.  
By deriving PDAs this way, you ensure your client always uses the correct accounts — exactly how your program expects them.

### 4. Time to Build 💪 (Your Turn!)
Now that you understand how to create and use PDAs, it’s your turn to put it into practice.  

🛠️ Your Tasks: 
1. **Implement `sol_transfer_from_pda` in `transfer_helper.rs`**  
This function should transfer SOL from a PDA (like BankInfo) back to a user.  
Since a PDA can’t sign on its own, you’ll need to use `invoke_signed()` and pass the correct `signers_seeds`

2. **Complete the `Withdraw` Instruction**  
Allow users to withdraw their deposited SOL from the vault (i.e., from the Bank Vault PDA).  
We’ve already provided the PDA seeds for this instruction — just plug them in to use `invoke_signed()` properly.

3. **Implement the `Pause` Instruction**  
Add logic to pause or unpause the app. Only the authority defined in BankInfo should be able to do this.  
💡 Hint: Use Anchor's `#[account(address = ...)]` to restrict access.

4. **Don't forget to write Tests in `bank-app.ts`**  
Create tests for your new `Withdraw` and `Pause` instructions.  
Be sure to:
- Withdraw the correct amount and verify the updated `UserReserve`.
- Test pausing and unpausing the app and ensure deposits/withdrawals are blocked when paused.  

Once you've completed these tasks, you’ll have hands-on experience managing PDA authority, securing instructions, and signing on behalf of a PDA — essential building blocks for any serious Solana developer.

🚀 Let’s get building!














