use std::io;
use thiserror::Error;

/// Main error type for the bok library
#[derive(Error, Debug)]
pub enum BokError {
    /// Error occurred while parsing BAS Excel file
    #[error("Failed to parse BAS file: {0}")]
    BasParsing(#[from] BasParsingError),

    /// Error occurred while downloading BAS file
    #[error("Failed to download BAS file: {0}")]
    BasDownload(#[from] reqwest::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    /// Invalid account class in BAS system
    #[error("Invalid BAS account class: {0}")]
    InvalidBasClass(u8),
}

/// Errors specific to BAS file parsing
#[derive(Error, Debug)]
pub enum BasParsingError {
    /// Failed to open the Excel workbook
    #[error("Failed to open workbook: {0}")]
    WorkbookOpen(String),

    /// No sheets found in the workbook
    #[error("No sheets found in workbook")]
    NoSheets,

    /// Failed to get worksheet range
    #[error("Failed to get worksheet range: {0}")]
    WorksheetRange(String),
}

/// Result type alias for bok operations
pub type Result<T> = std::result::Result<T, BokError>;
