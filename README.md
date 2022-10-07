# ScopeLint

_Work in progress, not ready for use_

## Overview

This is a simple and naive tool designed to for basic formatting/linting of Solidity and TOML code in foundry projects.
Solidity formatting uses the configuration in `foundry.toml`, and TOML formatting has a hardcoded configuration.

Formatting and checking does the following:

- Runs `forge fmt` to format Solidity.
- Uses the `taplo` crate to format TOML.
- Checks the naming conventions of forge tests.
- Coming soon: Validates function names and visibility in forge scripts.

## Usage

- Install with `cargo install scopelint`.
- Format code with `scopelint fmt`.
- Validate formatting in CI with `scopelint check`.
