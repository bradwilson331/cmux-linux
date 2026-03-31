//! Standalone generator for shell completions and man page.
//! Usage: cargo run --bin cmux-generate
//! Outputs to packaging/completions/ and packaging/man/
//!
//! NOTE: Uses #[path] to include the CLI module directly instead of
//! going through lib.rs. A lib.rs target breaks ghostty FFI linking
//! for cmux-app (see commit fd436c5b).

#[path = "../cli/mod.rs"]
mod cli;

use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::fs;
use std::path::Path;

use cli::Cli;

fn main() -> std::io::Result<()> {
    let mut cmd = Cli::command();

    // Generate shell completions
    let comp_dir = Path::new("packaging/completions");
    fs::create_dir_all(comp_dir)?;

    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        let path = generate_to(shell, &mut cmd, "cmux", comp_dir)?;
        eprintln!("Generated: {}", path.display());
    }

    // Generate man page
    let man_dir = Path::new("packaging/man");
    fs::create_dir_all(man_dir)?;

    let man = Man::new(cmd);
    let mut buf = Vec::new();
    man.render(&mut buf)?;
    fs::write(man_dir.join("cmux.1"), buf)?;
    eprintln!("Generated: packaging/man/cmux.1");

    Ok(())
}
