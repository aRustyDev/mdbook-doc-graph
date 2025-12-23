# Configuration

Configure the plugin in your `book.toml` file.

## Basic Configuration

```toml
[preprocessor.PLUGIN_NAME]
# Add your configuration options here
```

## Options

<!-- TODO: Document all configuration options -->

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `option1` | string | `""` | Description of option1 |
| `option2` | bool | `false` | Description of option2 |

## Examples

### Minimal Configuration

```toml
[preprocessor.PLUGIN_NAME]
```

### Full Configuration

```toml
[preprocessor.PLUGIN_NAME]
option1 = "value"
option2 = true
```

## Renderer Support

By default, this plugin supports all renderers. To limit to specific renderers:

```toml
[preprocessor.PLUGIN_NAME]
renderers = ["html", "epub"]
```

## Troubleshooting

### Plugin not running

Ensure the binary is in your PATH:

```bash
which mdbook-PLUGIN_NAME
```

### Configuration not applied

Check your `book.toml` syntax:

```bash
mdbook build 2>&1 | head -20
```

### Debug logging

Enable debug output:

```bash
RUST_LOG=debug mdbook build
```
