# Part One - Environment Setup
Before writing or deploying smart contracts on Solana, we need to set up a proper development environment. This part will walk you through everything you need to get started, from installing key tools to creating your first Anchor project.

### In this section, you will:
✅ Install Rust, the programming language used to write Solana programs  
✅ Install the Solana CLI, which allows you to interact with the blockchain  
✅ Install the Anchor framework, the most popular toolkit for Solana development  

By the end of this part, you’ll have everything you need to build, test, and deploy Solana smart contracts on Devnet.

Let’s get started! 🚀

### 1. Install Rust

Run the following command to install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

After installation, you'll need to reload your PATH environment variable to include Cargo’s bin directory
Run the following command:
```bash
. "$HOME/.cargo/env"
```
This ensures that the cargo and rustc commands are available globally in your terminal session.

Then, check if Rust has been successfully installed:
```bash
rustc --version
```

To ensure compatibility with the Anchor framework stable version (will be installed in the next part), we should set the Rust version to 1.83.0:
```bash
rustup default 1.83.0
```

### 2. Install The Solana CLI 

To interact with the Solana blockchain, you need to install the Solana Command Line Interface (CLI). The Solana CLI provides commands for creating wallets, deploying programs, and send transactions.

Run the following command to download and install the Solana CLI:
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/v1.18.21/install)"
```

After installation, update your environment so the `solana` command is available:
```bash
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
```

Check the version to confirm everything is set up correctly:
```bash
solana --version
```

Now the Solana CLI has successfully installed, you can create your first wallet by:
```bash
solana-keygen new 
```
You should see output like the following:
```bash
Wrote new keypair to /Users/nhatbui97/.config/solana/id.json
====================================================================
pubkey: jixspQw81GQVo969PPNeK7WteDhvWVFWhcLfLoMiPo2
====================================================================
Save this seed phrase and your BIP39 passphrase to recover your new keypair:
cloud taxi flash truth rug pill bronze duck bread month patch behave
====================================================================
```
⚠️ Important: Store your seed phrase securely. Anyone with access to it can control your funds.


Then switch RPC URL to devnet and get you some SOL for transaction fee:
```bash
solana config set -u https://api.devnet.solana.com 
solana airdrop 5
```

For easier access and UI interaction, you can import your wallet into [Phantom Wallet](https://phantom.com/download)
Run:
```bash
cat $HOME/.config/solana/id.json
```

This prints an array of numbers like:
```bash
[25,250,185,230,65,229,210,243,20,209,26,80,240,226,48,97,145,15,119,43,132,245,62,210,12,180,144,72,190,100,81,104,10,241,215,149,189,41,158,148,184,110,49,69,150,197,128,112,249,223,130,24,115,123,92,77,83,180,100,176,19,136,114,173]
```

Import this to Phantom Wallet and turn on Testnet Mode:
<p float="left">
  <img src="../Example Images/01-ImportPhantom1.png" alt="Step 1" width="240" height="400" style="margin-right: 10px;"/>
  <img src="../Example Images/01-ImportPhantom2.png" alt="Step 2" width="240" height="400" style="margin-right: 10px;"/>
  <img src="../Example Images/01-ImportPhantom3.png" alt="Step 3" width="240" height="400" style="margin-right: 10px;"/>
  <img src="../Example Images/01-ImportPhantom4.png" alt="Step 4" width="240" height="400"/>
</p>

### 3. Install Anchor CLI

Anchor is a framework for developing Solana programs. The Anchor framework leverages Rust macros to simplify the process of writing Solana programs.
The Anchor version manager (AVM) allows you to install and manage different Anchor versions on your system and easily update Anchor versions in the future.

Install AVM with the following command:
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --force
```

Confirm that AVM installed successfully:
```bash
avm --version
```

Most major Solana protocols (as of 14 May 2025) - such as Jito, Jupiter, Raydium, Orca,... - still use Anchor 0.29.0 as their stable release. Therefore, for compatibility and ease of integration in the future, we'll use this version:
```bash
avm use 0.29.0
```

Confirm your Anchor version:
```bash
anchor --version
```
Congratulations! You've successfully installed the Anchor framework.
You can now initialize your first Anchor project by running:
```bash
anchor init my-first-anchor-project
```

Once complete the output should look something like the following:
```bash
yarn install v1.22.22
warning package.json: No license field
info No lockfile found.
warning No license field
[1/4] 🔍  Resolving packages...
warning mocha > glob@7.2.0: Glob versions prior to v9 are no longer supported
warning mocha > glob > inflight@1.0.6: This module is not supported, and leaks memory. Do not use it. Check out lru-cache if you want a good and tested way to coalesce async requests by a key value, which is much more comprehensive and powerful.
[2/4] 🚚  Fetching packages...
[3/4] 🔗  Linking dependencies...
warning "@coral-xyz/anchor > @solana/web3.js > @solana/codecs-numbers@2.1.1" has incorrect peer dependency "typescript@>=5.3.3".
warning "@coral-xyz/anchor > @solana/web3.js > @solana/codecs-numbers > @solana/errors@2.1.1" has incorrect peer dependency "typescript@>=5.3.3".
warning "@coral-xyz/anchor > @solana/web3.js > @solana/codecs-numbers > @solana/codecs-core@2.1.1" has incorrect peer dependency "typescript@>=5.3.3".
[4/4] 🔨  Building fresh packages...
success Saved lockfile.
✨  Done in 8.05s.
Initialized empty Git repository in /Users/nhatbui97/Documents/Solana Program/solana-tutorials/01 - Environment Setup/my-first-anchor-project/.git/
my-first-anchor-project initialized
```
You're now ready to start building on Solana with Anchor!

