---
name: lint
description: Run scopelint check on Foundry projects, list findings in priority order, and address them (or ask the user). Use when the user wants to lint a Solidity/Foundry project, fix convention issues, or when reviewing code for ScopeLint-style best practices.
---

# ScopeLint Lint Workflow

Use the ScopeLint CLI to find convention and formatting issues in a Foundry project, present them in priority order, and either fix them or ask the user how to proceed. By default, offer to address all findings unless the user prefers to triage manually.

## When to use

- User asks to lint the project, run scopelint, or fix convention issues.
- User wants findings listed so they can address them or have the agent fix them.
- Working in a repo with Solidity/Foundry and you want to enforce test naming, constants, script entrypoints, internal prefixes, etc.

## Prerequisites

- Project has a Foundry layout (e.g. `src/`, `test/`, `script/`) and optionally `foundry.toml`.
- ScopeLint is installed: `cargo install scopelint` (or already on PATH).

## Workflow

1. **Run check**  
   From the project root (or the directory containing `foundry.toml`):
   ```bash
   scopelint check
   ```
   Capture stdout/stderr. Exit code is non-zero if there are findings.

2. **List findings in priority order**  
   Parse the output and present findings in this order (structural first, then naming, then cleanup):
   - **Script** – multiple public `run` methods or wrong script interface (blocks deployment clarity).
   - **Src** – internal/private functions not prefixed with `_` (consistency and readability).
   - **Test** – test names not matching `test(Fork)?(Fuzz)?(_Revert(If|When|On))?_(\w+)*` (tests as spec).
   - **Constant** – constants/immutables not `ALL_CAPS`.
   - **Error** – custom errors not following project convention (e.g. prefix).
   - **Variable** – variable naming issues.
   - **Import** – unused imports (can be auto-fixed with `scopelint fix`).
   - **Directive / Eip712** – directive or EIP-712 typehash issues.

   For each finding, show: **rule**, **file**, **line** (if any), and **message**. Group by file if there are many findings.

3. **Propose next step**  
   - **Default**: Offer to address all findings (apply fixes and/or edits as appropriate).
   - If the user prefers to fix themselves, list the prioritized list and stop.
   - If they want only some categories fixed (e.g. "fix imports and constants only"), do that.

4. **Applying fixes**  
   - **Unused imports**: Run `scopelint fix`; it removes unused imports and re-runs `scopelint check`. Prefer this over hand-editing.
   - **Other rules**: Apply edits in code (rename tests, constants, internal functions, script entrypoints, etc.) so that `scopelint check` passes. Optionally run `scopelint fmt` after edits to keep formatting consistent.

5. **Re-check**  
   After edits, run `scopelint check` again and report any remaining or new findings.

## Output format (for the user)

When listing findings, use a clear, scannable format, for example:

```markdown
## ScopeLint findings (by priority)

### Script
- `script/Deploy.s.sol`: multiple public run methods, only one is allowed.

### Src
- `src/Counter.sol` (line 42): internal function name should start with `_`.

### Test
- `test/Counter.t.sol` (line 18): Invalid test name `testCounter` — use convention like `test_RevertWhen_Zero()`.

### Constant
- `src/Token.sol` (line 12): constant/immutable should be ALL_CAPS.

### Import (auto-fixable)
- `src/Helper.sol` (line 5): unused import `Foo`.
```

Then state: "I can fix these (including running `scopelint fix` for imports). Proceed?" or follow the user's preference.

## Notes

- Paths are read from `foundry.toml` (e.g. `src`, `test`, `script`); optional `[check]` section can override. No need to change config if the layout is standard.
- Ignored findings (inline `// scopelint: ...` or `.scopelint` file) are not reported by `scopelint check`; only non-ignored items need to be listed and fixed.
- If `scopelint` is not installed, tell the user to run `cargo install scopelint` and then re-run the workflow.
