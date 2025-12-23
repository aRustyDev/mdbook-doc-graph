# Data Flow

## Overview

```mermaid
flowchart LR
    A[book.toml] --> B[mdbook build]
    B --> C[Load Book]
    C --> D[PreprocessorContext + Book]
    D --> E[mdbook-PLUGIN_NAME stdin]
    E --> F[Transform Chapters]
    F --> G[Modified Book JSON]
    G --> H[stdout]
    H --> I[Renderer]
```

## Input Format

MDBook sends a JSON array via stdin containing two objects:

### 1. PreprocessorContext

```json
{
  "root": "/path/to/book",
  "config": {
    "book": { "title": "...", "authors": [...] },
    "preprocessor": {
      "PLUGIN_NAME": {
        // Your configuration from book.toml
      }
    }
  },
  "renderer": "html",
  "mdbook_version": "0.4.x"
}
```

### 2. Book

```json
{
  "sections": [
    {
      "Chapter": {
        "name": "Chapter Title",
        "content": "# Chapter Title\n\nMarkdown content...",
        "number": [1],
        "sub_items": [],
        "path": "chapter.md",
        "source_path": "chapter.md",
        "parent_names": []
      }
    }
  ]
}
```

## Output Format

The plugin outputs the modified book JSON to stdout:

```json
{
  "sections": [
    {
      "Chapter": {
        "name": "Chapter Title",
        "content": "# Chapter Title\n\n<Transformed content>...",
        ...
      }
    }
  ]
}
```

## Processing Steps

1. **Parse Input**: Read JSON from stdin
2. **Load Config**: Extract plugin configuration from PreprocessorContext
3. **Process Chapters**: Transform each chapter's content
4. **Output Result**: Write modified Book JSON to stdout

## Error Handling

- Errors are written to stderr
- Exit code 0 = success
- Exit code 1 = failure
- Partial failures can be logged as warnings
