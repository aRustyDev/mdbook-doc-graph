# MDBook Plugin Development Recipes
#
# Usage: just <recipe>
# Run `just` or `just --list` to see available recipes.

root := `git rev-parse --show-toplevel`
docs := root / "docs"

# Default recipe - show available commands
default:
    @just --list

# ============================================================================
# Development
# ============================================================================

# Run all checks (format, lint, test)
check: fmt-check clippy test

# Format code
fmt:
    cargo fmt

# Check formatting without changes
fmt-check:
    cargo fmt --check

# Run clippy linter
clippy:
    cargo clippy -- -D warnings

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# ============================================================================
# Build
# ============================================================================

# Build debug
build:
    cargo build

# Build release
release:
    cargo build --release

# Install locally
install:
    cargo install --path .

# ============================================================================
# Integration Testing
# ============================================================================

# Run integration test with sample book
integration: install
    cd tests/fixtures/test-book && mdbook build

# Clean test book output
clean-test:
    rm -rf tests/fixtures/test-book/book

# ============================================================================
# Documentation
# ============================================================================

# Build documentation
docs-build:
    cd {{docs}} && mdbook build

# Serve documentation locally
docs-serve:
    cd {{docs}} && mdbook serve

# Clean documentation build
docs-clean:
    rm -rf {{docs}}/book

# ============================================================================
# CI/CD
# ============================================================================

# Run all CI checks
ci: check integration docs-build

# Clean all build artifacts
clean: docs-clean clean-test
    cargo clean

# ============================================================================
# Version Management (requires cargo-edit)
# ============================================================================

# Bump patch version (0.0.X)
bump-patch:
    cargo set-version --bump patch

# Bump minor version (0.X.0)
bump-minor:
    cargo set-version --bump minor

# Bump major version (X.0.0)
bump-major:
    cargo set-version --bump major

# ============================================================================
# Utilities
# ============================================================================

# List all files related to an ID
ls-related id:
    @rg --sort=path -l -n "{{id}}" .aim/ . 2>/dev/null || echo "No matches found"

# Find pattern in file
find pattern file:
    @rg --sort=path -n "{{pattern}}" "{{file}}"
