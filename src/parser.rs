use regex::Regex;
use solang_parser::{
    diagnostics::Diagnostic,
    pt::{Comment, SourceUnit},
};
use std::sync::LazyLock;

static TRANSIENT_KEYWORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\btransient\b").expect("transient regex is valid"));
const TRANSIENT_REPLACEMENT: &str = "         ";

/// Parses Solidity source code, with a fallback that strips unsupported keywords (e.g.
/// `transient`).
///
/// This keeps byte offsets stable by replacing keywords with same-length whitespace, so
/// comment and inline-config locations remain aligned with the original source.
/// To add more preprocessing, extend `sanitize()`.
///
/// # Errors
///
/// Returns the parser diagnostics when the source cannot be parsed (even after preprocessing).
pub fn parse_solidity(
    src: &str,
    file_no: usize,
) -> Result<(SourceUnit, Vec<Comment>), Vec<Diagnostic>> {
    match solang_parser::parse(src, file_no) {
        Ok(result) => Ok(result),
        Err(errs) => {
            let sanitized = sanitize(src);
            if sanitized == src {
                return Err(errs);
            }
            solang_parser::parse(&sanitized, file_no).map_or(Err(errs), Ok)
        }
    }
}

/// Preprocesses source so the parser can accept it. Add any future strip logic here.
fn sanitize(src: &str) -> String {
    strip_transient(src)
}

fn strip_transient(src: &str) -> String {
    if !src.contains("transient") {
        return src.to_string();
    }

    TRANSIENT_KEYWORD.replace_all(src, TRANSIENT_REPLACEMENT).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use solang_parser::pt::SourceUnitPart;

    #[test]
    fn test_parse_with_transient() {
        let src = r"
contract C {
    uint128 transient b;
}
";
        let result = parse_solidity(src, 0);
        assert!(
            result.is_ok(),
            "Solidity with transient keyword should parse (with or without fallback): {:?}",
            result.err()
        );
        let (pt, _) = result.unwrap();
        assert_eq!(pt.0.len(), 1);
        assert!(matches!(&pt.0[0], SourceUnitPart::ContractDefinition(_)));
    }
}
