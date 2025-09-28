use crate::chart_of_accounts::ChartOfAccount;

use super::AccountType;
fn bas_class_to_type(class: u8) -> Result<AccountType, String> {
    match class {
        1 => Ok(AccountType::Asset),
        2 => Ok(AccountType::Liability),
        3 => Ok(AccountType::Revenue),
        4..=8 => Ok(AccountType::Expense),
        _ => Err(format!("Invalid BAS class: {}", class)),
    }
}

struct BasClient {}

impl BasClient {
    fn get_current_plan() -> Result<ChartOfAccount, String> {
        Ok(vec![])
    }
}
