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
    Show {
        hash: String,
    },
}

fn main() -> Result<()> {
    let args = BokArgs::parse();

    let default_path = current_dir()?.join(".bok");
    create_dir_all(&default_path)?;
    let mut ledger = Ledger::new(2025, default_path);

    match args.command {
        BokCommand::Rec {
            left,
            right,
            amount,
            description,
        } => {
            let left_str = left.to_string();
            let right_str = right.to_string();
            let lines = vec![
                EntryLine::new(&left_str, amount, Side::Debit, Option::<String>::None),
                EntryLine::new(&right_str, amount, Side::Credit, Option::<String>::None),
            ];
            let entry_ref = ledger.add_entry("A1", description, lines)?;
            dbg!(ledger.get_entry(&entry_ref)?);
        }
        BokCommand::Show { hash } => match &(ledger.find_hash(&hash)?[..]) {
            [hash] => {
                let entry = ledger.get_entry(hash)?;
                dbg!(entry);
            }
            [] => println!("No entry found..."),
            _ => println!("Not a unique prefix"),
        },
    }
    Ok(())
}
