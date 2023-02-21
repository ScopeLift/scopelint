// The `.extension()` method only looks after the last dot in the file name, so it will return
// Some("sol") for both "Foo.sol" and "Foo.t.sol". This is not what we want here, so we just check
// extensions manually with `ends_with`.
#![allow(clippy::case_sensitive_file_extension_comparisons)]

use solang_parser::pt::{
    FunctionAttribute, FunctionDefinition, FunctionTy, SourceUnit, Visibility,
};
use std::path::Path;

/// The type of validator that found the invalid item.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ValidatorKind {
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
    kind: ValidatorKind,
    file: String, // File name.
    text: String, // Details to show about the invalid item.
    line: usize,  // Line number.
}

impl InvalidItem {
    #[must_use]
    /// Creates a new `InvalidItem`.
    pub const fn new(kind: ValidatorKind, file: String, text: String, line: usize) -> Self {
        Self { kind, file, text, line }
    }

    #[must_use]
    /// Returns a string describing the invalid item, which is shown to the user so they can triage
    /// findings.
    pub fn description(&self) -> String {
        match self.kind {
            ValidatorKind::Test => {
                format!("Invalid test name in {} on line {}: {}", self.file, self.line, self.text)
            }
            ValidatorKind::Constant => {
                format!(
                    "Invalid constant or immutable name in {} on line {}: {}",
                    self.file, self.line, self.text
                )
            }
            ValidatorKind::Script => {
                format!("Invalid script interface in {}: {}", self.file, self.text)
            }
            ValidatorKind::Src => {
                format!(
                    "Invalid src method name in {} on line {}: {}",
                    self.file, self.line, self.text
                )
            }
        }
    }
}

/// Categories of file kinds found in forge projects.
/// Two additional file kinds are not included here: `ScriptHelpers` and `TestHelpers`. These are
/// not currently used in any checks so they are excluded for now.
pub enum FileKind {
    /// Executable script files live in the `scripts` directory and end with `.s.sol`.
    Script,
    /// Core contracts live in the `src` directory and end with `.sol`.
    Src,
    /// Contracts with test methods live in the `test` directory and end with `.t.sol`.
    Test,
}

/// Provides a method to check if a file is of a given kind.
pub trait IsFileKind {
    /// Returns `true` if the file is of the given kind, `false` otherwise.
    fn is_file_kind(&self, kind: FileKind) -> bool;
}

impl IsFileKind for Path {
    fn is_file_kind(&self, kind: FileKind) -> bool {
        let path = self.to_str().unwrap();
        match kind {
            FileKind::Script => path.starts_with("./script") && path.ends_with(".s.sol"),
            FileKind::Src => path.starts_with("./src") && path.ends_with(".sol"),
            FileKind::Test => path.starts_with("./test") && path.ends_with(".t.sol"),
        }
    }
}

/// Provides a method to return the name of a function.
pub trait Name {
    /// Returns the name of the function for standard functions, or `constructor`, `fallback` or
    /// `receive` for other function types.
    fn name(&self) -> String;
}

/// Provides methods to return visibility information about a function.
pub trait VisibilitySummary {
    /// Returns `true` if the function is internal or private, `false` otherwise.
    fn is_internal_or_private(&self) -> bool;
    /// Returns `true` if the function is public or external, `false` otherwise.
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

#[must_use]
/// Converts the start offset of a `Loc` to `(line, col)`. Modified from <https://github.com/foundry-rs/foundry/blob/45b9dccdc8584fb5fbf55eb190a880d4e3b0753f/fmt/src/helpers.rs#L54-L70>
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

#[derive(Default)]
/// Given the number of expected findings for each file kind, this struct makes it easy to assert
/// the true number of findings for each file kind by calling it's `assert_eq` method.
pub struct ExpectedFindings {
    /// The number of expected findings for script helper contracts.
    pub script_helper: usize,
    /// The number of expected findings for script contracts.
    pub script: usize,
    /// The number of expected findings for source contracts.
    pub src: usize,
    /// The number of expected findings for test helper contracts.
    pub test_helper: usize,
    /// The number of expected findings for test contracts.
    pub test: usize,
}

type ValidatorFn = dyn Fn(&Path, &str, &SourceUnit) -> Vec<InvalidItem>;

impl ExpectedFindings {
    #[must_use]
    /// Creates a new `ExpectedFindings` with the given number of expected findings for each file
    /// kind. Use this when a validator applies to all file kinds. If a validator only applies to
    /// certain file kinds, you should initialize it using the form
    /// `ExpectedFindings { test: 3, ..ExpectedFindings::default() }`.
    pub const fn new(expected_findings: usize) -> Self {
        Self {
            script_helper: expected_findings,
            script: expected_findings,
            src: expected_findings,
            test_helper: expected_findings,
            test: expected_findings,
        }
    }

    /// Asserts that the number of invalid items found by the validator is equal to the expected
    /// number for the given content, for each file kind.
    /// # Panics
    /// In practice this should not panic unless one of validations fails.
    pub fn assert_eq(&self, content: &str, validate: &ValidatorFn) {
        let (pt, _comments) = solang_parser::parse(content, 0).expect("Parsing failed");

        let invalid_items_script_helper =
            validate(Path::new("./script/MyContract.sol"), content, &pt);
        let invalid_items_script = validate(Path::new("./script/MyContract.s.sol"), content, &pt);
        let invalid_items_src = validate(Path::new("./src/MyContract.sol"), content, &pt);
        let invalid_items_test_helper = validate(Path::new("./test/MyContract.sol"), content, &pt);
        let invalid_items_test = validate(Path::new("./test/MyContract.t.sol"), content, &pt);

        assert_eq!(invalid_items_script_helper.len(), self.script_helper);
        assert_eq!(invalid_items_script.len(), self.script);
        assert_eq!(invalid_items_src.len(), self.src);
        assert_eq!(invalid_items_test_helper.len(), self.test_helper);
        assert_eq!(invalid_items_test.len(), self.test);
    }
}
