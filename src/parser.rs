use solang_parser::{
    diagnostics::Diagnostic,
    pt::{Comment, SourceUnit},
};

/// Parses Solidity source code, with a fallback that strips unsupported keywords (e.g.
/// `transient`).
///
/// This keeps byte offsets stable by replacing keywords with same-length whitespace, so
/// comment and inline-config locations remain aligned with the original source.
/// To add more preprocessing, extend `sanitize()`.
///
/// Returns the parse tree, comments, and the byte offsets of any `transient` keywords that
/// were blanked during preprocessing (empty when no preprocessing was needed).
///
/// # Errors
///
/// Returns the parser diagnostics when the source cannot be parsed (even after preprocessing).
pub fn parse_solidity(
    src: &str,
    file_no: usize,
) -> Result<(SourceUnit, Vec<Comment>, Vec<usize>), Vec<Diagnostic>> {
    match solang_parser::parse(src, file_no) {
        Ok((pt, comments)) => Ok((pt, comments, Vec::new())),
        Err(errs) => {
            let (sanitized, offsets) = sanitize(src);
            if sanitized == src {
                return Err(errs);
            }
            solang_parser::parse(&sanitized, file_no)
                .map(|(pt, comments)| (pt, comments, offsets))
                .map_or(Err(errs), Ok)
        }
    }
}

/// Preprocesses source so the parser can accept it. Add any future strip logic here.
fn sanitize(src: &str) -> (String, Vec<usize>) {
    strip_transient(src)
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ScanState {
    Code,
    LineComment,
    BlockComment,
    SingleQuote,
    DoubleQuote,
}

const fn is_identifier_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn strip_transient(src: &str) -> (String, Vec<usize>) {
    if !src.contains("transient") {
        return (src.to_string(), Vec::new());
    }

    let bytes = src.as_bytes();
    let len = bytes.len();
    let mut out = bytes.to_vec();
    let mut offsets: Vec<usize> = Vec::new();
    let keyword = b"transient";
    let kw_len = keyword.len();
    let mut idx = 0;
    let mut state = ScanState::Code;

    while idx < len {
        match state {
            ScanState::Code => {
                if idx + kw_len <= len
                    && bytes[idx..idx + kw_len] == *keyword
                    && (idx == 0 || !is_identifier_char(bytes[idx - 1]))
                    && (idx + kw_len == len || !is_identifier_char(bytes[idx + kw_len]))
                {
                    offsets.push(idx);
                    out[idx..idx + kw_len].fill(b' ');
                    idx += kw_len;
                    continue;
                }
                match bytes[idx] {
                    b'/' if idx + 1 < len && bytes[idx + 1] == b'/' => {
                        state = ScanState::LineComment;
                        idx += 2;
                    }
                    b'/' if idx + 1 < len && bytes[idx + 1] == b'*' => {
                        state = ScanState::BlockComment;
                        idx += 2;
                    }
                    b'\'' => {
                        state = ScanState::SingleQuote;
                        idx += 1;
                    }
                    b'"' => {
                        state = ScanState::DoubleQuote;
                        idx += 1;
                    }
                    _ => idx += 1,
                }
            }
            ScanState::LineComment => {
                if bytes[idx] == b'\n' {
                    state = ScanState::Code;
                }
                idx += 1;
            }
            ScanState::BlockComment => {
                if bytes[idx] == b'*' && idx + 1 < len && bytes[idx + 1] == b'/' {
                    state = ScanState::Code;
                    idx += 2;
                } else {
                    idx += 1;
                }
            }
            ScanState::SingleQuote => {
                if bytes[idx] == b'\\' {
                    idx = (idx + 2).min(len);
                } else {
                    if bytes[idx] == b'\'' {
                        state = ScanState::Code;
                    }
                    idx += 1;
                }
            }
            ScanState::DoubleQuote => {
                if bytes[idx] == b'\\' {
                    idx = (idx + 2).min(len);
                } else {
                    if bytes[idx] == b'"' {
                        state = ScanState::Code;
                    }
                    idx += 1;
                }
            }
        }
    }

    (String::from_utf8(out).expect("same-length replacement preserves UTF-8"), offsets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solang_parser::pt::SourceUnitPart;

    #[test]
    fn test_parse_with_transient() {
        let src = r#"
contract C {
    uint128 transient b;
}
"#;
        let result = parse_solidity(src, 0);
        assert!(
            result.is_ok(),
            "Solidity with transient keyword should parse (with or without fallback): {:?}",
            result.err()
        );
        let (pt, _, offsets) = result.unwrap();
        assert_eq!(pt.0.len(), 1);
        assert!(matches!(&pt.0[0], SourceUnitPart::ContractDefinition(_)));
        assert_eq!(offsets.len(), 1, "should record one blanked transient offset");
    }

    #[test]
    fn test_strip_transient_skips_string_literal() {
        let src = r#"string constant NAME = "transient"; uint128 transient b;"#;
        let (sanitized, offsets) = strip_transient(src);
        assert!(
            sanitized.contains(r#""transient""#),
            "transient inside a string literal must not be blanked"
        );
        assert_eq!(offsets.len(), 1, "only the keyword occurrence should be recorded");
    }

    #[test]
    fn test_strip_transient_skips_line_comment() {
        let src = "// transient storage\nuint128 transient b;";
        let (sanitized, offsets) = strip_transient(src);
        assert!(
            sanitized.starts_with("// transient"),
            "transient inside a line comment must not be blanked"
        );
        assert_eq!(offsets.len(), 1);
    }

    #[test]
    fn test_strip_transient_skips_block_comment() {
        let src = "/* transient */ uint128 transient b;";
        let (sanitized, offsets) = strip_transient(src);
        assert!(
            sanitized.starts_with("/* transient"),
            "transient inside a block comment must not be blanked"
        );
        assert_eq!(offsets.len(), 1);
    }
}
