# Part Two - Deploy, Test, Upgrade, and Close

In the previous section, we have initialized my-first-anchor-project. In this part, we will learn to deploy this program to devnet, write test cases, upgrade the program and then close it to reclaim the SOL staked in the buffer account.

### 1. Deploy

After initialization, Anchor has already created an example program for us:
```rust
use anchor_lang::prelude::*;

declare_id!("GDGNBNAhHGmMKcxVxXBTTJ8xytmdjNuFWsr2igqhck27");

#[program]
pub mod my_first_anchor_project {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
```

Let's break this down.
The first line you’ll notice is the program's declared ID:
```rust
declare_id!("GDGNBNAhHGmMKcxVxXBTTJ8xytmdjNuFWsr2igqhck27");
```


This is the **program ID** (like a contract address) that will be used after deployment. The actual ID is determined by the keypair located at:
```
my-first-anchor-project/target/deploy/my_first_anchor_project-keypair.json
```
Your program ID will likely be different from the one shown above, and you can generate a new random keypair if needed.

Next, we have the main part of the program:
```rust
#[program]
pub mod my_first_anchor_project {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}
```
This example is very simple. It defines just one method called `initialize`, which currently doesn’t include any logic—it simply returns `Ok(())` when invoked.

Now, to deploy the program to devnet, you need to build it first.
Run the following command:

```bash
anchor build
```

The output should look something like this:

```bash
warning: unused variable: `ctx`
 --> programs/my-first-anchor-project/src/lib.rs:9:23
  |
9 |     pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
  |                       ^^^ help: if this is intentional, prefix it with an underscore: `_ctx`
  |
  = note: `#[warn(unused_variables)]` on by default

warning: `my-first-anchor-project` (lib) generated 1 warning (run `cargo fix --lib -p my-first-anchor-project` to apply 1 suggestion)
    Finished release [optimized] target(s) in 37.40s
```
You can safely ignore the warning for now.
After the build completes, you’ll find a `.so` file generated at:

```
my-first-anchor-project/target/deploy/my_first_anchor_project.so
```

This `.so` file is the compiled version of your program and will be used to deploy to the Solana Devnet.
Additionally, the IDL and TypeScript types are also generated at:
```
target/idl/my_first_anchor_project.json
target/types/my_first_anchor_project.ts
```
We’ll leave these files as they are for now and revisit them in the testing section.

Now you're ready to deploy the program! Run the following command:
```bash
solana program deploy target/deploy/my_first_anchor_project.so --program-id target/deploy/my_first_anchor_project-keypair.json
```

You should see output similar to this:
```bash
Program Id: GDGNBNAhHGmMKcxVxXBTTJ8xytmdjNuFWsr2igqhck27
```

🎉 **Congratulations!** You've successfully deployed your first Solana program to the Devnet.

You can also configure your `Anchor.toml` to specify the devnet cluster:
```
cluster = "Devnet"
```
Then, deploy using the Anchor CLI:
```bash
anchor deploy
```

This is convenient for local and devnet development and testing, but **not recommended for mainnet** deployments.


---

##### ⚠️ Why Not Use `anchor deploy` on Mainnet?

On mainnet, `anchor deploy` can often fail due to RPC reliability issues. Instead, it's better to use the `solana program deploy` command with a specific RPC provider.

For example, using the `--use-rpc` flag with a private, high-quality RPC endpoint:

```bash
solana program deploy target/deploy/my_first_anchor_project.so --program-id target/deploy/my_first_anchor_project-keypair.json --use-rpc
```

### 2. Test




