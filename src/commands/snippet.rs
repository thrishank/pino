use crate::SnippetKind;
use crate::snippets;
use anyhow::Result;
use colored::Colorize;

pub fn run(kind: &SnippetKind) -> Result<()> {
    match kind {
        SnippetKind::List => print_list(),
        SnippetKind::CreateAccount => print_snippet(
            "create_account",
            "Create a system-owned account",
            snippets::CREATE_ACCOUNT,
        ),
        SnippetKind::Transfer => print_snippet(
            "transfer",
            "Transfer lamports between accounts",
            snippets::TRANSFER,
        ),
        SnippetKind::Deserialize => print_snippet(
            "deserialize",
            "Deserialize instruction data",
            snippets::DESERIALIZE,
        ),
        SnippetKind::Pda => print_snippet("pda", "Derive and verify a PDA", snippets::PDA),
        SnippetKind::Log => print_snippet(
            "log",
            "Emit a program log with pinocchio-log",
            snippets::LOG,
        ),
        SnippetKind::Pubkey => {
            print_snippet("pubkey", "Declare and compare pubkeys", snippets::PUBKEY)
        }
    }
    Ok(())
}

fn print_list() {
    println!(
        "\n  {} Available snippets:\n",
        "pino snippet".bright_yellow().bold()
    );

    let items = [
        ("create-account", "Create a system-owned account"),
        ("transfer", "Transfer lamports between accounts"),
        ("deserialize", "Deserialize instruction data"),
        ("pda", "Derive and verify a PDA"),
        ("log", "Emit a program log"),
        ("pubkey", "Declare and compare pubkeys"),
    ];

    for (cmd, desc) in &items {
        println!(
            "  {}  {:<20} {}",
            "›".bright_yellow(),
            format!("pino snippet {}", cmd).bright_cyan(),
            desc.bright_white()
        );
    }
    println!();
}

fn print_snippet(name: &str, desc: &str, code: &str) {
    println!();
    println!("  {} {}", "//".bright_black(), desc.bright_white().bold());
    println!("  {} {}\n", "snippet:".bright_yellow(), name.bright_cyan());

    // Syntax-highlight keywords manually (basic)
    for line in code.lines() {
        let colored = highlight_rust(line);
        println!("    {}", colored);
    }
    println!();
}

fn highlight_rust(line: &str) -> String {
    // Very lightweight keyword coloring
    let keywords = [
        "use ", "let ", "fn ", "pub ", "struct ", "impl ", "if ", "else ", "for ", "in ", "mut ",
        "return ", "Ok(", "Err(", "unsafe ", "const ", "static ", "&mut ", "->",
    ];
    let mut out = line.to_string();

    // Comments
    if out.trim_start().starts_with("//") {
        return out.bright_black().to_string();
    }

    // Simple string literal highlighting
    // (skip full parser — just color leading keyword)
    for kw in keywords {
        if out.contains(kw) {
            out = out.replace(kw, &kw.yellow().to_string());
            break;
        }
    }
    out
}
