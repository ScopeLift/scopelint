use super::utils::InvalidItem;
use itertools::Itertools;
use std::fmt;

/// A collection of invalid items to generate a report from.
#[derive(Default)]
pub struct Report {
    /// A list of invalid items.
    invalid_items: Vec<InvalidItem>,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        self.invalid_items
            .iter()
            .filter(|item| !item.is_disabled)
            .sorted_unstable()
            .try_for_each(|item| writeln!(f, "{}", item.description()))
    }
}

impl Report {
    /// Extends the report with the invalid item.
    pub fn add_item(&mut self, item: InvalidItem) {
        self.invalid_items.push(item);
    }

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
