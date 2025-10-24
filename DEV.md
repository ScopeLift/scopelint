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

## Beta Release Workflow

This project includes a streamlined beta release system that allows for easy testing and iteration without affecting the main release process.

### Local Development

For fast local development and testing:

```bash
# Build and install from current source
./scripts/dev-install.sh
```

This script:
- ✅ Builds the binary from your current code
- ✅ Installs it to `/usr/local/bin/scopelint`
- ✅ Tests the installation
- ✅ Works completely offline
- ✅ Perfect for rapid iteration during development

### Beta Testing

For testing beta releases from GitHub:

```bash
# Install latest beta release
./scripts/install-beta.sh

# Install specific beta version
./scripts/install-beta.sh v1.0.0-beta.1

# Install to custom location
./scripts/install-beta.sh latest ~/bin
```

This script:
- ✅ Downloads pre-built binaries from GitHub releases
- ✅ Installs and tests the binary
- ✅ Works with any existing GitHub release
- ✅ Perfect for testing published beta versions

### Complete Development Workflow

#### 1. Local Development
```bash
# Make changes to your code
# ... edit files ...

# Test locally (fast iteration)
./scripts/dev-install.sh

# Test in your projects
cd ~/my-solidity-project
scopelint --help
```

#### 2. Create Beta Release
```bash
# Build for release with beta tag
GIT_TAG=beta cargo build --release

# Create and push tag
git tag v1.0.0-beta
git push origin v1.0.0-beta

# Create GitHub release via CLI
gh release create v1.0.0-beta --prerelease target/release/scopelint
```

#### 3. Test Beta Release
```bash
# Install the beta
./scripts/install-beta.sh v1.0.0-beta

# Test in different projects
cd ~/project1
scopelint --help

cd ~/project2
scopelint --help
```

#### 4. Final Release

If beta is good, create final release
-  Use existing release.yml workflow
- This publishes to crates.io automatically