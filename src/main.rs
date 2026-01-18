mod splitter;
mod utils;

use anyhow::Result;
use console::{style, Term};
use rfd::FileDialog;
use std::path::PathBuf;

use crate::splitter::split_zip;
use crate::utils::format_size;

fn main() {
    let term = Term::stdout();
    let _ = term.clear_screen();

    print_header();

    match run() {
        Ok(_) => {
            println!();
            println!(
                "  {} Press Enter to exit...",
                style("✓").green().bold()
            );
            let _ = term.read_line();
        }
        Err(e) => {
            println!();
            println!("  {} {}", style("ERROR:").red().bold(), e);
            println!();
            println!("  Press Enter to exit...");
            let _ = term.read_line();
            std::process::exit(1);
        }
    }
}

fn run() -> Result<()> {
    // Show file dialog
    println!(
        "  {} Opening file dialog...",
        style("→").cyan().bold()
    );
    println!();

    let file_path = select_zip_file()?;

    // Display selected file info
    let metadata = std::fs::metadata(&file_path)?;
    println!(
        "  {} Selected: {}",
        style("📁").cyan(),
        style(file_path.file_name().unwrap().to_string_lossy()).white().bold()
    );
    println!(
        "  {} Size: {}",
        style("📊").cyan(),
        style(format_size(metadata.len())).yellow()
    );
    println!();

    // Perform the split
    println!(
        "  {} Splitting ZIP file...",
        style("⚡").yellow().bold()
    );
    println!();

    let result = split_zip(&file_path)?;

    // Display results
    println!();
    print_divider();
    println!(
        "  {} {}",
        style("SPLIT COMPLETE").green().bold(),
        style("━━━━━━━━━━━━━━━━━━━━━━━━━━").dim()
    );
    print_divider();
    println!();

    println!(
        "  {} Files processed: {}",
        style("•").dim(),
        style(result.total_files_processed).cyan()
    );

    if result.files_split > 0 {
        println!(
            "  {} Large files split: {}",
            style("•").dim(),
            style(result.files_split).yellow()
        );
    }

    println!(
        "  {} Output archives: {}",
        style("•").dim(),
        style(result.output_files.len()).green().bold()
    );
    println!();

    println!("  {} Output files:", style("📦").cyan());
    for (i, path) in result.output_files.iter().enumerate() {
        let size = std::fs::metadata(path)
            .map(|m| format_size(m.len()))
            .unwrap_or_else(|_| "???".to_string());
        
        println!(
            "     {} {} ({})",
            style(format!("{}.", i + 1)).dim(),
            style(path.file_name().unwrap().to_string_lossy()).white(),
            style(size).yellow()
        );
    }

    Ok(())
}

fn select_zip_file() -> Result<PathBuf> {
    let file = FileDialog::new()
        .add_filter("ZIP Archives", &["zip"])
        .set_title("Select a ZIP file to split")
        .pick_file();

    match file {
        Some(path) => Ok(path),
        None => Err(anyhow::anyhow!("No file selected. Operation cancelled.")),
    }
}

fn print_header() {
    println!();
    println!(
        "  {}",
        style("╔═══════════════════════════════════════════════════╗").cyan()
    );
    println!(
        "  {}{}{}",
        style("║").cyan(),
        style("              SHARD-A-ZIP v1.0.0                    ").white().bold(),
        style("║").cyan()
    );
    println!(
        "  {}{}{}",
        style("║").cyan(),
        style("         Cross-Platform ZIP Splitter               ").dim(),
        style("║").cyan()
    );
    println!(
        "  {}",
        style("╚═══════════════════════════════════════════════════╝").cyan()
    );
    println!();
    print_divider();
    println!();
}

fn print_divider() {
    println!("  {}", style("───────────────────────────────────────────────────").dim());
}
