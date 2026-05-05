use crate::scaffold;
use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct InitConfig {
    pub name: String,
    pub anchor: bool,
    pub token: bool,
    pub idl: bool,
    pub ts_tests: bool,
}

pub fn run(
    name: &str,
    anchor: bool,
    token: bool,
    idl: bool,
    ts_tests: bool,
    yes: bool,
) -> Result<()> {
    let config = if yes {
        InitConfig {
            name: name.to_string(),
            anchor,
            token,
            idl,
            ts_tests,
        }
    } else {
        prompt_config(name, anchor, token, idl, ts_tests)?
    };

    println!(
        "\n{} {}\n",
        "Scaffolding".bright_cyan(),
        config.name.bright_white().bold()
    );

    let steps = build_steps(&config);
    let total = steps.len() as u64;

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::with_template(
            "  {spinner:.yellow} [{bar:30.yellow/white}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("█▓░"),
    );
    pb.enable_steady_tick(Duration::from_millis(80));

    for (msg, action) in steps {
        pb.set_message(msg.to_string());
        action(&config)?;
        pb.inc(1);
        std::thread::sleep(Duration::from_millis(120)); // visual pacing
    }

    pb.finish_and_clear();

    print_success(&config);
    Ok(())
}

fn prompt_config(
    name: &str,
    anchor: bool,
    token: bool,
    idl: bool,
    ts_tests: bool,
) -> Result<InitConfig> {
    let theme = ColorfulTheme::default();

    println!("{}", "Configure your project:".bright_white().bold());
    println!();

    let options = &[
        "Anchor compatibility layer",
        "Token crates (pinocchio-token, pinocchio-associated-token)",
        "IDL + TypeScript type generation",
        "TypeScript test scaffold (Bankrun / Mocha)",
    ];

    let defaults = &[anchor, token, idl, ts_tests];
    let default_indices: Vec<usize> = defaults
        .iter()
        .enumerate()
        .filter_map(|(i, &on)| if on { Some(i) } else { None })
        .collect();

    let selections = MultiSelect::with_theme(&theme)
        .with_prompt("Select features")
        .items(options)
        .defaults(
            &defaults
                .iter()
                .map(|&b| b)
                .collect::<Vec<bool>>(),
        )
        .interact()?;

    let _ = default_indices; // suppress warning

    Ok(InitConfig {
        name: name.to_string(),
        anchor: selections.contains(&0),
        token: selections.contains(&1),
        idl: selections.contains(&2),
        ts_tests: selections.contains(&3),
    })
}

type StepFn = Box<dyn Fn(&InitConfig) -> Result<()>>;

fn build_steps(config: &InitConfig) -> Vec<(&'static str, StepFn)> {
    let mut steps: Vec<(&'static str, StepFn)> = vec![
        (
            "Creating project directory",
            Box::new(|c| scaffold::create_project_dir(&c.name)),
        ),
        (
            "Writing Cargo.toml",
            Box::new(|c| scaffold::write_cargo_toml(c)),
        ),
        (
            "Writing lib.rs with instruction entrypoint",
            Box::new(|c| scaffold::write_lib_rs(c)),
        ),
        (
            "Writing instruction deserializer",
            Box::new(|c| scaffold::write_instruction_rs(c)),
        ),
        (
            "Writing processor stubs",
            Box::new(|c| scaffold::write_processor_rs(c)),
        ),
        (
            "Writing error types",
            Box::new(|c| scaffold::write_errors_rs_with_config(c)),
        ),
        (
            "Writing state / account structs",
            Box::new(|c| scaffold::write_state_rs_with_config(c)),
        ),
    ];

    if config.idl {
        steps.push((
            "Generating IDL definition (anchor-compatible)",
            Box::new(|c| scaffold::write_idl(c)),
        ));
        steps.push((
            "Generating TypeScript type bindings",
            Box::new(|c| scaffold::write_ts_types(c)),
        ));
    }

    if config.ts_tests {
        steps.push((
            "Scaffolding TypeScript tests",
            Box::new(|c| scaffold::write_ts_tests(c)),
        ));
        steps.push((
            "Writing package.json",
            Box::new(|c| scaffold::write_package_json(c)),
        ));
        steps.push((
            "Writing tsconfig.json",
            Box::new(|c| scaffold::write_tsconfig_with_config(c)),
        ));
    }

    steps.push((
        "Writing Rust integration tests",
        Box::new(|c| scaffold::write_rust_tests(c)),
    ));

    steps.push((".gitignore", Box::new(|c| scaffold::write_gitignore(c))));
    steps.push(("README.md", Box::new(|c| scaffold::write_readme(c))));

    steps
}

fn print_success(config: &InitConfig) {
    println!();
    println!("  {} {}", "✓".green().bold(), "Project created!".bright_white().bold());
    println!();
    println!("  {}", "Next steps:".bright_white());
    println!(
        "    {}  {}",
        "$".bright_black(),
        format!("cd {}", config.name).bright_cyan()
    );
    println!(
        "    {}  {}",
        "$".bright_black(),
        "cargo build-sbf".bright_cyan()
    );

    if config.ts_tests {
        println!(
            "    {}  {}",
            "$".bright_black(),
            "npm install && npm test".bright_cyan()
        );
    }

    println!();
    println!("  {}", "Included crates:".bright_white());
    println!("    {} pinocchio", "·".bright_yellow());
    println!("    {} pinocchio-log", "·".bright_yellow());
    println!("    {} pinocchio-pubkey", "·".bright_yellow());
    println!("    {} pinocchio-system", "·".bright_yellow());
    if config.token {
        println!("    {} pinocchio-token", "·".bright_yellow());
        println!("    {} pinocchio-associated-token", "·".bright_yellow());
    }
    if config.anchor {
        println!("    {} anchor-lang (compatibility)", "·".bright_yellow());
    }
    println!();
    println!("  {} Use {} for quick code snippets",
        "Tip:".bright_yellow().bold(),
        "pino snippet list".bright_cyan()
    );
    println!();

    // Confirm user wants to open
    let open = Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Open project in $EDITOR?")
        .default(false)
        .interact();

    if let Ok(true) = open {
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
        let _ = std::process::Command::new(editor)
            .arg(&config.name)
            .spawn();
    }
}
