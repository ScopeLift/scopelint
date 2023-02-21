# ScopeLint

A simple and opinionated tool designed for basic formatting/linting of Solidity and TOML code in foundry projects.

## Overview

Solidity formatting uses the configuration in `foundry.toml`, and TOML formatting has a hardcoded configuration.

Formatting and checking does the following:

- Runs `forge fmt` to format Solidity.
- Uses the `taplo` crate to format TOML.
- Validates test names follow a convention of `test(Fork)?(Fuzz)?_(Revert(If_|When_){1})?\w{1,}`.
- Validates constants and immutables are in `ALL_CAPS`.
- Validates function names and visibility in forge scripts to 1 public `run` method per script (executable script files are expected to end with `.s.sol`, whereas non-executable helper contracts in the scripts dir just end with `.sol`).
- Validates internal functions in `src/` start with a leading underscore.

## Usage

- Install with `cargo install scopelint`.
- Format code with `scopelint fmt`.
- Validate formatting with `scopelint check`.
- Print the version with `scopelint --version`.
- Use the ScopeLift [foundry template](https://github.com/ScopeLift/foundry-template/) to automatically run scopelint and slither in CI.

## Limitations

This tool is currently opinionated and does not let you configure it's behavior.
