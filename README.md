# MDBook Plugin Template

Template repository for developing MDBook plugins (preprocessors and backends).

## Quick Start

1. **Create a new repo from this template:**
   ```bash
   gh repo create arustydev/mdbook-your-plugin \
     --public \
     --template aRustyDev/tmpl-mdbook-plugin \
     --clone
   ```

2. **Customize the template:**
   - Replace `PLUGIN_NAME` with your plugin name (e.g., `json-table`)
   - Replace `PLUGIN_NAME_UNDERSCORE` with underscored version (e.g., `json_table`)
   - Replace `DESCRIPTION` with your plugin description
   - Update `Cargo.toml` metadata

3. **Develop:**
   ```bash
   just check      # Run fmt, clippy, tests
   just install    # Install locally
   just integration # Test with sample book
   ```

## Template Contents

```
.
├── Cargo.toml          # Rust package manifest (customize this!)
├── src/
│   ├── lib.rs          # Library entry point
│   ├── main.rs         # CLI entry point
│   ├── config.rs       # Configuration parsing
│   ├── error.rs        # Error types
│   └── preprocessor.rs # Preprocessor implementation
├── tests/
│   ├── integration.rs  # Integration tests
│   └── fixtures/
│       └── test-book/  # Sample MDBook for testing
├── docs/
│   ├── book.toml       # MDBook config
│   └── src/
│       ├── README.md   # Documentation home
│       ├── data-flow.md
│       ├── configuration.md
│       └── adr/        # Architecture Decision Records
├── .github/
│   ├── workflows/      # CI/CD pipelines
│   └── templates/      # Issue/PR templates
├── justfile            # Development recipes
└── .releaserc          # Semantic release config
```

## Development Workflow

### Available Commands

```bash
just              # Show all commands
just check        # Run fmt + clippy + test
just build        # Build debug
just release      # Build release
just install      # Install locally
just integration  # Test with sample book
just docs-serve   # Serve documentation
just ci           # Run full CI pipeline
```

### TDD Workflow

1. Write failing tests in `tests/integration.rs`
2. Implement in `src/preprocessor.rs`
3. Run `just check` until tests pass
4. Test with sample book: `just integration`

## Key Files to Customize

| File | What to Change |
|------|----------------|
| `Cargo.toml` | Package name, description, keywords |
| `src/lib.rs` | Module structure, exports |
| `src/preprocessor.rs` | Plugin name, `run()` implementation |
| `src/config.rs` | Configuration fields |
| `src/error.rs` | Error variants |
| `docs/src/README.md` | Documentation |
| `tests/fixtures/test-book/` | Test content |

## Important Notes

### toml Version Compatibility

The `Cargo.toml` uses `toml = "0.5"` which **must match mdbook's version**. Using `toml = "0.8"` will cause type conflicts:

```
error[E0308]: mismatched types
  --> expected `Map<String, Value>`, found a different `Map<String, Value>`
```

### Preprocessor vs Backend

This template defaults to a **preprocessor**. For a **backend**:

1. Change `[preprocessor.PLUGIN_NAME]` to `[output.PLUGIN_NAME]` in configs
2. Use `RenderContext` instead of `PreprocessorContext`
3. Write files to `ctx.destination` instead of returning modified book

## Resources

- [MDBook Preprocessor Docs](https://rust-lang.github.io/mdBook/for_developers/preprocessors.html)
- [MDBook Backend Docs](https://rust-lang.github.io/mdBook/for_developers/backends.html)
- [Example: mdbook-frontmatter](https://github.com/aRustyDev/mdbook-frontmatter)

## License

GPL-3.0-or-later
