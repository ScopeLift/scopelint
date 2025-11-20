use crate::check::{
    utils::{InvalidItem, ValidatorKind},
    Parsed,
};
use regex::Regex;
use std::sync::LazyLock;

// Regex to match import statements with symbol lists: `import {Symbol1, Symbol2} from "...";`
static RE_IMPORT_SYMBOL_LIST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"import\s*\{([^}]+)\}\s+from\s+"[^"]+";"#).unwrap());

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
fn is_symbol_used_excluding_imports(
    source: &str,
    symbol: &str,
    import_ranges: &[(usize, usize)],
) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::utils::ExpectedFindings;

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
}
