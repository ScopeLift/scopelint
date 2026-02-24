use crate::check::{
    utils::{FileKind, InvalidItem, IsFileKind, ValidatorKind},
    Parsed,
};
/// Check if a file is a source file
fn is_matching_file(parsed: &Parsed) -> bool {
    parsed.file.is_file_kind(FileKind::Src, &parsed.path_config)
}

#[must_use]
/// Validates that source files have SPDX license headers.
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    if !is_matching_file(parsed) {
        return Vec::new();
    }

    let mut invalid_items: Vec<InvalidItem> = Vec::new();

    // Check if SPDX header is present
    if find_spdx_header(&parsed.src).is_none() {
        // Create a simple location for file-level issues
        let loc = solang_parser::pt::Loc::File(0, 0, 0);
        invalid_items.push(InvalidItem::new(
            ValidatorKind::Src,
            parsed,
            loc,
            "Missing SPDX-License-Identifier header".to_string(),
        ));
    }

    invalid_items
}

/// Check if a line is a comment line
fn is_comment_line(line: &str) -> bool {
    line.starts_with("//") || line.starts_with("/*")
}

/// Check if a line contains a valid SPDX header
fn has_spdx_header(line: &str) -> bool {
    line.starts_with("// SPDX-License-Identifier:")
}

/// Find SPDX header in header section
fn find_spdx_header(src: &str) -> Option<&str> {
    for line in src.lines() {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Check if this comment line has SPDX
        if is_comment_line(trimmed) && has_spdx_header(trimmed) {
            return Some(trimmed);
        }

        // If we hit any non-comment content, stop looking
        if !is_comment_line(trimmed) {
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;

    #[test]
    fn test_validate() {
        let content = r"
            // SPDX-License-Identifier: MIT
            pragma solidity ^0.8.17;
            
            contract Test {
                uint256 public number;
            }
        ";

        ExpectedFindings::new(0).assert_eq(content, &validate);
    }

    #[test]
    fn test_validate_missing_spdx() {
        let content = r"
            pragma solidity ^0.8.17;
            
            contract Test {
                uint256 public number;
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_validate_comment_then_spdx() {
        let content = r"
            // This is a comment
            // SPDX-License-Identifier: MIT
            pragma solidity ^0.8.17;
            
            contract Test {
                uint256 public number;
            }
        ";

        ExpectedFindings::new(0).assert_eq(content, &validate);
    }

    #[test]
    fn test_validate_pragma_then_spdx() {
        let content = r"
            pragma solidity ^0.8.17;
            // SPDX-License-Identifier: MIT
            
            contract Test {
                uint256 public number;
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_validate_comment_then_pragma() {
        let content = r"
            // This is a comment
            pragma solidity ^0.8.17;
            
            contract Test {
                uint256 public number;
            }
        ";

        let expected_findings = ExpectedFindings { src: 1, ..ExpectedFindings::default() };
        expected_findings.assert_eq(content, &validate);
    }
}
