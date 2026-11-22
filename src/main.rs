mod cli;
mod config;
mod import;
mod model;
mod scan;
mod tui;

use anyhow::Result;
use std::io::{self, IsTerminal, Write};

fn main() -> Result<()> {
    let (force, hf, lms) = match cli::parse(std::env::args().skip(1))? {
        cli::Cli::Help => {
            print!("{}", cli::HELP);
            return Ok(());
        }
        cli::Cli::Run { force, hf, lms } => (force, hf, lms),
    };

    let config = config::load_or_create_config(hf, lms)?;
    let models = scan::find_models(&config);

    if models.is_empty() {
        println!("No GGUF models found in {}", config.hf_cache.display());
        return Ok(());
    }

    let Some(indices) = tui::run_tui(&models, force)? else {
        return Ok(());
    };

    if indices.is_empty() {
        println!("Nothing selected.");
        return Ok(());
    }

    println!("\nImporting {} model(s)...\n", indices.len());
    let summary = import::import_selected(&models, &indices, force);

    println!("Linked:    {}", summary.ok);
    if summary.skipped_linked > 0 {
        println!("Skipped:   {} (already linked)", summary.skipped_linked);
    }
    if summary.skipped_conflict > 0 {
        println!(
            "Conflicts: {} (use --force to overwrite)",
            summary.skipped_conflict
        );
    }
    if !summary.failed.is_empty() {
        println!("Failed:    {}", summary.failed.len());
        for (name, err) in &summary.failed {
            println!("  {name}: {err}");
        }
    }

    if io::stdout().is_terminal() {
        print!("\nPress Enter to exit...");
        let _ = io::stdout().flush();
        let _ = io::stdin().read_line(&mut String::new());
    }

    if !summary.failed.is_empty() {
        std::process::exit(1);
    }
    Ok(())
}
