mod disasm;
mod parser;
mod strings;
mod types;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser as ClapParser;

use crate::parser::ParsedBinary;
use crate::types::AnalysisResult;

#[derive(ClapParser, Debug)]
#[command(name = "aegisto", about = "Binary analysis framework using AI agents")]
struct Cli {
    /// Path to the binary file to analyze
    #[arg(short, long)]
    binary: PathBuf,

    /// If provided, write JSON results to this file instead of stdout
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Maximum number of instructions to disassemble (default: 500)
    #[arg(long, default_value_t = 500)]
    max_instructions: usize,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let parsed: ParsedBinary =
        parser::parse_file(&cli.binary).context("Failed to parse binary")?;

    let raw_bytes = std::fs::read(&cli.binary)
        .with_context(|| format!("Failed to read file: {}", cli.binary.display()))?;

    let extracted_strings = strings::extract_strings(&raw_bytes, 4);

    let instructions = match &parsed.executable_section {
        Some((_name, base_va, code_bytes)) => {
            disasm::disassemble(code_bytes, *base_va, cli.max_instructions)?
        }
        None => Vec::new(),
    };

    let result = AnalysisResult {
        file_path: cli.binary.display().to_string(),
        format: parsed.format,
        entry_point: parsed.entry_point,
        sections: parsed.sections,
        imports: parsed.imports,
        instructions,
        strings: extracted_strings,
    };

    match cli.output {
        Some(output_path) => {
            let json = serde_json::to_string_pretty(&result)
                .context("Failed to serialize results to JSON")?;
            std::fs::write(&output_path, &json)
                .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
            eprintln!("Results written to {}", output_path.display());
        }
        None => {
            print_human_readable(&result);
        }
    }

    Ok(())
}

fn print_human_readable(result: &AnalysisResult) {
    println!("=== Aegisto Analysis Result ===");
    println!("File:       {}", result.file_path);
    println!("Format:     {}", result.format);
    println!("Entry:      {:#x}", result.entry_point);
    println!();

    println!("--- Sections ({}) ---", result.sections.len());
    for s in &result.sections {
        println!(
            "  {:<20} size={:<#10x}  vaddr={:#010x}  [{}]",
            s.name, s.size, s.virtual_address, s.flags
        );
    }
    println!();

    println!("--- Imports ({}) ---", result.imports.len());
    for imp in &result.imports {
        if imp.library.is_empty() {
            println!("  {}", imp.name);
        } else {
            println!("  {} ({})", imp.name, imp.library);
        }
    }
    println!();

    println!("--- Disassembly ({} instructions) ---", result.instructions.len());
    for inst in &result.instructions {
        if inst.operands.is_empty() {
            println!("  {:#010x}:  {}", inst.address, inst.mnemonic);
        } else {
            println!("  {:#010x}:  {} {}", inst.address, inst.mnemonic, inst.operands);
        }
    }
    println!();

    println!("--- Strings ({}) ---", result.strings.len());
    for s in &result.strings {
        println!("  @ {:#08x}: \"{}\"", s.offset, s.value);
    }
}
