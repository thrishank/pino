use crate::commands::init::InitConfig;
use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

fn project_path(config: &InitConfig, rel: &str) -> PathBuf {
    PathBuf::from(&config.name).join(rel)
}

pub fn create_project_dir(name: &str) -> Result<()> {
    let path = PathBuf::from(name);
    if path.exists() {
        anyhow::bail!("directory '{}' already exists", name);
    }
    fs::create_dir_all(path.join("src/instructions"))?;
    fs::create_dir_all(path.join("tests"))?;
    Ok(())
}

pub fn write_cargo_toml(config: &InitConfig) -> Result<()> {
    let mut deps = String::from(
        r#"pinocchio = "0.6"
pinocchio-log = "0.3"
pinocchio-pubkey = "0.2"
pinocchio-system = "0.2"
"#,
    );

    if config.token {
        deps.push_str(
            r#"pinocchio-token = "0.4"
pinocchio-associated-token = "0.1"
"#,
        );
    }

    if config.anchor {
        deps.push_str(r#"anchor-lang = { version = "0.30", features = ["no-log-cpi-calls"] }
"#);
    }

    let content = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "lib"]
name = "{name_snake}"

[features]
default = []
test-sbf = []

[dependencies]
{deps}
[dev-dependencies]
solana-program-test = "1.18"
solana-sdk = "1.18"
tokio = {{ version = "1", features = ["full"] }}
"#,
        name = config.name,
        name_snake = config.name.replace('-', "_"),
        deps = deps,
    );

    fs::write(project_path(config, "Cargo.toml"), content)
        .context("writing Cargo.toml")?;
    Ok(())
}

pub fn write_lib_rs(config: &InitConfig) -> Result<()> {
    let anchor_attr = if config.anchor {
        "#[cfg(feature = \"anchor\")]\nuse anchor_lang::prelude::*;\n\n"
    } else {
        ""
    };

    let content = format!(
        r#"//! {name} — Pinocchio Solana Program
{anchor}
use pinocchio::{{
    account_info::AccountInfo,
    entrypoint,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
}};

mod errors;
mod instructions;
mod processor;
mod state;

use instructions::ProgramInstruction;

// Register the program entrypoint
entrypoint!(process_instruction);

/// Program entrypoint. Pinocchio calls this for every transaction
/// targeting this program ID.
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {{
    let ix = ProgramInstruction::from_bytes(instruction_data)?;
    processor::process(program_id, accounts, ix)
}}
"#,
        name = config.name,
        anchor = anchor_attr,
    );

    fs::write(project_path(config, "src/lib.rs"), content)?;
    Ok(())
}

pub fn write_instruction_rs(config: &InitConfig) -> Result<()> {
    let content = r#"//! Instruction deserialization — zero-copy, no heap allocation.
use core::mem::size_of;
use pinocchio::program_error::ProgramError;

/// Discriminators for each instruction variant.
/// First byte of instruction_data selects the handler.
#[repr(u8)]
pub enum InstructionDiscriminator {
    Initialize = 0,
    Execute    = 1,
    Close      = 2,
}

// ─── Initialize ────────────────────────────────────────────────────────────

#[repr(C)]
pub struct InitializeArgs {
    pub bump: u8,
    pub amount: u64,
}

impl InitializeArgs {
    pub fn from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        // SAFETY: repr(C), length checked above, alignment guaranteed by Solana BPF ABI
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }
}

// ─── Execute ───────────────────────────────────────────────────────────────

#[repr(C)]
pub struct ExecuteArgs {
    pub amount: u64,
}

impl ExecuteArgs {
    pub fn from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < size_of::<Self>() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }
}

// ─── Router ────────────────────────────────────────────────────────────────

pub enum ProgramInstruction<'a> {
    Initialize(&'a InitializeArgs),
    Execute(&'a ExecuteArgs),
    Close,
}

impl<'a> ProgramInstruction<'a> {
    /// Parse the raw `instruction_data` slice from the entrypoint.
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, ProgramError> {
        let (&disc, rest) = data.split_first().ok_or(ProgramError::InvalidInstructionData)?;

        match disc {
            0 => Ok(Self::Initialize(InitializeArgs::from_bytes(rest)?)),
            1 => Ok(Self::Execute(ExecuteArgs::from_bytes(rest)?)),
            2 => Ok(Self::Close),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}
"#;

    fs::write(project_path(config, "src/instructions.rs"), content)?;
    Ok(())
}

pub fn write_processor_rs(config: &InitConfig) -> Result<()> {
    let content = format!(
        r#"//! Instruction processors for {name}.
use pinocchio::{{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
    ProgramResult,
}};
use pinocchio_log::log;

use crate::instructions::{{ExecuteArgs, InitializeArgs, ProgramInstruction}};

pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: ProgramInstruction,
) -> ProgramResult {{
    match instruction {{
        ProgramInstruction::Initialize(args) => process_initialize(accounts, args),
        ProgramInstruction::Execute(args)    => process_execute(accounts, args),
        ProgramInstruction::Close            => process_close(accounts),
    }}
}}

fn process_initialize(accounts: &[AccountInfo], args: &InitializeArgs) -> ProgramResult {{
    let [_payer, _state_account, _system_program] = accounts else {{
        return Err(ProgramError::NotEnoughAccountKeys);
    }};

    log!("initialize: amount={{}}", args.amount);

    // TODO: create state account, validate signers, write initial data
    Ok(())
}}

fn process_execute(accounts: &[AccountInfo], args: &ExecuteArgs) -> ProgramResult {{
    let [_authority, _state_account] = accounts else {{
        return Err(ProgramError::NotEnoughAccountKeys);
    }};

    log!("execute: amount={{}}", args.amount);

    // TODO: load state, apply business logic, persist
    Ok(())
}}

fn process_close(accounts: &[AccountInfo]) -> ProgramResult {{
    let [_authority, _state_account, _destination] = accounts else {{
        return Err(ProgramError::NotEnoughAccountKeys);
    }};

    log!("close");

    // TODO: zero out account, transfer lamports, reassign owner to system
    Ok(())
}}
"#,
        name = config.name,
    );

    fs::write(project_path(config, "src/processor.rs"), content)?;
    Ok(())
}

pub fn write_errors_rs_with_config(config: &InitConfig) -> Result<()> {
    let content = r#"//! Custom program errors.
use pinocchio::program_error::ProgramError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ProgramError {
    InvalidAuthority  = 6000,
    AccountAlreadyInitialized,
    ArithmeticOverflow,
    InvalidAmount,
}

impl From<CustomError> for ProgramError {
    fn from(e: CustomError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

pub type CustomError = self::ProgramError;
"#;

    fs::write(project_path(config, "src/errors.rs"), content)?;
    Ok(())
}

pub fn write_state_rs_with_config(config: &InitConfig) -> Result<()> {
    let content = r#"//! On-chain account state layouts.
//! Use repr(C) for deterministic layout; document every field.

/// Discriminator tag written into byte 0 of every state account.
#[repr(u8)]
pub enum AccountTag {
    Uninitialized = 0,
    State         = 1,
}

/// Primary program state account.
/// Total size: 1 + 32 + 8 + 1 = 42 bytes
#[repr(C)]
pub struct State {
    /// Account discriminator — must equal AccountTag::State.
    pub tag: u8,
    /// Authority allowed to mutate this account.
    pub authority: [u8; 32],
    /// Tracked value.
    pub amount: u64,
    /// PDA bump for this account.
    pub bump: u8,
}

impl State {
    pub const SIZE: usize = core::mem::size_of::<Self>();
    pub const TAG: u8 = AccountTag::State as u8;

    /// Borrow state from a raw account data slice (zero-copy).
    pub fn from_bytes(data: &[u8]) -> Result<&Self, pinocchio::program_error::ProgramError> {
        if data.len() < Self::SIZE {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        if data[0] != Self::TAG {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        // SAFETY: repr(C), size checked, BPF alignment guaranteed
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }

    /// Mutable borrow for writes.
    pub fn from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, pinocchio::program_error::ProgramError> {
        if data.len() < Self::SIZE {
            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
        }
        Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
    }
}
"#;

    fs::write(project_path(config, "src/state.rs"), content)?;
    Ok(())
}

pub fn write_idl(config: &InitConfig) -> Result<()> {
    let idl = serde_json::json!({
        "version": "0.1.0",
        "name": config.name,
        "instructions": [
            {
                "name": "initialize",
                "discriminant": 0,
                "accounts": [
                    { "name": "payer", "isMut": true, "isSigner": true },
                    { "name": "stateAccount", "isMut": true, "isSigner": false },
                    { "name": "systemProgram", "isMut": false, "isSigner": false }
                ],
                "args": [
                    { "name": "bump", "type": "u8" },
                    { "name": "amount", "type": "u64" }
                ]
            },
            {
                "name": "execute",
                "discriminant": 1,
                "accounts": [
                    { "name": "authority", "isMut": true, "isSigner": true },
                    { "name": "stateAccount", "isMut": true, "isSigner": false }
                ],
                "args": [
                    { "name": "amount", "type": "u64" }
                ]
            },
            {
                "name": "close",
                "discriminant": 2,
                "accounts": [
                    { "name": "authority", "isMut": true, "isSigner": true },
                    { "name": "stateAccount", "isMut": true, "isSigner": false },
                    { "name": "destination", "isMut": true, "isSigner": false }
                ],
                "args": []
            }
        ],
        "accounts": [
            {
                "name": "State",
                "type": {
                    "kind": "struct",
                    "fields": [
                        { "name": "tag", "type": "u8" },
                        { "name": "authority", "type": { "array": ["u8", 32] } },
                        { "name": "amount", "type": "u64" },
                        { "name": "bump", "type": "u8" }
                    ]
                }
            }
        ],
        "errors": [
            { "code": 6000, "name": "InvalidAuthority" },
            { "code": 6001, "name": "AccountAlreadyInitialized" },
            { "code": 6002, "name": "ArithmeticOverflow" },
            { "code": 6003, "name": "InvalidAmount" }
        ]
    });

    let idl_dir = project_path(config, "idl");
    fs::create_dir_all(&idl_dir)?;
    fs::write(
        idl_dir.join(format!("{}.json", config.name)),
        serde_json::to_string_pretty(&idl)?,
    )?;
    Ok(())
}

pub fn write_ts_types(config: &InitConfig) -> Result<()> {
    let name_pascal = to_pascal(&config.name);
    let name_snake = config.name.replace('-', "_");

    let content = format!(
        r#"/**
 * Auto-generated TypeScript types for {name}
 * Compatible with @coral-xyz/anchor IDL format
 */
import {{ PublicKey }} from "@solana/web3.js";
import BN from "bn.js";

// ─── Account Types ──────────────────────────────────────────────────────────

export interface {pascal}State {{
  tag: number;
  authority: PublicKey;
  amount: BN;
  bump: number;
}}

// ─── Instruction Builders ───────────────────────────────────────────────────

export function buildInitializeIx(
  payer: PublicKey,
  stateAccount: PublicKey,
  programId: PublicKey,
  args: {{ bump: number; amount: BN }}
) {{
  const data = Buffer.alloc(10); // 1 discriminator + 1 bump + 8 amount
  data.writeUInt8(0, 0);         // discriminator = Initialize
  data.writeUInt8(args.bump, 1);
  args.amount.toBuffer("le", 8).copy(data, 2);

  return {{
    programId,
    keys: [
      {{ pubkey: payer,          isSigner: true,  isWritable: true }},
      {{ pubkey: stateAccount,   isSigner: false, isWritable: true }},
      {{ pubkey: PublicKey.default, isSigner: false, isWritable: false }}, // system
    ],
    data,
  }};
}}

export function buildExecuteIx(
  authority: PublicKey,
  stateAccount: PublicKey,
  programId: PublicKey,
  args: {{ amount: BN }}
) {{
  const data = Buffer.alloc(9);  // 1 + 8
  data.writeUInt8(1, 0);         // discriminator = Execute
  args.amount.toBuffer("le", 8).copy(data, 1);

  return {{
    programId,
    keys: [
      {{ pubkey: authority,    isSigner: true,  isWritable: true }},
      {{ pubkey: stateAccount, isSigner: false, isWritable: true }},
    ],
    data,
  }};
}}

export function buildCloseIx(
  authority: PublicKey,
  stateAccount: PublicKey,
  destination: PublicKey,
  programId: PublicKey,
) {{
  const data = Buffer.alloc(1);
  data.writeUInt8(2, 0); // discriminator = Close

  return {{
    programId,
    keys: [
      {{ pubkey: authority,    isSigner: true,  isWritable: true }},
      {{ pubkey: stateAccount, isSigner: false, isWritable: true }},
      {{ pubkey: destination,  isSigner: false, isWritable: true }},
    ],
    data,
  }};
}}

// ─── Account Decoder ────────────────────────────────────────────────────────

export function decode{pascal}State(data: Buffer): {pascal}State {{
  if (data[0] !== 1) throw new Error("invalid account tag");
  return {{
    tag: data[0],
    authority: new PublicKey(data.subarray(1, 33)),
    amount: new BN(data.subarray(33, 41), "le"),
    bump: data[41],
  }};
}}

export const PROGRAM_ID_{upper} = new PublicKey("11111111111111111111111111111111"); // replace
"#,
        name = config.name,
        pascal = name_pascal,
        upper = name_snake.to_uppercase(),
    );

    let types_dir = project_path(config, "ts");
    fs::create_dir_all(&types_dir)?;
    fs::write(types_dir.join("types.ts"), content)?;
    Ok(())
}

pub fn write_ts_tests(config: &InitConfig) -> Result<()> {
    let name_pascal = to_pascal(&config.name);

    let content = format!(
        r#"import {{
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
  Transaction,
  TransactionInstruction,
  LAMPORTS_PER_SOL,
}} from "@solana/web3.js";
import BN from "bn.js";
import {{ assert }} from "chai";
import {{ buildInitializeIx, buildExecuteIx, decode{pascal}State }} from "../ts/types";

// ─── Test harness ────────────────────────────────────────────────────────────
// Uses bankrun for local validator-free testing.
// Run: npm test (requires @solana/bankrun)

describe("{name}", () => {{
  const connection = new Connection("http://localhost:8899", "confirmed");
  const payer = Keypair.generate();
  let stateAccount: Keypair;
  const PROGRAM_ID = new PublicKey("11111111111111111111111111111111"); // replace

  before(async () => {{
    // Airdrop to payer
    const sig = await connection.requestAirdrop(payer.publicKey, 2 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig);
    stateAccount = Keypair.generate();
  }});

  it("initializes the state account", async () => {{
    const ix = buildInitializeIx(
      payer.publicKey,
      stateAccount.publicKey,
      PROGRAM_ID,
      {{ bump: 255, amount: new BN(1_000_000) }}
    );

    const tx = new Transaction().add(new TransactionInstruction(ix));
    const sig = await sendAndConfirmTransaction(connection, tx, [payer, stateAccount]);
    console.log("initialize tx:", sig);

    const info = await connection.getAccountInfo(stateAccount.publicKey);
    assert.isNotNull(info);
    const state = decode{pascal}State(Buffer.from(info!.data));
    assert.equal(state.tag, 1);
    assert.isTrue(state.amount.eq(new BN(1_000_000)));
  }});

  it("executes an update", async () => {{
    const ix = buildExecuteIx(
      payer.publicKey,
      stateAccount.publicKey,
      PROGRAM_ID,
      {{ amount: new BN(500_000) }}
    );

    const tx = new Transaction().add(new TransactionInstruction(ix));
    await sendAndConfirmTransaction(connection, tx, [payer]);
  }});
}});
"#,
        name = config.name,
        pascal = name_pascal,
    );

    fs::write(project_path(config, "tests/program.test.ts"), content)?;
    Ok(())
}

pub fn write_package_json(config: &InitConfig) -> Result<()> {
    let content = format!(
        r#"{{
  "name": "{name}",
  "version": "0.1.0",
  "private": true,
  "scripts": {{
    "test": "ts-mocha -p ./tsconfig.json tests/**/*.test.ts",
    "build:types": "tsc"
  }},
  "devDependencies": {{
    "@types/bn.js": "^5.1.5",
    "@types/chai": "^4.3.12",
    "@types/mocha": "^10.0.6",
    "chai": "^4.4.1",
    "mocha": "^10.3.0",
    "ts-mocha": "^10.0.0",
    "typescript": "^5.4.5"
  }},
  "dependencies": {{
    "@solana/web3.js": "^1.91.8",
    "@solana/bankrun": "^0.3.0",
    "bn.js": "^5.2.1"
  }}
}}
"#,
        name = config.name,
    );

    fs::write(project_path(config, "package.json"), content)?;
    Ok(())
}

pub fn write_tsconfig_with_config(config: &InitConfig) -> Result<()> {
    let content = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "strict": true,
    "esModuleInterop": true,
    "outDir": "./dist",
    "rootDir": "."
  },
  "include": ["ts/**/*", "tests/**/*"],
  "exclude": ["node_modules", "dist"]
}
"#;
    fs::write(project_path(config, "tsconfig.json"), content)?;
    Ok(())
}

pub fn write_rust_tests(config: &InitConfig) -> Result<()> {
    let name_snake = config.name.replace('-', "_");

    let content = format!(
        r#"//! Integration tests using solana-program-test.
#![cfg(feature = "test-sbf")]

use {name_snake}::process_instruction;
use solana_program_test::*;
use solana_sdk::{{
    instruction::{{AccountMeta, Instruction}},
    pubkey::Pubkey,
    signature::{{Keypair, Signer}},
    system_program,
    transaction::Transaction,
}};

fn program_test(program_id: Pubkey) -> ProgramTest {{
    ProgramTest::new("{name_snake}", program_id, processor!(process_instruction))
}}

#[tokio::test]
async fn test_initialize() {{
    let program_id = Pubkey::new_unique();
    let mut ctx = program_test(program_id).start_with_context().await;

    let state_account = Keypair::new();
    let payer = ctx.payer.insecure_clone();

    // Build the initialize instruction
    let mut data = vec![0u8]; // discriminator = Initialize
    data.push(255u8);          // bump
    data.extend_from_slice(&1_000_000u64.to_le_bytes()); // amount

    let ix = Instruction {{
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(state_account.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data,
    }};

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&payer.pubkey()),
        &[&payer, &state_account],
        ctx.last_blockhash,
    );

    ctx.banks_client.process_transaction(tx).await.unwrap();
}}
"#,
        name_snake = name_snake,
    );

    fs::write(project_path(config, "tests/integration.rs"), content)?;
    Ok(())
}

pub fn write_gitignore(config: &InitConfig) -> Result<()> {
    let content = "/target\nnode_modules\ndist\n.env\n*.so\n";
    fs::write(project_path(config, ".gitignore"), content)?;
    Ok(())
}

pub fn write_readme(config: &InitConfig) -> Result<()> {
    let mut features = String::new();
    if config.anchor { features.push_str("- Anchor compatibility layer\n"); }
    if config.token  { features.push_str("- SPL Token / ATA crates\n"); }
    if config.idl    { features.push_str("- IDL + TypeScript type bindings (`idl/` and `ts/`)\n"); }
    if config.ts_tests { features.push_str("- TypeScript test scaffold (`tests/*.test.ts`)\n"); }

    let content = format!(
        r#"# {name}

A [Pinocchio](https://github.com/febo/pinocchio) Solana program.

## Features
{features}
- Zero-copy instruction deserialization
- Rust integration tests (`tests/integration.rs`)

## Build

```bash
cargo build-sbf
```

## Test (Rust)

```bash
cargo test-sbf
```
{ts_test_section}
## Snippets

Use the `pino` CLI for quick boilerplate:

```bash
pino snippet list
pino snippet create-account
pino snippet deserialize
```

## Crates

| Crate | Version |
|---|---|
| pinocchio | 0.6 |
| pinocchio-log | 0.3 |
| pinocchio-pubkey | 0.2 |
| pinocchio-system | 0.2 |
{token_row}
"#,
        name = config.name,
        features = features,
        token_row = if config.token {
            "| pinocchio-token | 0.4 |\n| pinocchio-associated-token | 0.1 |"
        } else { "" },
        ts_test_section = if config.ts_tests {
            "\n## Test (TypeScript)\n\n```bash\nnpm install\nnpm test\n```\n"
        } else { "" },
    );

    fs::write(project_path(config, "README.md"), content)?;
    Ok(())
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn to_pascal(s: &str) -> String {
    s.split(['-', '_'])
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}
