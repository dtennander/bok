use crate::Side;

mod bas;
pub type ChartOfAccount = Vec<Account>;

pub struct Account {
    account_number: usize,
    name: String,
    description: String,
    account_type: AccountType,
}

#[derive(Debug, Clone)]
pub enum AccountType {
    Asset,
    Liability,
    /// Equity is only used outside of the BAS swedish system as in BAS Equity is within Liabilitis
    Equity,
    Revenue,
    Expense,
}

impl AccountType {
    pub fn blanance_type(&self) -> Side {
        match self {
            AccountType::Asset | AccountType::Expense => Side::Debit,
            AccountType::Liability | AccountType::Equity | AccountType::Revenue => Side::Credit,
        }
    }

    pub fn statement_category(&self) -> FinancialStatement {
        match self {
            AccountType::Asset | AccountType::Liability | AccountType::Equity => {
                FinancialStatement::BalanceSheet
            }
            AccountType::Revenue | AccountType::Expense => FinancialStatement::IncomeStatement,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FinancialStatement {
    BalanceSheet,
    IncomeStatement,
}
