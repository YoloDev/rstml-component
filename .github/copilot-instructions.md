# GitHub Copilot Instructions for rstml-component

## Project Overview

`rstml-component` is a Rust library for creating dynamic HTML components. It enables developers to define HTML components using Rust structs and generate HTML content efficiently, particularly useful for server-side applications.

## Repository Structure

This is a Cargo workspace with the following structure:
- **Main crate** (`rstml-component`): Core library in the root
- **Macro crate** (`macro/`): Procedural macros for the library
- **Axum integration** (`integrations/axum/`): Integration with the Axum web framework
- **Tests** (`tests/`): Integration tests

## Coding Standards

### Rust Style Guidelines

- **Formatting**: Use hard tabs (not spaces) for indentation
- **Tab width**: 2 spaces equivalent
- **Line endings**: Unix-style (LF)
- **Charset**: UTF-8
- **Trailing whitespace**: Must be trimmed
- **Final newline**: Required in all files

### Code Quality

- **Linting**: All code must pass `cargo clippy --all -- -D warnings` (warnings treated as errors)
- **Formatting**: Code must be formatted with `rustfmt` using the project's `rustfmt.toml` configuration
- **Documentation**: Public APIs should have doc comments
- **Features**: Use `#[cfg_attr(docsrs, doc(cfg(...)))]` for feature-gated items

## Build & Test Commands

### Building
```bash
cargo build --workspace --all-features
```

### Testing
```bash
cargo test --workspace --all-features
```

### Linting
```bash
cargo clippy --all -- -D warnings
```

### Formatting
```bash
cargo fmt --all -- --check
```

### Documentation
```bash
cargo doc --workspace --all-features
```

## Development Workflow

1. Make code changes following the coding standards
2. Format code with `cargo fmt`
3. Run clippy to check for issues
4. Run tests to ensure functionality
5. Update documentation if needed

## Feature Flags

- `sanitize`: Enables HTML sanitization functionality using the `ammonia` crate

## Dependencies Management

This is a workspace. When adding dependencies:
- Add workspace-shared dependencies to `[workspace.dependencies]` in the root `Cargo.toml`
- Reference them in member crates using `{ workspace = true }`

## CI/CD

The project uses GitHub Actions for CI:
- Tests run on macOS, Windows, and Ubuntu
- All tests must pass before merging
- Documentation is automatically built and published to GitHub Pages from the main branch

## Important Files

- `Cargo.toml`: Workspace configuration and dependencies
- `rustfmt.toml`: Rust formatting configuration
- `.editorconfig`: Editor configuration
- `cliff.toml`: Changelog generation configuration
- `release-plz.toml`: Release automation configuration
