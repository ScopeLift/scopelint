// Parses inline configuration directives from Solidity comments. This module is based on the
// [`forgefmt` comments module](https://github.com/foundry-rs/foundry/blob/675a824648b6259c1566009b79356a4fc109b6ac/crates/fmt/src/inline_config.rs).
//
// Rule-specific ignore directives:
// - `// scopelint: ignore-error` - ignores next item for error_prefix validator
// - `// scopelint: ignore-error-line` - ignores current line
// - `// scopelint: ignore-error-next-line` - ignores next line
// - `// scopelint: ignore-error-start` / `// scopelint: ignore-error-end` - ignore a region
// - `// scopelint: ignore-error-file` - ignores entire file for error_prefix validator
//
// Supported rules: error, import, variable, constant, test, script, src, eip712

// We disable clippy in this file to keep this file as close to the original as possible, so it's
// easier to merge in upstream changes.
#![allow(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
use crate::check::{
    comments::{CommentState, CommentStringExt},
    utils::ValidatorKind,
};
use itertools::Itertools;
use solang_parser::pt::Loc;
use std::{fmt, str::FromStr};

/// An inline config item
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
pub enum InlineConfigItem {
    /// Disables the next code item regardless of newlines
    DisableNextItem,
    /// Disables formatting on the current line
    DisableLine,
    /// Disables formatting between the next newline and the newline after
    DisableNextLine,
    /// Disables formatting for any code that follows this and before the next "disable-end"
    DisableStart,
    /// Disables formatting for any code that precedes this and after the previous "disable-start"
    DisableEnd,
    /// Ignores the next code item for linting rules
    IgnoreNextItem,
    /// Ignores the current line for linting rules
    IgnoreLine,
    /// Ignores the next line for linting rules
    IgnoreNextLine,
    /// Ignores linting rules for any code that follows this and before the next "ignore-end"
    IgnoreStart,
    /// Ignores linting rules for any code that precedes this and after the previous "ignore-start"
    IgnoreEnd,
    /// Rule-specific ignore directives (e.g., "ignore-error" for error_prefix validator)
    IgnoreRule {
        /// The validator kind to ignore
        kind: ValidatorKind,
        /// The scope of the ignore (next-item, line, next-line, start, end)
        scope: RuleIgnoreScope,
    },
}

impl FromStr for InlineConfigItem {
    type Err = InvalidInlineConfigItem;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Check for rule-specific ignore directives (e.g., "ignore-error", "ignore-error-line")
        if let Some(rest) = s.strip_prefix("ignore-") {
            // Try to parse as rule-specific ignore
            // Check for "ignore-<rule>-file" first (before splitting on '-')
            if rest.ends_with("-file") {
                if let Some(rule) = rest.strip_suffix("-file") {
                    if let Some(kind) = parse_rule_name(rule) {
                        return Ok(InlineConfigItem::IgnoreRule {
                            kind,
                            scope: RuleIgnoreScope::File,
                        });
                    }
                }
            }
            // Then check for other scopes like "ignore-<rule>-next-item"
            if let Some((rule, scope_str)) = rest.split_once('-') {
                if let Some(kind) = parse_rule_name(rule) {
                    let scope = match scope_str {
                        "next-item" => RuleIgnoreScope::NextItem,
                        "line" => RuleIgnoreScope::Line,
                        "next-line" => RuleIgnoreScope::NextLine,
                        "start" => RuleIgnoreScope::Start,
                        "end" => RuleIgnoreScope::End,
                        _ => return Err(InvalidInlineConfigItem(s.into())),
                    };
                    return Ok(InlineConfigItem::IgnoreRule { kind, scope });
                }
            }
            // Check if it's just "ignore-<rule>" (defaults to next-item scope for better usability)
            if let Some(kind) = parse_rule_name(rest) {
                return Ok(InlineConfigItem::IgnoreRule { kind, scope: RuleIgnoreScope::NextItem });
            }
        }

        // Generic directives
        Ok(match s {
            "disable-next-item" => InlineConfigItem::DisableNextItem,
            "disable-line" => InlineConfigItem::DisableLine,
            "disable-next-line" => InlineConfigItem::DisableNextLine,
            "disable-start" => InlineConfigItem::DisableStart,
            "disable-end" => InlineConfigItem::DisableEnd,
            "ignore-next-item" => InlineConfigItem::IgnoreNextItem,
            "ignore-line" => InlineConfigItem::IgnoreLine,
            "ignore-next-line" => InlineConfigItem::IgnoreNextLine,
            "ignore-start" => InlineConfigItem::IgnoreStart,
            "ignore-end" => InlineConfigItem::IgnoreEnd,
            s => return Err(InvalidInlineConfigItem(s.into())),
        })
    }
}

/// The scope of a rule-specific ignore directive
#[derive(Debug, Clone, Copy)]
pub enum RuleIgnoreScope {
    /// Ignores the next code item
    NextItem,
    /// Ignores the current line
    Line,
    /// Ignores the next line
    NextLine,
    /// Starts an ignore region
    Start,
    /// Ends an ignore region
    End,
    /// Ignores the entire file for this rule
    File,
}

/// Maps a rule name (e.g., "error") to a `ValidatorKind`
fn parse_rule_name(rule: &str) -> Option<ValidatorKind> {
    match rule {
        "error" => Some(ValidatorKind::Error),
        "import" => Some(ValidatorKind::Import),
        "variable" => Some(ValidatorKind::Variable),
        "constant" => Some(ValidatorKind::Constant),
        "test" => Some(ValidatorKind::Test),
        "script" => Some(ValidatorKind::Script),
        "src" => Some(ValidatorKind::Src),
        "eip712" => Some(ValidatorKind::Eip712),
        _ => None,
    }
}

#[derive(Debug)]
pub struct InvalidInlineConfigItem(String);

impl fmt::Display for InvalidInlineConfigItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("Invalid inline config item: {}", self.0))
    }
}

/// A disabled formatting range. `loose` designates that the range includes any loc which
/// may start in between start and end, whereas the strict version requires that
/// `range.start >= loc.start <=> loc.end <= range.end`
#[derive(Debug)]
struct DisabledRange {
    start: usize,
    end: usize,
    loose: bool,
}

impl DisabledRange {
    fn includes(&self, loc: Loc) -> bool {
        if self.loose {
            // For loose ranges, check if location starts within the range
            loc.start() >= self.start && loc.start() < self.end
        } else {
            // For strict ranges, check if entire location is within the range
            loc.start() >= self.start && loc.end() <= self.end
        }
    }
}

/// An ignored formatting range. `loose` designates that the range includes any loc which
/// may start in between start and end, whereas the strict version requires that
/// `range.start >= loc.start <=> loc.end <= range.end`
#[derive(Debug)]
struct IgnoredRange {
    start: usize,
    end: usize,
    loose: bool,
}

impl IgnoredRange {
    fn includes(&self, loc: Loc) -> bool {
        if self.loose {
            // For loose ranges, check if location starts within the range [start, end)
            loc.start() >= self.start && loc.start() < self.end
        } else {
            // For strict ranges, check if entire location is within the range [start, end]
            loc.start() >= self.start && loc.end() <= self.end
        }
    }
}

/// This is a list of Inline Config items for locations in a source file. This is acquired by
/// parsing the comments for `scopelint:` items. See [`Comments::parse_inline_config_items`] for
/// details.
#[derive(Default, Debug)]
pub struct InlineConfig {
    disabled_ranges: Vec<DisabledRange>,
    ignored_ranges: Vec<IgnoredRange>,
    /// Rule-specific ignored ranges, keyed by ValidatorKind
    rule_ignored_ranges: std::collections::HashMap<ValidatorKind, Vec<IgnoredRange>>,
}

impl InlineConfig {
    /// Build a new inline config with an iterator of inline config items and their locations in a
    /// source file
    pub fn new(items: impl IntoIterator<Item = (Loc, InlineConfigItem)>, src: &str) -> Self {
        // Disable ranges (for formatting)
        let mut disabled_ranges = vec![];
        let mut disabled_range_start = None;
        let mut disabled_depth = 0usize;

        // Ignore ranges (for linting)
        let mut ignored_ranges = vec![];
        let mut ignored_range_start = None;
        let mut ignored_depth = 0usize;

        // Rule-specific ignore ranges
        let mut rule_ignored_ranges: std::collections::HashMap<ValidatorKind, Vec<IgnoredRange>> =
            std::collections::HashMap::new();
        let mut rule_ignored_starts: std::collections::HashMap<ValidatorKind, Option<usize>> =
            std::collections::HashMap::new();
        let mut rule_ignored_depths: std::collections::HashMap<ValidatorKind, usize> =
            std::collections::HashMap::new();

        for (loc, item) in items.into_iter().sorted_by_key(|(loc, _)| loc.start()) {
            match item {
                InlineConfigItem::DisableNextItem => {
                    let offset = loc.end();
                    let mut char_indices = src[offset..]
                        .comment_state_char_indices()
                        .filter_map(|(state, idx, ch)| match state {
                            CommentState::None => Some((idx, ch)),
                            _ => None,
                        })
                        .skip_while(|(_, ch)| ch.is_whitespace());
                    if let Some((mut start, _)) = char_indices.next() {
                        start += offset;
                        let end = char_indices
                            .find(|(_, ch)| !ch.is_whitespace())
                            .map(|(idx, _)| offset + idx)
                            .unwrap_or(src.len());
                        disabled_ranges.push(DisabledRange { start, end, loose: true });
                    }
                }
                InlineConfigItem::DisableLine => {
                    let mut prev_newline =
                        src[..loc.start()].char_indices().rev().skip_while(|(_, ch)| *ch != '\n');
                    let start = prev_newline.next().map(|(idx, _)| idx).unwrap_or_default();

                    let end_offset = loc.end();
                    let mut next_newline =
                        src[end_offset..].char_indices().skip_while(|(_, ch)| *ch != '\n');
                    let end =
                        end_offset + next_newline.next().map(|(idx, _)| idx).unwrap_or_default();

                    disabled_ranges.push(DisabledRange { start, end, loose: false });
                }
                InlineConfigItem::DisableNextLine => {
                    let offset = loc.end();
                    let mut char_indices =
                        src[offset..].char_indices().skip_while(|(_, ch)| *ch != '\n').skip(1);
                    if let Some((mut start, _)) = char_indices.next() {
                        start += offset;
                        let end = char_indices
                            .find(|(_, ch)| *ch == '\n')
                            .map(|(idx, _)| offset + idx + 1)
                            .unwrap_or(src.len());
                        disabled_ranges.push(DisabledRange { start, end, loose: false });
                    }
                }
                InlineConfigItem::DisableStart => {
                    if disabled_depth == 0 {
                        disabled_range_start = Some(loc.end());
                    }
                    disabled_depth += 1;
                }
                InlineConfigItem::DisableEnd => {
                    disabled_depth = disabled_depth.saturating_sub(1);
                    if disabled_depth == 0 {
                        if let Some(start) = disabled_range_start.take() {
                            disabled_ranges.push(DisabledRange {
                                start,
                                end: loc.start(),
                                loose: false,
                            })
                        }
                    }
                }
                InlineConfigItem::IgnoreNextItem => {
                    let offset = loc.end();
                    let mut char_indices = src[offset..]
                        .comment_state_char_indices()
                        .filter_map(|(state, idx, ch)| match state {
                            CommentState::None => Some((idx, ch)),
                            _ => None,
                        })
                        .skip_while(|(_, ch)| ch.is_whitespace());
                    if let Some((mut start, _)) = char_indices.next() {
                        start += offset;
                        // Find the end of the function declaration by looking for the closing brace
                        let mut brace_count = 0;
                        let mut found_function_start = false;
                        let mut end = src.len();

                        for (idx, ch) in src[start..].char_indices() {
                            if ch == '{' {
                                brace_count += 1;
                                found_function_start = true;
                            } else if ch == '}' {
                                brace_count -= 1;
                                if found_function_start && brace_count == 0 {
                                    end = start + idx + 1;
                                    break;
                                }
                            }
                        }
                        ignored_ranges.push(IgnoredRange { start, end, loose: true });
                    }
                }
                InlineConfigItem::IgnoreLine => {
                    let mut prev_newline =
                        src[..loc.start()].char_indices().rev().skip_while(|(_, ch)| *ch != '\n');
                    let start = prev_newline.next().map(|(idx, _)| idx).unwrap_or_default();

                    let end_offset = loc.end();
                    let mut next_newline =
                        src[end_offset..].char_indices().skip_while(|(_, ch)| *ch != '\n');
                    let end =
                        end_offset + next_newline.next().map(|(idx, _)| idx).unwrap_or_default();

                    ignored_ranges.push(IgnoredRange { start, end, loose: false });
                }
                InlineConfigItem::IgnoreNextLine => {
                    let offset = loc.end();
                    let mut char_indices =
                        src[offset..].char_indices().skip_while(|(_, ch)| *ch != '\n').skip(1);
                    if let Some((mut start, _)) = char_indices.next() {
                        start += offset;
                        let end = char_indices
                            .find(|(_, ch)| *ch == '\n')
                            .map(|(idx, _)| offset + idx + 1)
                            .unwrap_or(src.len());
                        ignored_ranges.push(IgnoredRange { start, end, loose: false });
                    }
                }
                InlineConfigItem::IgnoreStart => {
                    if ignored_depth == 0 {
                        ignored_range_start = Some(loc.end());
                    }
                    ignored_depth += 1;
                }
                InlineConfigItem::IgnoreEnd => {
                    ignored_depth = ignored_depth.saturating_sub(1);
                    if ignored_depth == 0 {
                        if let Some(start) = ignored_range_start.take() {
                            ignored_ranges.push(IgnoredRange {
                                start,
                                end: loc.start(),
                                loose: false,
                            })
                        }
                    }
                }
                InlineConfigItem::IgnoreRule { kind, scope } => {
                    let kind = kind.clone();
                    let ranges = rule_ignored_ranges.entry(kind.clone()).or_insert_with(Vec::new);
                    let range_start =
                        rule_ignored_starts.entry(kind.clone()).or_insert_with(|| None);
                    let depth = rule_ignored_depths.entry(kind).or_insert_with(|| 0);

                    match scope {
                        RuleIgnoreScope::NextItem => {
                            let offset = loc.end();
                            let mut char_indices = src[offset..]
                                .comment_state_char_indices()
                                .filter_map(|(state, idx, ch)| match state {
                                    CommentState::None => Some((idx, ch)),
                                    _ => None,
                                })
                                .skip_while(|(_, ch)| ch.is_whitespace());
                            if let Some((mut start, _)) = char_indices.next() {
                                start += offset;
                                // Find the end of the function declaration by looking for the
                                // closing brace
                                let mut brace_count = 0;
                                let mut found_function_start = false;
                                let mut end = src.len();

                                for (idx, ch) in src[start..].char_indices() {
                                    if ch == '{' {
                                        brace_count += 1;
                                        found_function_start = true;
                                    } else if ch == '}' {
                                        brace_count -= 1;
                                        if found_function_start && brace_count == 0 {
                                            end = start + idx + 1;
                                            break;
                                        }
                                    }
                                }
                                ranges.push(IgnoredRange { start, end, loose: true });
                            }
                        }
                        RuleIgnoreScope::Line => {
                            let mut prev_newline = src[..loc.start()]
                                .char_indices()
                                .rev()
                                .skip_while(|(_, ch)| *ch != '\n');
                            let start = prev_newline.next().map(|(idx, _)| idx).unwrap_or_default();

                            let end_offset = loc.end();
                            let mut next_newline =
                                src[end_offset..].char_indices().skip_while(|(_, ch)| *ch != '\n');
                            let end = end_offset +
                                next_newline.next().map(|(idx, _)| idx).unwrap_or_default();

                            ranges.push(IgnoredRange { start, end, loose: false });
                        }
                        RuleIgnoreScope::NextLine => {
                            let offset = loc.end();
                            let mut char_indices = src[offset..]
                                .char_indices()
                                .skip_while(|(_, ch)| *ch != '\n')
                                .skip(1);
                            if let Some((mut start, _)) = char_indices.next() {
                                start += offset;
                                let end = char_indices
                                    .find(|(_, ch)| *ch == '\n')
                                    .map(|(idx, _)| offset + idx + 1)
                                    .unwrap_or(src.len());
                                // Use loose: true to include locations that might extend slightly
                                // beyond the line
                                ranges.push(IgnoredRange { start, end, loose: true });
                            }
                        }
                        RuleIgnoreScope::Start => {
                            if *depth == 0 {
                                *range_start = Some(loc.end());
                            }
                            *depth += 1;
                        }
                        RuleIgnoreScope::End => {
                            *depth = depth.saturating_sub(1);
                            if *depth == 0 {
                                if let Some(start) = range_start.take() {
                                    ranges.push(IgnoredRange {
                                        start,
                                        end: loc.end(),
                                        loose: false,
                                    })
                                }
                            }
                        }
                        RuleIgnoreScope::File => {
                            // File-level ignore: ignore from start of file to end
                            // Use loose: true to ensure any location in the file is covered
                            // For loose ranges with < check, use src.len() + 1 to include all valid
                            // offsets
                            ranges.push(IgnoredRange { start: 0, end: src.len() + 1, loose: true });
                        }
                    }
                }
            }
        }
        if let Some(start) = disabled_range_start.take() {
            disabled_ranges.push(DisabledRange { start, end: src.len(), loose: false })
        }
        if let Some(start) = ignored_range_start.take() {
            ignored_ranges.push(IgnoredRange { start, end: src.len(), loose: false })
        }
        // Handle unclosed rule-specific ignore regions
        for (kind, range_start) in rule_ignored_starts {
            if let Some(start) = range_start {
                let ranges = rule_ignored_ranges.entry(kind).or_insert_with(Vec::new);
                ranges.push(IgnoredRange { start, end: src.len(), loose: false });
            }
        }
        Self { disabled_ranges, ignored_ranges, rule_ignored_ranges }
    }

    /// Check if the location is in a disabled range
    pub fn is_disabled(&self, loc: Loc) -> bool {
        self.disabled_ranges.iter().any(|range| range.includes(loc))
    }

    /// Check if the location is in an ignored range (generic ignore)
    pub fn is_ignored(&self, loc: Loc) -> bool {
        self.ignored_ranges.iter().any(|range| range.includes(loc))
    }

    /// Check if the location is in an ignored range for a specific validator kind
    pub fn is_rule_ignored(&self, loc: Loc, kind: &ValidatorKind) -> bool {
        self.rule_ignored_ranges
            .get(kind)
            .map_or(false, |ranges| ranges.iter().any(|range| range.includes(loc)))
    }
}
