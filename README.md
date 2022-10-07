# ScopeLint

_Work in progress, not ready for use_

## Overview

This is a simple and opinionated tool designed to for basic formatting/linting of Solidity and TOML code in foundry projects.
Solidity formatting uses the configuration in `foundry.toml`, and TOML formatting has a hardcoded configuration.

Formatting and checking does the following:

- Runs `forge fmt` to format Solidity.
- Uses the `taplo` crate to format TOML.
- Validates test names follow a convention of `test(Fork)?(Fuzz)?_(Revert(If_|When_){1})?\w{1,}`.
- Validates constants and immutables are in `ALL_CAPS`.
- Potentially coming soon (ideas welcome):
    - Validate function names and visibility in forge scripts to 1 public `run` method per script.
    - Validate internal functions in `src/` start with a leading underscore.
    - What else?

## Usage

- Install with `cargo install scopelint`.
- Format code with `scopelint fmt`.
- Validate formatting with `scopelint check`.
- Use the ScopeLift [foundry template](https://github.com/ScopeLift/foundry-template/) to automatically run scopelint and slither in CI.
