# ScopeLint

This is a simple and naive tool designed to for basic formatting/linting of Solidity and TOML code in foundry projects.
Solidity formatting uses the configuration in `foundry.toml`, and TOML formatting has a hardcoded configuration.

- Install with `cargo install scopelint`.
- Format code with `scopelint fmt`.
- Validate formatting in CI with `scopelint check`.
