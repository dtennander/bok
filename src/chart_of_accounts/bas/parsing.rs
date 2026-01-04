use std::io::{Read, Seek};

use calamine::{Data, Reader, Xlsx};

use crate::chart_of_accounts::{Account, ChartOfAccount, bas::bas_class_to_type};
use crate::error::{BasParsingError, Result};

// Constants for Excel file structure
const HEADER_ROW_COUNT: usize = 8;
const COLUMN_ACCOUNT: usize = 5;
const COLUMN_DESCRIPTION: usize = 6;

/// Attempts to parse a cell value as a usize (account number)
fn parse_account_number(cell: &Data) -> Option<usize> {
    match cell {
        Data::Int(num) => Some(*num as usize),
        Data::Float(num) => Some(*num as usize),
        Data::String(s) => s.parse::<usize>().ok(),
        _ => None,
    }
}

/// Attempts to parse a cell value as a String (description)
fn parse_description(cell: &Data) -> Option<String> {
    match cell {
        Data::String(s) => Some(s.clone()),
        _ => None,
    }
}

/// Extracts account number from a row, trying column C first, then column B
fn extract_account_number(row: &[Data]) -> Option<usize> {
    // Fallback to 3-digit from column B
    row.get(COLUMN_ACCOUNT).and_then(parse_account_number)
}

/// Represents the raw data extracted from a BAS Excel row
struct BasRowData {
    account_number: usize,
    description: String,
}

/// Attempts to parse a single row into BasRowData
fn parse_bas_row(row: &[Data]) -> Option<BasRowData> {
    let account_number = extract_account_number(row)?;

    let description = row.get(COLUMN_DESCRIPTION).and_then(parse_description)?;

    // Skip empty descriptions
    if description.trim().is_empty() {
        return None;
    }

    Some(BasRowData {
        account_number,
        description,
    })
}
/// Converts BasRowData into an Account, determining the account type
fn create_account(row_data: BasRowData) -> Result<Account> {
    let class = (row_data.account_number / 1000) as u8;
    let account_type = bas_class_to_type(class)?;

    Ok(Account::new(
        row_data.account_number,
        row_data.description.clone(),
        row_data.description,
        account_type,
    ))
}

/// Opens the BAS workbook and returns the first worksheet range
pub fn load_bas_worksheet<R: Read + Seek>(r: R) -> Result<ChartOfAccount> {
    let mut workbook = Xlsx::new(r).map_err(|e| BasParsingError::WorkbookOpen(e.to_string()))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .ok_or(BasParsingError::NoSheets)?
        .clone();
    println!("Found first sheet to be named: {sheet_name}");

    Ok(workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| BasParsingError::WorksheetRange(e.to_string()))?
        .rows()
        .enumerate()
        .filter(|(idx, _)| *idx >= HEADER_ROW_COUNT)
        .filter_map(|(_, row)| parse_bas_row(row))
        .filter_map(|row_data| create_account(row_data).ok())
        .collect::<Vec<_>>())
}
