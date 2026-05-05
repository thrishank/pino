pub const CREATE_ACCOUNT: &str = r#"
use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;

pub fn process_create_account(
    accounts: &[AccountInfo],
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> ProgramResult {
    let [payer, new_account, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    CreateAccount {
        from: payer,
        to: new_account,
        lamports,
        space,
        owner,
    }
    .invoke()?;

    Ok(())
}
"#;

pub const TRANSFER: &str = r#"
use pinocchio::{account_info::AccountInfo, ProgramResult};
use pinocchio_system::instructions::Transfer;

pub fn process_transfer(
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let [from, to, _system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Transfer { from, to, lamports: amount }.invoke()?;

    Ok(())
}
"#;

pub const DESERIALIZE: &str = r#"
// Zero-copy instruction data deserialization (no allocations)
use core::mem::size_of;

#[repr(C)]
pub struct MyInstruction {
    pub discriminator: u8,
    pub amount: u64,
    pub bump: u8,
}

impl MyInstruction {
    pub const SIZE: usize = size_of::<Self>();

    /// Parse from raw instruction_data bytes.
    /// Called from the program entrypoint.
    pub fn from_bytes(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::SIZE {
            return Err(ProgramError::InvalidInstructionData);
        }
        // SAFETY: repr(C), data length checked above
        Ok(unsafe { &*(data.as_ptr() as *const Self) })
    }
}

// In your entrypoint / process_instruction:
pub fn process(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = MyInstruction::from_bytes(instruction_data)?;
    match ix.discriminator {
        0 => process_variant_a(accounts, ix.amount),
        1 => process_variant_b(accounts, ix.bump),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
"#;

pub const PDA: &str = r#"
use pinocchio::pubkey::Pubkey;
use pinocchio_pubkey::pubkey;

// Declare a static program ID at compile time
const MY_PROGRAM_ID: Pubkey = pubkey!("YourProgramId11111111111111111111111111111111");

// Derive a PDA and verify it matches an account
pub fn verify_pda(
    expected: &AccountInfo,
    seeds: &[&[u8]],
    bump: u8,
) -> ProgramResult {
    let mut seeds_with_bump: Vec<&[u8]> = seeds.to_vec();
    let bump_slice = [bump];
    seeds_with_bump.push(&bump_slice);

    let (derived, _bump) = Pubkey::find_program_address(seeds, &MY_PROGRAM_ID);

    if derived != *expected.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    Ok(())
}
"#;

pub const LOG: &str = r#"
use pinocchio_log::log;

pub fn process_example(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    // Simple string
    log!("processing transfer");

    // With arguments (no heap allocation)
    log!("amount: {}", amount);
    log!("accounts: {}", accounts.len());

    // Log an error before returning
    if amount == 0 {
        log!("error: zero amount");
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}
"#;

pub const PUBKEY: &str = r#"
use pinocchio::pubkey::Pubkey;
use pinocchio_pubkey::pubkey;

// Declare known keys at compile time (zero runtime cost)
const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const WSOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub fn check_mint(mint_account: &AccountInfo) -> ProgramResult {
    let key = mint_account.key();

    if key == &USDC_MINT {
        log!("mint is USDC");
    } else if key == &WSOL_MINT {
        log!("mint is wSOL");
    } else {
        return Err(ProgramError::InvalidArgument);
    }

    Ok(())
}
"#;
