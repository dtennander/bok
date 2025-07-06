use std::{env::current_dir, io::Result, path::PathBuf};

use bok::{EntryLine, Ledger, Side};
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct BokArgs {
    #[command(subcommand)]
    command: BokCommand,
}

#[derive(Subcommand)]
enum BokCommand {
    /// Record a item in the Ledger.
    #[command(name = "record", visible_alias = "rec")]
    Rec {
        debit: u16,
        credit: u16,
        amount: usize,
        description: String,
    },
    /// Show a entry using it's REF.
    ///
    /// A REF can be either the sha of that entry or a symbol reference pointing to a entry, i.e.
    /// HEAD.
    Show { r#ref: String },
    /// Show the history from a given REF.
    Log { r#ref: Option<String> },
    /// Initialize a book from a new year.
    Init { year: usize, dir: Option<PathBuf> },
}

fn main() -> Result<()> {
    let args = BokArgs::parse();

    let default_path = current_dir()?.join(".bok");

    if let BokCommand::Init { year, dir } = args.command {
        Ledger::init(year, dir.unwrap_or(default_path))?;
        println!("Ledger initialized");
        return Ok(());
    }

    let mut ledger = Ledger::from_dir(default_path)?;
    match args.command {
        BokCommand::Rec {
            debit: left,
            credit: right,
            amount,
            description,
        } => {
            let left_str = left.to_string();
            let right_str = right.to_string();
            let lines = vec![
                EntryLine::new(&left_str, amount, Side::Debit, Option::<String>::None),
                EntryLine::new(&right_str, amount, Side::Credit, Option::<String>::None),
            ];
            let entry_ref = ledger.add_entry("A1", &description, lines)?;
            let entry = ledger.get_entry(&entry_ref)?;
            println!("{}", entry.show());
        }
        BokCommand::Show { r#ref: entry_ref } => {
            let hash = ledger.from_ref(&entry_ref)?;
            let entry = ledger.get_entry(&hash)?;
            let show = entry.show();
            print!("{}", show);
        }
        BokCommand::Log { r#ref: start } => {
            let hash = ledger.from_ref(&start.unwrap_or("HEAD".to_string()))?;
            let out = ledger.show_log(hash)?;
            print!("{}", out);
        }
        BokCommand::Init { .. } => {
            panic!("Shouldn't happen!")
        }
    }
    Ok(())
}
