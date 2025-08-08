# Development Guide

This document provides information for developers working on `scopelint`.

## Prerequisites

- **Rust Toolchain**: Install the [Rust toolchain](https://www.rust-lang.org/tools/install)
- **Foundry**: Install [Foundry](https://getfoundry.sh/) for Solidity development
- **Nightly Rust**: This project uses nightly Rust for advanced rustfmt features

## Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/ScopeLift/scopelint.git
   cd scopelint
   ```

2. **Switch to nightly Rust**:
   ```bash
   rustup default nightly
   ```

3. **Install dependencies**:
   ```bash
   cargo build
   ```

## Development Workflow

### Code Quality

Before submitting any changes, ensure your code passes all quality checks:

```bash
# Format code
cargo fmt

# Run clippy (linting)
cargo clippy --workspace --all-targets --all-features

# Run tests
cargo test

# Check formatting
cargo fmt --check
```

### Project Structure

```
scopelint/
├── src/
│   ├── main.rs          # Binary entry point
│   ├── lib.rs           # Library entry point
│   ├── config.rs        # CLI configuration and argument parsing
│   ├── check/           # Code validation and linting
│   │   ├── mod.rs       # Main check module
│   │   ├── validators/  # Individual validation rules
│   │   ├── comments.rs  # Comment parsing
│   │   ├── inline_config.rs # Inline configuration parsing
│   │   ├── report.rs    # Report generation
│   │   └── utils.rs     # Shared utilities
│   ├── fmt/             # Code formatting
│   │   └── mod.rs       # Formatting implementation
│   └── spec/            # Specification generation
│       └── mod.rs       # Spec generation implementation
├── tests/               # Integration tests
│   ├── check.rs         # Check command tests
│   ├── spec.rs          # Spec command tests
│   └── test-projects/   # Sample projects for testing
```

### Adding New Validators

To add a new validation rule:

1. **Create a new validator file** in `src/check/validators/`:
   ```rust
   use crate::check::{
       utils::{InvalidItem, ValidatorKind},
       Parsed,
   };
   use solang_parser::pt::SourceUnitPart;
   
   #[must_use]
   /// Validates that [describe what this validates].
   pub fn validate(parsed: &Parsed) -> Vec<InvalidItem> {
       // Implementation here
       Vec::new()
   }
   ```

2. **Register the validator** in `src/check/validators/mod.rs`:
   ```rust
   pub mod your_validator;
   ```

3. **Add it to the validation loop** in `src/check/mod.rs`:
   ```rust
   results.add_items(validators::your_validator::validate(&parsed));
   ```

4. **Write tests** in the validator file:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use crate::check::utils::ExpectedFindings;
   
       #[test]
       fn test_validate() {
           let content = r"
               contract MyContract {
                   // Test cases here
               }
           ";
   
           let expected_findings = ExpectedFindings::new(0);
           expected_findings.assert_eq(content, &validate);
       }
   }
   ```

### Testing

#### Unit Tests
Run unit tests with:
```bash
cargo test
```

#### Integration Tests
The project includes integration tests that run the binary against sample projects:

- `tests/check.rs`: Tests the `check` command
- `tests/spec.rs`: Tests the `spec` command

#### Test Projects
Sample projects in `tests/` are used for integration testing:
- `check-proj1-AllFindings/`: Project with known validation issues
- `check-proj2-NoFindings/`: Project that should pass all checks
- `spec-proj1/`: Project for testing specification generation

### Configuration

### CI/CD

The project uses GitHub Actions for continuous integration:

- **Build**: Compiles the project with nightly Rust
- **Test**: Runs all tests
- **Lint**: Runs clippy and formatting checks
- **Clippy**: Runs additional clippy checks

#### Formatting Issues
- Always run `cargo fmt` before committing
- The project uses nightly Rust features, ensure you're on nightly toolchain