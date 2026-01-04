use crate::Side;

pub mod bas;
pub type ChartOfAccount = Vec<Account>;
mod serde;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Account {
    account_number: usize,
    name: String,
    description: String,
    account_type: AccountType,
}

impl Account {
    /// Create a new Account
    pub fn new(
        account_number: usize,
        name: String,
        description: String,
        account_type: AccountType,
    ) -> Self {
        Account {
            account_number,
            name,
            description,
            account_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};
    impl Arbitrary for AccountType {
        fn arbitrary(g: &mut Gen) -> Self {
            match u8::arbitrary(g) % 5 {
                0 => AccountType::Asset,
                1 => AccountType::Liability,
                2 => AccountType::Equity,
                3 => AccountType::Revenue,
                _ => AccountType::Expense,
            }
        }
    }

    impl Arbitrary for Account {
        fn arbitrary(g: &mut Gen) -> Self {
            Account::new(
                usize::arbitrary(g) % 10000,
                String::arbitrary(g),
                String::arbitrary(g),
                AccountType::arbitrary(g),
            )
        }
    }
}
