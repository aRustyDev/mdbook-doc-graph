//! Preprocessor implementation.

use crate::{Config, Error};
use mdbook::book::{Book, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;

/// Plugin preprocessor for MDBook.
pub struct PluginPreprocessor;

impl PluginPreprocessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PluginPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor for PluginPreprocessor {
    fn name(&self) -> &str {
        // TODO: Replace with your plugin name
        "PLUGIN_NAME"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> mdbook::errors::Result<Book> {
        // Get preprocessor config
        let config = ctx
            .config
            .get_preprocessor(self.name())
            .ok_or_else(|| mdbook::errors::Error::msg("Missing preprocessor configuration"))?;

        let _config = Config::from_table(config).map_err(mdbook::errors::Error::msg)?;

        // Process each chapter
        let mut errors: Vec<String> = Vec::new();

        book.for_each_mut(|item| {
            if let BookItem::Chapter(ref mut chapter) = item {
                if let Err(e) = process_chapter(chapter) {
                    errors.push(e.to_string());
                }
            }
        });

        if !errors.is_empty() {
            return Err(mdbook::errors::Error::msg(format!(
                "Plugin errors:\n{}",
                errors.join("\n")
            )));
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        // TODO: Customize supported renderers
        // By default, support all renderers except "not-supported"
        renderer != "not-supported"
    }
}

/// Process a single chapter.
fn process_chapter(chapter: &mut Chapter) -> Result<(), Error> {
    // TODO: Implement chapter processing logic
    //
    // Example: Transform content
    // chapter.content = transform_content(&chapter.content)?;
    //
    // For now, do nothing (pass-through)
    let _ = chapter;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preprocessor_name() {
        let preprocessor = PluginPreprocessor::new();
        assert_eq!(preprocessor.name(), "PLUGIN_NAME");
    }

    #[test]
    fn test_supports_html_renderer() {
        let preprocessor = PluginPreprocessor::new();
        assert!(preprocessor.supports_renderer("html"));
    }

    #[test]
    fn test_supports_epub_renderer() {
        let preprocessor = PluginPreprocessor::new();
        assert!(preprocessor.supports_renderer("epub"));
    }

    #[test]
    fn test_does_not_support_not_supported() {
        let preprocessor = PluginPreprocessor::new();
        assert!(!preprocessor.supports_renderer("not-supported"));
    }
}
