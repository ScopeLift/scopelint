use solang_parser::pt::{FunctionAttribute, FunctionDefinition, FunctionTy, Visibility};
use std::path::Path;

/// The type of validator that found the invalid item.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
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

    pub fn description(&self) -> String {
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

pub enum FileKind {
    ScriptContracts,
    SrcContracts,
    TestContracts,
}

pub trait IsFileKind {
    fn is_file_kind(&self, kind: FileKind) -> bool;
}

impl IsFileKind for Path {
    fn is_file_kind(&self, kind: FileKind) -> bool {
        let path = self.to_str().unwrap();
        match kind {
            // Executable script files are expected to end with `.s.sol`, whereas non-executable
            // helper contracts in the scripts dir just end with `.sol`.
            FileKind::ScriptContracts => path.starts_with("./script") && path.ends_with(".s.sol"),
            FileKind::SrcContracts => path.starts_with("./src") && path.ends_with(".sol"),
            // Contracts with test methods are expected to end with `.t.sol`, whereas e.g. mocks and
            // helper contracts in the test dir just end with `.sol`.
            FileKind::TestContracts => path.starts_with("./test") && path.ends_with(".t.sol"),
        }
    }
}

pub trait Name {
    fn name(&self) -> String;
}

pub trait VisibilitySummary {
    fn is_internal_or_private(&self) -> bool;
    fn is_public_or_external(&self) -> bool;
}

impl Name for FunctionDefinition {
    fn name(&self) -> String {
        match self.ty {
            FunctionTy::Constructor => "constructor".to_string(),
            FunctionTy::Fallback => "fallback".to_string(),
            FunctionTy::Receive => "receive".to_string(),
            FunctionTy::Function | FunctionTy::Modifier => self.name.as_ref().unwrap().name.clone(),
        }
    }
}

impl VisibilitySummary for FunctionDefinition {
    fn is_internal_or_private(&self) -> bool {
        self.attributes.iter().any(|a| match a {
            FunctionAttribute::Visibility(v) => {
                matches!(v, Visibility::Private(_) | Visibility::Internal(_))
            }
            _ => false,
        })
    }

    fn is_public_or_external(&self) -> bool {
        self.attributes.iter().any(|a| match a {
            FunctionAttribute::Visibility(v) => {
                matches!(v, Visibility::Public(_) | Visibility::External(_))
            }
            _ => false,
        })
    }
}

// Converts the start offset of a `Loc` to `(line, col)`. Modified from https://github.com/foundry-rs/foundry/blob/45b9dccdc8584fb5fbf55eb190a880d4e3b0753f/fmt/src/helpers.rs#L54-L70
#[must_use]
pub fn offset_to_line(content: &str, start: usize) -> usize {
    debug_assert!(content.len() > start);

    let mut line_counter = 1; // First line is `1`.
    for (offset, c) in content.chars().enumerate() {
        if c == '\n' {
            line_counter += 1;
        }
        if offset > start {
            return line_counter
        }
    }

    unreachable!("content.len() > start")
}
