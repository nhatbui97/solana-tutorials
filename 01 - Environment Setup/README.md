# Part One - Environment Setup
Before writing or deploying smart contracts on Solana, we need to set up a proper development environment. This part will walk you through everything you need to get started, from installing key tools to creating your first Anchor project.

### In this section, you will:
✅ Install Rust, the programming language used to write Solana programs  
✅ Install the Solana CLI, which allows you to interact with the blockchain  
✅ Install the Anchor framework, the most popular toolkit for Solana development  

By the end of this part, you’ll have everything you need to build, test, and deploy Solana smart contracts on Devnet.

Let’s get started! 🚀
### ONE COMMAND DO ALL THE INSTALL
```
curl --proto '=https' --tlsv1.2 -sSfL https://solana-install.solana.workers.dev | bash
```
A successful installation will return output like the following:
```
Installed Versions:
Rust: rustc 1.91.0 
Solana CLI: solana-cli 2.3.13 (src:5466f459; feat:2142755730, client:Agave)
Anchor CLI: 0.32.1
Node.js: v24.10.0
Yarn: 1.22.22
```
verify again by:

```
rustc --version && solana --version && anchor --version && node --version && yarn --version
```
This will install THE NEWEST VERSION, not STABLE VERSION. In order to install stable versions, paste and run these following commands:
```bash
rustup default 1.90.0
agave-install init 2.3.0
avm use 0.31.1

```
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

To ensure compatibility with the Anchor framework stable version (will be installed in the next part), we should set the Rust version to 1.90.0:
```bash
rustup default 1.90.0
```

### 2. Install The Solana CLI 

To interact with the Solana blockchain, you need to install the Solana Command Line Interface (CLI). The Solana CLI provides commands for creating wallets, deploying programs, and send transactions.

Run the following command to download and install the Solana CLI:
```bash
sh -c "$(curl -sSfL https://release.anza.xyz/v2.3.0/install)"
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

Most major Solana protocols (as of 14 May 2025) - such as Jito, Jupiter, Raydium, Orca,... - still use Anchor 0.29.0 as their stable release.
Update, this is outdated (11 Nov 25), we need to use v0.30 and above, the most stable is 0.31.1.
```bash
avm use 0.31.1
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






# For one who use Windows to run this
Im someone who likes windows, like alot, i did use Linux couple of years but always got somthing error. So i wrote this down to help some one has trouble while setup Solana on Windows.
If you doesn't like to run wsl on Windows, and run Rust directly, i wouldn't recommend it, tho i did run it flawlessly 1.5 years with chocolatey (im running cosmwasm, not solana), dyor for this. But i suggest you do wsl setup, much easier.

https://solana.com/docs/intro/installation - Here's the official guide for you, but i will tldr this step down below
1. First you need to install WSL2 (WSL) on Windows. This in order to help you run Linux terminal directly on windows.
2. Second, access Windows Store on your Windows 10/11, search "Ubuntu". You can install whatever distro (version) you want but i recommend get the one has the highest review score.
3. After that, open windows search, search "Ubuntu", terminal will popup, you type username, password etc... ready to go!



## Some error i faced myself, or just notes
- If your C drive (where you windows install) too small, not enough space or something else, you might want to change location where the distro (ubuntu) install, do this 

1. Just click Ubuntu 22.04 install button in Microsoft Store App(which only download appx package, installation will be triggered when we click Ubuntu in StartMenu first time).

2. Then search `install.tar.gz` via Everything app, suprise are as follows: Expand-appx from Macrosoft Store

3. Copy all the files to `D:\WSL\appx` or where you like, click <distribution>.exe to install, then a ext4.vhdx file will be created like follows:after install

4. Finally trigger uninstall from StartMenu which delete the packages in drive-C.

contact me if this unclear.


- Access your local (in Windows directory):
  - use cd mnt/<what_drive_you_want_to_access>. Example drive c: cd mnt/c/, ls to see list of all folder in that
  - use symlink to easy link a "shortcut" to your work folder, example:  ln -s <path_to_where_the_folder_you_want_to_link> ./Work, this will create a symlink at the current folder where the terminal at as name "Work"
- linker "cc" not found when install whatever in the setup step - as for me when install avm
```
sudo apt-get update
sudo apt install build-essential
```
then rerun the error command



- If you run into error: version GLIBC_2.39' not found (required by /home/bill/.avm/bin/anchor-0.32.1) or similar
which mean your distro outdated, which use older glibc version. It can be handled by: Reinstall the lastest distro. You can do this by install thru windows store by downloading another distro version. But for me im lazy, when already install bunch of packages. So do this if you're in same shoes as me:
```
sudo apt update && sudo apt upgrade -y
sudo apt install update-manager-core -y
sudo do-release-upgrade
```
this three commands will update to the latest ubuntu version (for me 24.04)
then you free to run without error.

1. If you run into error "Checking for a new Ubuntu release In /etc/update-manager/release-upgrades Prompt is set to never so upgrading is not possible." then:
```
sudo nano /etc/update-manager/release-upgrades
```
2. find : Prompt=never change it to Prompt=normal then you can install new version


DO THIS IN WINDOWS TERMINAL, NOT VSCODE TERMINAL



- Right now solana-cli version above 0.30 is recommended, 0.31.0 specifically, install 0.29.0 is a bit tricky, and messy. If forced, can still do this but contact me.
- When running `anchor build` with lower solana version like 0.29, this problem can occured: "rustc v1.75 or above ... bla bla bla" but when you check your rustc version, its 1.83, 1.90, so why its still error? it is because the solana-install, anchor use it own rustc version and it is outdated.  If this happen then you must install the agave-install 
follow this instruction https://docs.anza.xyz/cli/install, change v3.0.8 to v2.3.0 - the most stable version. Then proceed to anchor build, again.

If you run into build-sbf not found, the most likely you have not listened to me - and install both v0.29.0 and v0.30, then uninstall v0.29.0, or check agave-install/solana-install whether if it 1.18 or above v2, if it v2 then your solana-cli outdated, if v1.18 or under v2, then update to above v2. your problem will be solved. If not then contact me.

If you must stay on Anchor 0.29, lock Solana back to 1.18.13 and Rust 1.73-1.78 and `cargo install --git https://github.com/coral-xyz/anchor --tag v0.29.0 avm --locked`