use crate::check::{
    utils::{InvalidItem, ValidatorKind},
    Parsed,
};
use regex::Regex;
use std::{collections::HashSet, sync::LazyLock};

// Regex to match import statements with symbol lists: `import {Symbol1, Symbol2} from "...";`
static RE_IMPORT_SYMBOL_LIST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"import\s*\{([^}]+)\}\s+from\s+"[^"]+";"#).unwrap());

// Same but with path captured for fix_source (reconstructing the statement).
static RE_IMPORT_SYMBOL_LIST_WITH_PATH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"import\s*\{([^}]+)\}\s+from\s+"([^"]+)";"#).unwrap());

// Regex to match aliased imports: `import "..." as Alias;`
static RE_IMPORT_ALIAS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"import\s+"[^"]+"\s+as\s+(\w+);"#).unwrap());

#[must_use]
/// Validates that all imported symbols are actually used in the file.
/// Reports unused imports that can be safely removed.
///
/// This validator checks:
/// - Named imports: `import {Symbol1, Symbol2} from "...";`
/// - Aliased imports: `import "..." as Alias;`
/// - Simple imports (`import "...";`) are skipped as we can't determine what symbols they import
///
/// # Panics
///
/// Panics if regex captures are unexpectedly empty (should not happen with valid regex patterns).
pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
    let mut invalid_items: Vec<InvalidItem> = Vec::new();
    let mut imported_symbols: Vec<(String, usize, usize)> = Vec::new(); // (symbol_name, import_start, import_end)
    let mut import_ranges: Vec<(usize, usize)> = Vec::new();

    // First pass: collect all imported symbols and their import statement ranges
    for cap in RE_IMPORT_SYMBOL_LIST.captures_iter(&parsed.src) {
        let m = cap.get(0).unwrap();
        let match_start = m.start();
        let match_end = m.end();
        import_ranges.push((match_start, match_end));

        let symbols_str = cap.get(1).unwrap().as_str();

        // Parse individual symbols (handle aliases like "Symbol as Alias")
        for symbol_part in symbols_str.split(',') {
            let symbol_part = symbol_part.trim();
            if let Some((_symbol, alias)) = symbol_part.split_once(" as ") {
                // Has alias: use the alias name
                imported_symbols.push((alias.trim().to_string(), match_start, match_end));
            } else {
                // No alias: use the symbol name
                imported_symbols.push((symbol_part.to_string(), match_start, match_end));
            }
        }
    }

    // Check for aliased imports: `import "..." as Alias;`
    for cap in RE_IMPORT_ALIAS.captures_iter(&parsed.src) {
        let m = cap.get(0).unwrap();
        let match_start = m.start();
        let match_end = m.end();
        import_ranges.push((match_start, match_end));

        let alias = cap.get(1).unwrap().as_str();
        imported_symbols.push((alias.to_string(), match_start, match_end));
    }

    // Second pass: check if imported symbols are used (excluding the import statements themselves)
    for (symbol_name, import_start, import_end) in imported_symbols {
        // Check if symbol is used outside of import statements
        let is_used = is_symbol_used_excluding_imports(&parsed.src, &symbol_name, &import_ranges);
        if !is_used {
            // Find the symbol within the import statement to get exact location
            let import_text = &parsed.src[import_start..import_end];
            if let Some(relative_pos) = import_text.find(&symbol_name) {
                let offset = import_start + relative_pos;
                let loc = solang_parser::pt::Loc::File(0, offset, offset + symbol_name.len());
                invalid_items.push(InvalidItem::new(
                    ValidatorKind::Import,
                    parsed,
                    loc,
                    format!("Unused import: '{symbol_name}'"),
                ));
            }
        }
    }

    invalid_items
}

/// Checks if a symbol is used in the source code, excluding import statements and comments.
/// This prevents false positives where the symbol appears only in the import line or comments.
/// However, symbols used in `@inheritdoc` `NatSpec` directives are considered as used.
fn is_symbol_used_excluding_imports(
    source: &str,
    symbol: &str,
    import_ranges: &[(usize, usize)],
) -> bool {
    // First, check if symbol is used in @inheritdoc directives (even in comments)
    // Pattern: @inheritdoc followed by optional whitespace and the symbol name
    let inheritdoc_pattern = format!(r"@inheritdoc\s+{}\b", regex::escape(symbol));
    let inheritdoc_re = regex::Regex::new(&inheritdoc_pattern).unwrap();
    if inheritdoc_re.is_match(source) {
        return true; // Symbol is used in @inheritdoc
    }

    // Create a regex pattern that matches the symbol as a whole word
    // This prevents false positives (e.g., "ERC20" matching in "ERC20Token")
    let pattern = format!(r"\b{}\b", regex::escape(symbol));
    let re = regex::Regex::new(&pattern).unwrap();

    // Check all matches and see if any are outside import ranges and comments
    for cap in re.find_iter(source) {
        let match_start = cap.start();
        let match_end = cap.end();

        // Check if this match is within any import statement
        let is_in_import =
            import_ranges.iter().any(|(start, end)| match_start >= *start && match_end <= *end);

        if is_in_import {
            continue; // Skip matches in import statements
        }

        // Check if this match is in a comment
        // Find the line containing this match
        let line_start = source[..match_start].rfind('\n').map_or(0, |i| i + 1);
        let line_end = source[match_start..].find('\n').map_or(source.len(), |i| match_start + i);
        let line = &source[line_start..line_end.min(source.len())];

        // Check if the line is a comment (starts with // or contains /* before the match)
        let line_before_match = &line[..(match_start - line_start).min(line.len())];
        let is_in_comment = line.trim_start().starts_with("//") ||
            line_before_match.contains("/*") ||
            line_before_match.contains("//");

        // If we found a match outside import statements and comments, the symbol is used
        if !is_in_comment {
            return true;
        }
    }

    false
}

/// Returns the source with unused imports removed, or `None` if no changes.
///
/// - `only_remove`: if `Some(set)`, only remove symbols in the set (e.g. fixable from report). If
///   `None`, remove all unused imports.
///
/// # Panics
///
/// Panics if a regex capture group is missing (should not happen with the current patterns).
#[must_use]
#[allow(clippy::implicit_hasher)]
pub fn fix_source(parsed: &Parsed, only_remove: Option<&HashSet<String>>) -> Option<String> {
    let mut import_ranges: Vec<(usize, usize)> = Vec::new();
    for cap in RE_IMPORT_SYMBOL_LIST_WITH_PATH.captures_iter(&parsed.src) {
        let m = cap.get(0).expect("capture 0 always present");
        import_ranges.push((m.start(), m.end()));
    }
    for cap in RE_IMPORT_ALIAS.captures_iter(&parsed.src) {
        let m = cap.get(0).expect("capture 0 always present");
        import_ranges.push((m.start(), m.end()));
    }

    let mut edits: Vec<(usize, usize, String)> = Vec::new();

    // Named imports: `import { A, B } from "path";`
    for cap in RE_IMPORT_SYMBOL_LIST_WITH_PATH.captures_iter(&parsed.src) {
        let m = cap.get(0).expect("capture 0 always present");
        let start = m.start();
        let end = m.end();
        let symbols_str = cap.get(1).expect("capture 1 always present").as_str();
        let path = cap.get(2).expect("capture 2 always present").as_str();

        let mut kept: Vec<&str> = Vec::new();
        for symbol_part in symbols_str.split(',') {
            let symbol_part = symbol_part.trim();
            let name =
                symbol_part.split_once(" as ").map_or(symbol_part, |(_, alias)| alias.trim());
            let should_remove = only_remove.map_or_else(
                || !is_symbol_used_excluding_imports(&parsed.src, name, &import_ranges),
                |set| set.contains(name),
            );
            if !should_remove {
                kept.push(symbol_part);
            }
        }

        if kept.is_empty() {
            edits.push((start, end, String::new()));
        } else if kept.len() < symbol_part_count(symbols_str) {
            let new_list = kept.join(", ");
            edits.push((start, end, format!(r#"import {{ {new_list} }} from "{path}";"#)));
        }
    }

    // Aliased imports: `import "..." as Alias;`
    for cap in RE_IMPORT_ALIAS.captures_iter(&parsed.src) {
        let m = cap.get(0).expect("capture 0 always present");
        let start = m.start();
        let end = m.end();
        let alias = cap.get(1).expect("capture 1 always present").as_str();
        let should_remove = only_remove.map_or_else(
            || !is_symbol_used_excluding_imports(&parsed.src, alias, &import_ranges),
            |set| set.contains(alias),
        );
        if should_remove {
            edits.push((start, end, String::new()));
        }
    }

    if edits.is_empty() {
        return None;
    }

    // Apply from end to start so offsets stay valid.
    edits.sort_by_key(|(s, _e, _r)| std::cmp::Reverse(*s));
    let mut out = parsed.src.clone();
    for (start, end, replacement) in edits {
        out = format!("{}{}{}", &out[..start], replacement, &out[end..]);
    }
    Some(out)
}

fn symbol_part_count(symbols_str: &str) -> usize {
    symbols_str.split(',').filter(|s| !s.trim().is_empty()).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;
    use itertools::Itertools;

    #[test]
    fn test_no_unused_imports() {
        let content = r#"
            import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
            
            contract MyContract {
                ERC20 public token;
                
                function useToken() external {
                    token.transfer(msg.sender, 100);
                }
            }
        "#;

        let expected_findings = ExpectedFindings::new(0);
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_unused_import() {
        let content = r#"
            import {ERC20, IERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
            
            contract MyContract {
                ERC20 public token;
                // IERC20 is imported but never used
            }
        "#;

        let expected_findings = ExpectedFindings {
            script_helper: 1,
            src: 1,
            test_helper: 1,
            test: 1,
            handler: 1,
            script: 1,
        };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_unused_aliased_import() {
        let content = r#"
            import "@openzeppelin/contracts/token/ERC20/ERC20.sol" as OZERC20;
            
            contract MyContract {
                // OZERC20 is imported but never used
            }
        "#;

        let expected_findings = ExpectedFindings {
            script_helper: 1,
            src: 1,
            test_helper: 1,
            test: 1,
            handler: 1,
            script: 1,
        };
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_used_aliased_import() {
        let content = r#"
            import "@openzeppelin/contracts/token/ERC20/ERC20.sol" as OZERC20;
            
            contract MyContract {
                OZERC20 public token;
            }
        "#;

        let expected_findings = ExpectedFindings::new(0);
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_inheritdoc_usage() {
        let content = r#"
            import {IGovernor, Governor} from "@openzeppelin/contracts/governance/Governor.sol";
            
            abstract contract MyGovernor is Governor {
                /// @inheritdoc IGovernor
                function hasVoted(uint256 proposalId, address account) public view override returns (bool) {
                    return false;
                }
            }
        "#;

        let expected_findings = ExpectedFindings::new(0);
        expected_findings.assert_eq(content, &validate);
    }

    #[test]
    fn test_inheritdoc_with_unused_import() {
        let content = r#"
            import {IGovernor, Governor, IERC20} from "@openzeppelin/contracts/governance/Governor.sol";
            
            abstract contract MyGovernor is Governor {
                /// @inheritdoc IGovernor
                function hasVoted(uint256 proposalId, address account) public view override returns (bool) {
                    return false;
                }
                // IERC20 is imported but never used (not even in @inheritdoc)
            }
        "#;

        let expected_findings = ExpectedFindings {
            script_helper: 1,
            src: 1,
            test_helper: 1,
            test: 1,
            handler: 1,
            script: 1,
        };
        expected_findings.assert_eq(content, &validate);
    }

    fn parsed_from_src(content: &str) -> crate::check::Parsed {
        use crate::check::{comments::Comments, inline_config::InlineConfig};
        use std::path::PathBuf;

        let (pt, comments) = crate::parser::parse_solidity(content, 0).expect("parse");
        let comments = Comments::new(comments, content);
        let (inline_config_items, invalid_inline_config_items): (Vec<_>, Vec<_>) =
            comments.parse_inline_config_items().partition_result();
        let inline_config = InlineConfig::new(inline_config_items, content);
        crate::check::Parsed {
            file: PathBuf::from("./src/Contract.sol"),
            src: content.to_string(),
            pt,
            comments,
            inline_config,
            invalid_inline_config_items,
            file_config: crate::check::file_config::FileConfig::default(),
            path_config: crate::foundry_config::CheckPaths::default(),
        }
    }

    #[test]
    fn test_fix_source_removes_unused_from_named_import() {
        let content = r#"import {ERC20, IERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MyContract {
    ERC20 public token;
}
"#;
        let parsed = parsed_from_src(content);
        let fixed = fix_source(&parsed, None).unwrap();
        assert!(
            fixed.contains(
                r#"import { ERC20 } from "@openzeppelin/contracts/token/ERC20/ERC20.sol";"#
            ),
            "expected single used symbol in import, got: {fixed:?}"
        );
        assert!(!fixed.contains("IERC20"));
    }

    #[test]
    fn test_fix_source_removes_whole_aliased_import() {
        let content = r#"import "@openzeppelin/contracts/token/ERC20/ERC20.sol" as OZERC20;

contract MyContract {
}
"#;
        let parsed = parsed_from_src(content);
        let fixed = fix_source(&parsed, None).unwrap();
        assert!(!fixed.contains("OZERC20"));
        assert!(!fixed.contains("as OZERC20"));
    }

    #[test]
    fn test_fix_source_no_change_when_all_used() {
        let content = r#"import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract MyContract {
    ERC20 public token;
}
"#;
        let parsed = parsed_from_src(content);
        let fixed = fix_source(&parsed, None);
        assert!(fixed.is_none());
    }
}
