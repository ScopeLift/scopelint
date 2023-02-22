use super::utils::InvalidItem;
use std::fmt;

/// A collection of invalid items to generate a report from.
#[derive(Default)]
pub struct Report {
    /// A list of invalid items.
    invalid_items: Vec<InvalidItem>,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sorted_items = self.invalid_items.clone();
        sorted_items.sort();

        for item in sorted_items {
            writeln!(f, "{}", item.description())?;
        }
        Ok(())
    }
}

impl Report {
    /// Extends the report with a list of invalid items.
    pub fn add_items(&mut self, items: Vec<InvalidItem>) {
        self.invalid_items.extend(items);
    }

    /// Returns true if no issues were found.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.invalid_items.is_empty()
    }
}
