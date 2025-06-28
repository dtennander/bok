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
    pub fn show(&self) -> String {
        match self {
            Entry::Origin { year } => {
                format!("----------------{}---------------\n", year)
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

    /// Prints the Entry in a short, one-line format: 'My Title, a long description cut off after this long...'.
    /// Description is truncated if too long.
    pub fn show_short(&self) -> String {
        match self {
            Entry::Origin { year } => {
                format!("----------------{}---------------\n", year)
            }
            Entry::Entry {
                name, description, ..
            } => {
                let max_len = 60;
                let desc = if description.len() > max_len {
                    let mut d = description.chars().take(max_len).collect::<String>();
                    d.push_str("...");
                    d
                } else {
                    description.clone()
                };
                format!("{}, {}\n", name, desc)
            }
        }
    }
}
