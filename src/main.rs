use std::{env::current_dir, fs::create_dir_all, io::Result};

use bok::{EntryLine, Ledger, Side};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct BokArgs {
    #[command(subcommand)]
    command: BokCommand,
}

#[derive(Subcommand)]
enum BokCommand {
    Rec {
        left: u16,
        right: u16,
        amount: usize,
        description: String,
    },
}

fn main() -> Result<()> {
    let args = BokArgs::parse();
    match args.command {
        BokCommand::Rec {
            left,
            right,
            amount,
            description,
        } => {
            let default_path = current_dir()?.join(".bok/");
            if !default_path.exists() {
                create_dir_all(&default_path)?;
            }
            let mut ledger = Ledger::new(2025, default_path);

            let left_str = left.to_string();
            let right_str = right.to_string();
            let lines = vec![
                EntryLine::new(&left_str, amount, Side::Debit, Option::<String>::None),
                EntryLine::new(&right_str, amount, Side::Credit, Option::<String>::None),
            ];
            let entry = ledger.add_entry("A1", description, lines)?;
            dbg!(entry);
        }
    }
    Ok(())
}
