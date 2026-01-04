use std::io::Cursor;

use crate::chart_of_accounts::{ChartOfAccount, bas::parsing::load_bas_worksheet};
use crate::error::{BokError, Result};

use super::AccountType;
mod parsing;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BasYear {
    Y2025 = 2025,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BasLanguange {
    SV,
    EN,
}

const BAS_ACCOUNT_MAP: &[(BasYear, &[(BasLanguange, &str)])] = &[(
    BasYear::Y2025,
    &[
        (
            BasLanguange::SV,
            "https://www.bas.se/wp-content/uploads/2025/01/Kontoplan-BAS-2025.xlsx",
        ),
        (
            BasLanguange::EN,
            "https://www.bas.se/wp-content/uploads/2025/01/Chart-of-accounts-BAS-2025.xlsx",
        ),
    ],
)];

fn bas_download_link(year: BasYear, language: BasLanguange) -> &'static str {
    let link = BAS_ACCOUNT_MAP.iter().find_map(|(y, links)| {
        if *y != year {
            None
        } else {
            links
                .iter()
                .find_map(|(lang, link)| if *lang == language { Some(link) } else { None })
        }
    });
    // SAFETY: Writer needs to uphold that BasYear and BasLanguange always conforms to this
    unsafe { link.unwrap_unchecked() }
}

fn bas_class_to_type(class: u8) -> Result<AccountType> {
    match class {
        1 => Ok(AccountType::Asset),
        2 => Ok(AccountType::Liability),
        3 => Ok(AccountType::Revenue),
        4..=8 => Ok(AccountType::Expense),
        _ => Err(BokError::InvalidBasClass(class)),
    }
}

pub fn get_bas_plan(year: BasYear, language: BasLanguange) -> Result<ChartOfAccount> {
    let link = bas_download_link(year, language);
    println!("Will download chart from {link}");
    let response = reqwest::blocking::get(link)?;
    let bytes = response.bytes()?;
    let accs = load_bas_worksheet(Cursor::new(bytes))?;
    println!("Got chart with {} accounts", accs.iter().len());
    Ok(accs)
}
