# ScopeLint Claude Code Plugin

This plugin adds two [Claude Code](https://code.claude.com) skills for Foundry/Solidity workflows:

| Skill | Command | Description |
|-------|----------|-------------|
| **lint** | `/scopelint:lint` | Run `scopelint check`, list findings in priority order, and fix them (or ask). By default, offer to address all findings. |
| **review** | `/scopelint:review` | Code structure and quality review using ScopeLint’s philosophy; outputs findings with the “why” behind each convention. |

Skills are also **model-invoked**: Claude can use them automatically when you ask to lint the project, fix convention issues, or request a code/structure review of a Solidity/Foundry codebase.

## Installation

1. Add the ScopeLint marketplace (this repo):
   ```bash
   /plugin marketplace add ScopeLift/scopelint
   ```
2. Install the plugin:
   ```bash
   /plugin install scopelint@scopelint
   ```
3. Restart Claude Code if needed. Run `/help` to see the skills under the `scopelint` namespace.

## Requirements

- [ScopeLint](https://github.com/ScopeLift/scopelint) installed: `cargo install scopelint`
- A Foundry project layout (`src/`, `test/`, `script/`, and optionally `foundry.toml`)

## Local development

Load the plugin without installing from the marketplace:

```bash
claude --plugin-dir ./plugins/scopelint
```

Then run e.g. `/scopelint:lint` or ask Claude to “run scopelint and fix the findings.”
