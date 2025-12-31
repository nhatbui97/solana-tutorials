# 08 – Working with Anchor IDL
Last lesson was tuff, wasn't it? Do not worry! This one will be easy, and straight forward.


By the end of this lesson, you will:
- Understand the structure and purpose of Anchor IDLs
- Know how IDLs are generated, stored, and updated on-chain
- Find IDLs
- From IDL to create a crate for CPI.


## 1. What is an IDL?

"An Interface Description Language (IDL) file for an Anchor program provides a standardized JSON file describing the program's instructions and accounts. This file simplifies the process of integrating your on-chain program with client applications." - I took this from anchor's website.

Basically, the IDL is not your program - it's a JSON description that tells clients how to interact with your program. Think of it as your program's API documentation, but structured for machines. 

You will use this to run your program after deploying through JS/TS, or to interact with other programs.

It contains:
- **Instructions**: Function signatures, parameter types, and required accounts
- **Accounts**: Struct definitions for program-owned data
- **Types**: Custom types used across instructions
- **Events**: Emitted log data structures
- **Errors**: Custom error codes and messages


```rust
{
  "version": "0.1.0",
  "name": "bank_app",
  "instructions": [
    {
      "name": "initialize",
      "accounts": [
        { "name": "authority", "isMut": true, "isSigner": true },
        { "name": "bankInfo", "isMut": true, "isSigner": false }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "BankInfo",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "authority", "type": "publicKey" },
          { "name": "isPaused", "type": "bool" }
        ]
      }
    }
  ]
}
```

## How IDLs are generated?

When you run `anchor build` or `anchor idl build`, Anchor:
- Parses your Rust code to extract instruction signatures, account structures, and type.
- Generates a JSON IDL file at `target/idl/<program_name>.json`
- Generates TypeScript types at `target/types/<program_name>.ts (derived from IDL)`

If you run, `anchor idl init -f <target/idl/program.json> <program-id>`, Anchor will create an idl account, owned by that program itself. Remind that, this is not default - means that the IDL is not automatically uploaded onchain if you run `anchor deploy`

Most of the time, I don't see anyone using this, no one upload their program's IDL :D. In that case, how can you find IDL of a program?

## Finding IDLs
The simplest method is through onchain storage, using anchor CLI:
```rust
anchor idl fetch -p mainnet <PROGRAM_ID> // change to devnet if ur program on devnet
```
But im pretty sure most of the time you will see one of these errors:
```
# Error: IDL not found
# This means either:
# 1. The program never uploaded its IDL (old deployment)
# 2. The IDL was erased (using anchor idl erase)
# 3. The program isn't an Anchor program
```
Simply because no one init/upload the IDL onchain.

What to do? I'm pretty sure that, if you are reading this, you are working for a DeFi protocol, or self-learning to do interact with a DeFi protocol. 

In this case, the most useful way is to: Surfing the web, and most of the time you'll found what you need in the Protocol github, or through their Doc on their website (which will navigate to Gitbook/github).
Example: [Jupiter Earn](https://github.com/jup-ag/jupiter-lend/blob/main/docs/earn/cpi.md)

If you could not found the github or doc, or simply they don't have it, or not DeFi, you can use [IDL Extractor](https://github.com/dvrvsimi/solana-idl-extractor), all the guide in the repo, also the IDL is not 100% the same as the program.


## IDL Structure (>0.30)
```rust
{
  "instructions": [
    {
      "name": "deposit",
      "accounts": [
        {
          "name": "user",
          "isMut": true,
          "isSigner": true,
          "docs": ["The user depositing SOL"]
        },
        {
          "name": "bankVault",
          "isMut": true,
          "isSigner": false,
          "pda": {
            "seeds": [
              { "kind": "const", "value": [98, 97, 110, 107, 45, 118, 97, 117, 108, 116] }
            ]
          }
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ],
      "returns": null
    }
  ]
}
```

Fields:
-  `name`: Instruction name (must match Rust function)
- `accounts`: List of required accounts with mutability and signer flags
- `args`: Instruction parameters with types
- `returns`: Return type (usually null for Solana programs)
- `pda`: PDA derivation info

Accounts Section:
```rust
{
  "accounts": [
    {
      "name": "BankInfo",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "authority", "type": "publicKey" },
          { "name": "isPaused", "type": "bool" },
          { "name": "bump", "type": "u8" }
        ]
      }
    }
  ]
}
```
Types Section:
```rust
{
  "types": [
    {
      "name": "DepositEvent",
      "type": {
        "kind": "struct",
        "fields": [
          { "name": "user", "type": "publicKey" },
          { "name": "amount", "type": "u64" },
          { "name": "timestamp", "type": "i64" }
        ]
      }
    }
  ]
}
```
Errors Section:

```rust
{
  "errors": [
    {
      "code": 6000,
      "name": "Unauthorized",
      "msg": "You are not authorized to perform this action"
    },
    {
      "code": 6001,
      "name": "InsufficientFunds",
      "msg": "Insufficient funds for this operation"
    }
  ]
}
```

## IDL to Program Interface
Ever wonder that if we have an IDL, can we create the program interface, or CPI to interact with that contract? Yes, and [this github repo](https://github.com/saber-hq/anchor-gen) will help you do that.
The guide in repo is not clear enough so i will help you.
1. Use `cargo init` to create new project.
2. Add the following to a Cargo.toml file in a new crate:
```toml
[dependencies]
anchor-gen = "0.3.1"
```
3. In that project, change `main.rs` to `lib.rs`, and write: 
```rust
anchor_gen::generate_cpi_crate!("../../path/to-idl/idl.json"); 
```
4. Run `cargo build` and you got the CPI.

!!! Note that: `anchor-gen = 0.3.1` will only work with anchor under v0.29.0, and >0.30 you need to use `0.4.1` because of how IDL format change

| Anchor version | IDL account fields |
|---------------|-------------------|
| ≤ 0.29.x      | `isSigner`, `isMut` |
| ≥ 0.30.0      | `signer`, `writable`     |


And you good to go. 


Now try it yourself, using the repo i gave, and convert the idl.json to CPI.