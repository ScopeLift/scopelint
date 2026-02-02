use once_cell::sync::Lazy;
use regex::Regex;
use solang_parser::{
    diagnostics::Diagnostic,
    pt::{Comment, SourceUnit},
};

static TRANSIENT_KEYWORD: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\btransient\b").expect("transient regex is valid"));
const TRANSIENT_REPLACEMENT: &str = "         ";

/// Parses Solidity source code, with a fallback that strips unsupported transient keywords.
///
/// This keeps byte offsets stable by replacing `transient` with same-length whitespace, so
/// comment and inline-config locations remain aligned with the original source.
pub fn parse_solidity(
    src: &str,
    file_no: usize,
) -> Result<(SourceUnit, Vec<Comment>), Vec<Diagnostic>> {
    match solang_parser::parse(src, file_no) {
        Ok(result) => Ok(result),
        Err(errs) => {
            if !src.contains("transient") {
                return Err(errs)
            }

            let sanitized = strip_transient(src);
            match solang_parser::parse(&sanitized, file_no) {
                Ok(result) => Ok(result),
                Err(_) => Err(errs),
            }
        }
    }
}

fn strip_transient(src: &str) -> String {
    if !src.contains("transient") {
        return src.to_string()
    }

    TRANSIENT_KEYWORD
        .replace_all(src, TRANSIENT_REPLACEMENT)
        .into_owned()
}
