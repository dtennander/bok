use crate::{Entry, Side};

impl Entry {
    /// Prints the Entry in a short format, often used in log.
    ///
    /// Example:
    ///
    /// 2025-05-01: My Title
    /// A long description of what happened.
    ///
    ///            debit  |  credit
    /// account 1     100 |           # Short description.
    /// account 2         |     100   # Second description
    ///
    pub fn show_short(&self) -> String {
        match self {
            Entry::Origin { year } => {
                format!("----------------{}---------------", year)
            }
            Entry::Entry {
                name,
                description,
                lines,
                ..
            } => {
                let mut result = String::new();
                result.push_str(name);
                result.push('\n');
                result.push_str(description);
                result.push('\n');
                result.push('\n');

                result.push_str(&format!("{: >10} {:>10} | {:>10}", "", "debit", "credit"));
                result.push('\n');
                for line in lines {
                    let (debit, credit) = if line.side == Side::Debit {
                        (line.amount.to_string(), "".to_string())
                    } else {
                        ("".to_string(), line.amount.to_string())
                    };
                    result.push_str(&format!(
                        "{: >10} {:>10} | {:>10}",
                        line.account, debit, credit
                    ));
                    if let Some(description) = &line.description {
                        result.push_str(&format!("# {}", description));
                    }
                    result.push('\n');
                }

                result
            }
        }
    }
}
