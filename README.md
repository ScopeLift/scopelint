# `ScopeLint`

A simple and opinionated tool designed for basic formatting/linting of Solidity and TOML code in foundry projects.

- [`ScopeLint`](#scopelint)
  - [Installation](#installation)
  - [Usage](#usage)
    - [`scopelint fmt`](#scopelint-fmt)
    - [`scopelint check`](#scopelint-check)
    - [`scopelint spec`](#scopelint-spec)
  - [Development](#development)

## Installation

When using the [ScopeLift Foundry template](https://github.com/ScopeLift/foundry-template/) scopelint will automatically be ran in CI. To run locally:

1. Install the [rust toolchain](https://www.rust-lang.org/tools/install)
2. Run `cargo install scopelint`

## Usage

Once installed there are three commands:

- `scopelint fmt`
- `scopelint check`
- `scopelint spec`

For all commands, please open issues for any bug reports, suggestions, or feature requests.

### `scopelint fmt`

This command will format:

- Solidity files using the configuration specified in `foundry.toml`.
- TOML files using a hardcoded configuration that indents keys and sorts them alphabetically to improve readability.

**Flags:**
- `--check`: Show changes without modifying files (dry run mode)

### `scopelint check`

This command ensures that development [best practices](https://book.getfoundry.sh/tutorials/best-practices) are consistently followed by validating that:

- Test names follow a convention of `^test(Fork)?(Fuzz)?(_Revert(If|When|On))?_(\w+)*$`. (To see a list of example valid test names, see [here](https://github.com/ScopeLift/scopelint/blob/1857e3940bfe92ac5a136827374f4b27ff083971/src/check/validators/test_names.rs#L106-L127)).
- Constants and immutables are in `ALL_CAPS`.
- Function names and visibility in forge scripts only have 1 public `run` method per script.
- Internal or private functions in the `src/` directory start with a leading underscore.

[More checks](https://github.com/ScopeLift/scopelint/issues/10) are planned for the future.

Scopelint is opinionated in that it does not let you disable rules globally.
However, you can ignore specific rules for specific files using:

1. **Inline comments** in your Solidity files:
   ```solidity
   // scopelint: ignore-src-file  // Ignore 'src' rule for entire file
   // scopelint: ignore-error-next-line  // Ignore 'error' rule for next line
   ```

2. **`.scopelint` config file** in your project root:
   ```toml
   # Ignore entire files
   [ignore]
   files = [
       "src/legacy/old.sol",
       "test/integration/*.sol"
   ]

   # Ignore specific rules for specific files
   [ignore.overrides]
   "src/BaseBridgeReceiver.sol" = ["src"]  # Only ignore 'src' rule
   "src/legacy/**/*.sol" = ["src", "error"]  # Ignore multiple rules
   ```

   Supported rules: `error`, `import`, `variable`, `constant`, `test`, `script`, `src`, `eip712`

### `scopelint spec`

Most developers don't have formal specifications they are building towards, and instead only have a general idea of what they want their contracts to do.
As a result, documentation and tests are the closest things many protocols have to a specification (unless they go through the formal verification process).
And because documentation is often not written until the end of the development process, it is often incomplete or inaccurate, and therefore tests are typically the closest thing to a specification.

`scopelint spec` embraces this philosophy of "your tests are your spec" to help developers come up with a spec with minimal effort—structure your tests contracts and test names and described in the [Best Practices guide](https://book.getfoundry.sh/tutorials/best-practices), and `scopelint spec` will generate a specification for you!
This specification can be shared with other stakeholders to make sure everyone is on the same page about what the contract should do.

Below is a simple example for an ERC-20 token, the full example repo can be found [here](https://github.com/ScopeLift/scopelint-erc20-example).

![erc20-scopelint-spec-example](./assets/spec.gif)

**Flags:**
- `--show-internal`: Include internal and private functions in the specification (by default, only public and external functions are shown)

Currently this feature is in beta, and we are looking for feedback on how to improve it.
Right now it's focused on specifications for unit tests, which are very useful for developers but less useful for higher-level stakeholders.
As a result, it does not yet include information about protocol invariants or integration test / user-story types of specifications.
If you have any thoughts or ideas, please open an issue [here](https://github.com/ScopeLift/scopelint/issues/new).

## Development

For developers interested in contributing to `scopelint`, please see our [Development Guide](DEV.md) for detailed information about:

- Setting up the development environment
- Project structure and architecture
- Adding new validators and features
- Testing and code quality standards
- Contributing guidelines
