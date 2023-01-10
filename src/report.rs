use std::fmt;

/// The type of validator that found the invalid item.
pub enum Validator {
    /// A constant or immutable variable.
    Constant,
    /// A script file.
    Script,
    /// A source contract.
    Src,
    /// A test contract.
    Test,
}

/// A single invalid item found by a validator.
pub struct InvalidItem {
    kind: Validator,
    file: String, // File name.
    text: String, // Details to show about the invalid item.
    line: usize,  // Line number.
}

impl InvalidItem {
    /// Initializes a new `InvalidItem`.
    #[must_use]
    pub const fn new(kind: Validator, file: String, text: String, line: usize) -> Self {
        Self { kind, file, text, line }
    }

    fn description(&self) -> String {
        match self.kind {
            Validator::Test => {
                format!("Invalid test name in {} on line {}: {}", self.file, self.line, self.text)
            }
            Validator::Constant => {
                format!(
                    "Invalid constant or immutable name in {} on line {}: {}",
                    self.file, self.line, self.text
                )
            }
            Validator::Script => {
                format!("Invalid script interface in {}: {}", self.file, self.text)
            }
            Validator::Src => {
                format!(
                    "Invalid src method name in {} on line {}: {}",
                    self.file, self.line, self.text
                )
            }
        }
    }
}

/// A collection of invalid items to generate a report from.
#[derive(Default)]
pub struct Report {
    /// A list of invalid items.
    invalid_items: Vec<InvalidItem>,
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.invalid_items {
            writeln!(f, "{}", item.description())?;
        }
        Ok(())
    }
}

impl Report {
    /// Extends the report with a new invalid item.
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
