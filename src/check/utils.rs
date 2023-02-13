use crate::check::report;
use solang_parser::pt::{FunctionDefinition, FunctionTy};
use std::{
    error::Error,
    ffi::OsStr,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub trait Validate {
    fn validate(&self, content: &str, file: &Path) -> Option<report::InvalidItem>;
}

pub trait Name {
    fn name(&self) -> String;
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
pub enum FileKind {
    TestContracts,
}

pub fn get_files(kind: &FileKind) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    let paths = ["./src", "./script", "./test"];

    for path in paths {
        for result in WalkDir::new(path) {
            let dent = match result {
                Ok(dent) => dent,
                Err(err) => {
                    eprintln!("{err}");
                    continue
                }
            };

            if !dent.file_type().is_file() || dent.path().extension() != Some(OsStr::new("sol")) {
                continue
            }

            match kind {
                FileKind::TestContracts => {
                    if path == "./test" && dent.path().to_str().unwrap().ends_with(".t.sol") {
                        files.push(dent.path().to_path_buf());
                    }
                }
            }
        }
    }

    Ok(files)
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
