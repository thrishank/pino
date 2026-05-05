use anyhow::Result;
use colored::Colorize;

pub fn run() -> Result<()> {
    println!("\n  {} Available Pinocchio crates\n", "📦".bright_yellow());

    let crates = [
        (
            "pinocchio",
            "Core entrypoint, account info, CPI primitives",
            true,
            "https://crates.io/crates/pinocchio",
        ),
        (
            "pinocchio-log",
            "Efficient sol_log! macro with format args",
            true,
            "https://crates.io/crates/pinocchio-log",
        ),
        (
            "pinocchio-pubkey",
            "pubkey! macro for static key declarations",
            true,
            "https://crates.io/crates/pinocchio-pubkey",
        ),
        (
            "pinocchio-system",
            "System program CPI helpers (create, transfer, alloc)",
            true,
            "https://crates.io/crates/pinocchio-system",
        ),
        (
            "pinocchio-token",
            "SPL Token program CPI instructions",
            false,
            "https://crates.io/crates/pinocchio-token",
        ),
        (
            "pinocchio-token-2022",
            "SPL Token 2022 program CPI instructions",
            false,
            "https://crates.io/crates/pinocchio-token-2022",
        ),
        (
            "pinocchio-associated-token",
            "Associated Token Account CPI helpers",
            false,
            "https://crates.io/crates/pinocchio-associated-token",
        ),
    ];

    println!(
        "  {:<35} {:<48} {}",
        "CRATE".bright_white().bold(),
        "DESCRIPTION".bright_white().bold(),
        "DEFAULT".bright_white().bold()
    );
    println!("  {}", "─".repeat(95).bright_black());

    for (name, desc, included, _url) in &crates {
        let flag = if *included {
            "✓ yes".green().to_string()
        } else {
            "· --token flag".bright_black().to_string()
        };
        println!("  {:<35} {:<48} {}", name.bright_cyan(), desc, flag);
    }

    println!();
    println!(
        "  {} Use {} to include token crates",
        "Tip:".bright_yellow().bold(),
        "pino init <name> --token".bright_cyan()
    );
    println!();

    Ok(())
}
