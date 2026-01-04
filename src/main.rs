use std::{env::current_dir, io, path::PathBuf};

use anyhow::{Context, Result};
use bok::chart_of_accounts::bas::{BasLanguange, BasYear, get_bas_plan};
use bok::{EntryLine, Ledger, Side};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Generator, Shell, generate};

use crate::completions::{
    bash_dynamic_completions, fish_dynamic_completions, zsh_dynamic_completions,
};
mod completions;

/// Supported languages for the chart of accounts
#[derive(Clone, Copy, ValueEnum)]
enum Language {
    /// English
    En,
    /// Swedish
    Sv,
}

impl From<Language> for BasLanguange {
    fn from(lang: Language) -> Self {
        match lang {
            Language::En => BasLanguange::EN,
            Language::Sv => BasLanguange::SV,
        }
    }
}

/// Supported years for the chart of accounts
#[derive(Clone, Copy, ValueEnum)]
enum SupportedYear {
    /// Year 2025
    #[value(name = "2025")]
    Y2025,
}

impl From<SupportedYear> for BasYear {
    fn from(year: SupportedYear) -> Self {
        match year {
            SupportedYear::Y2025 => BasYear::Y2025,
        }
    }
}

#[derive(Parser)]
#[command(name = "bok")]
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
    Init {
        year: SupportedYear,
        #[clap(long, short, default_value = ".bok")]
        directory: Option<PathBuf>,
        #[clap(long, short, default_value = "en")]
        language: Language,
    },
    /// Generate shell completions
    Completions {
        /// The shell to generate completions for
        shell: Shell,
    },
    /// List all available accounts for the current year
    Accounts,
}

fn print_completions<G: Generator>(generator: G, cmd: &mut clap::Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
}

fn main() -> Result<()> {
    let args = BokArgs::parse();

    let default_path = current_dir()?.join(".bok");

    // Handle commands that don't need an existing ledger
    match args.command {
        BokCommand::Init {
            year,
            directory,
            language,
        } => {
            let bas_year: BasYear = year.into();
            let bas_language: BasLanguange = language.into();
            let accs = get_bas_plan(bas_year, bas_language).context("Failed to get BAS plan")?;
            Ledger::init(bas_year as usize, directory.unwrap_or(default_path), accs)?;
            println!("Ledger initialized");
            return Ok(());
        }
        BokCommand::Completions { shell } => {
            let mut cmd = BokArgs::command();
            print_completions(shell, &mut cmd);
            // Append dynamic completions based on shell type
            match shell {
                Shell::Fish => print!("{}", fish_dynamic_completions()),
                Shell::Bash => print!("{}", bash_dynamic_completions()),
                Shell::Zsh => print!("{}", zsh_dynamic_completions()),
                _ => {}
            }
            return Ok(());
        }
        _ => {}
    }

    let mut ledger = Ledger::from_dir(default_path.clone())?;
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
        BokCommand::Accounts => {
            let chart = ledger.chart_of_accounts()?;
            for account in chart {
                println!(
                    "{}\t{:?}\t{}",
                    account.account_number, account.account_type, account.name,
                );
            }
        }
        BokCommand::Init { .. } | BokCommand::Completions { .. } => {
            unreachable!("Already handled above")
        }
    }
    Ok(())
}
