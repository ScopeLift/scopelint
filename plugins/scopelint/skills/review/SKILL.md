---
name: review
description: Review Foundry/Solidity projects for code structure and quality using ScopeLint's philosophy (conventions over config, tests as spec, clear script entrypoints). Use when the user asks for a code review, structure review, or quality review of a Solidity/Foundry codebase.
---

# ScopeLift Review

Review code structure and quality in Foundry/Solidity projects using the same principles behind ScopeLint: conventions over config, tests as spec, and clear boundaries so protocol logic stays easy to reason about. Combine `scopelint check` output with brief, actionable findings that explain the "why."

## When to use

- User asks for a code review, structure review, or quality review of a Solidity/Foundry project.
- User wants to understand how their project aligns with Foundry best practices and ScopeLint's conventions.
- Reviewing a PR or a directory of contracts/tests/scripts and you want to surface both tool findings and higher-level structure issues.

## ScopeLint "why" (feed this into findings)

- **Foundry gives great primitives**, but projects easily drift into inconsistent naming, test layouts, and scripts that are hard to reason about. The review should call out where that drift shows up.
- **Solidity-aware checks**: Rules are tailored to Foundry (tests, scripts, sources), not generic linting. Findings should reference these contexts (e.g. "script," "test," "src").
- **Conventions over config**: A small set of opinionated practices (test naming, internal prefixes, single script `run`, ALL_CAPS constants) so teams converge quickly. Findings should tie issues to these conventions.
- **Tests as spec**: Test layout and naming should read like a spec; `scopelint spec` can generate a human-readable spec from them. The review can note when test names or structure don't support that.
- **Fits the workflow**: Same paths as Forge via `foundry.toml`; no custom config required. Mention if path or layout quirks might affect tooling or CI.

## Workflow

1. **Run scopelint check**  
   From project root:
   ```bash
   scopelint check
   ```
   Capture all findings (file, rule, line, message). These are your **tool findings**.

2. **Optional: run scopelint spec**  
   If tests exist:
   ```bash
   scopelint spec
   ```
   Use the output to see whether test structure already reads like a spec or where it's vague/inconsistent. Reference this in the review when discussing test quality.

3. **Inspect structure**  
   Quickly check:
   - **Scripts**: One public `run` per script? Helpers in a separate contract/file?
   - **Tests**: Naming like `test_RevertWhen_*` / `testFork_*` / `testFuzz_*`? Grouped by behavior?
   - **Src**: Internal/private functions prefixed with `_`? Constants/immutables in ALL_CAPS?
   - **Layout**: `src` / `test` / `script` (or custom paths in `foundry.toml`) used consistently?

4. **Produce the review**  
   Output a short review that:
   - **Summarizes** overall structure and alignment with the conventions above.
   - **Lists findings** in two groups:
     - **ScopeLint findings**: Each item from `scopelint check` with a one-line "why" (e.g. "Internal functions should be prefixed with `_` so they're visually distinct and match Foundry conventions").
     - **Structure / quality**: Any extra observations (e.g. "Test file has many unrelated tests; consider splitting by contract or behavior," "Script has two public entrypoints; only one `run` is recommended").
   - **Suggests next steps**: Run `scopelint fix` for auto-fixable items, then fix the rest by hand; optionally run `scopelint fmt` and `scopelint spec` to tidy format and regenerate the spec.

## Review output format

Use a structure like:

```markdown
## ScopeLift review

### Summary
[2–4 sentences on structure, test/script/source layout, and alignment with conventions.]

### ScopeLint findings (with rationale)
| Rule   | File / location | Issue | Why it matters |
|--------|------------------|-------|----------------|
| script | script/Deploy.s.sol | … | Single public `run` keeps deployment and tooling predictable. |
| src    | src/Counter.sol (L42) | … | `_` prefix for internal/private avoids confusion with external API. |
| test   | test/Counter.t.sol (L18) | … | Convention names (e.g. test_RevertWhen_*) let tests double as a spec. |
| constant | src/Token.sol (L12) | … | ALL_CAPS makes constants easy to spot. |

### Structure / quality
- [Bullet points for anything not covered by ScopeLint but relevant to structure or readability.]

### Next steps
1. Run `scopelint fix` to remove unused imports and re-check.
2. Address remaining findings (rename tests, constants, internal functions, or script entrypoints).
3. Optionally run `scopelint fmt` and `scopelint spec` and commit the updated spec.
```

## Notes

- Keep the "why" column short: one sentence per rule type is enough. Reuse the same rationale for the same rule across findings.
- If `scopelint check` passes, the review can be brief: "No ScopeLint findings; structure looks aligned with conventions," plus any optional structure/quality notes.
- If the project isn't Foundry or doesn't use the usual layout, say so and focus on whatever structure and tooling they do use; you can still apply the same philosophy (conventions, tests as spec, clear entrypoints) where relevant.
