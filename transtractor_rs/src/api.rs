use transtractor_core::configs::db::ConfigDB;
use transtractor_core::configs::typer::StatementTyper;
use transtractor_core::parsers::flows::{
    config_json_file_to_config,
    text_items_to_debug::text_items_to_debug,
    text_items_to_layout::text_items_to_layout,
    text_items_to_statement_datas::text_items_to_statement_datas,
};
use transtractor_core::structs::{StatementData, TextItem};

use crate::error::ParseError;
use crate::pdf_extractor;

/// High-level Rust API for parsing bank statements
pub struct Parser {
    db: ConfigDB,
    typer: StatementTyper,
}

impl Parser {
    /// Create a new parser instance
    pub fn new() -> Self {
        Self {
            db: ConfigDB::new(true, false),
            typer: StatementTyper::new(),
        }
    }

    /// Create a new parser pre-loaded with all bundled default configurations.
    /// Mirrors Python's `Parser()` behavior.
    pub fn with_defaults() -> Result<Self, crate::error::ParseError> {
        let mut parser = Self::new();
        for config_json in transtractor_configs::CONFIGS {
            parser.import_config_from_str(config_json)?;
        }
        Ok(parser)
    }

    /// Load a configuration from a JSON file
    pub fn import_config_from_file(&mut self, path: &str) -> Result<(), ParseError> {
        self.db
            .register_from_file(path)
            .map_err(|e| ParseError::ConfigError(e))?;

        let cfg = config_json_file_to_config::from_json_file(path)
            .map_err(|e| ParseError::ConfigError(e))?;
        self.typer.add_account_terms(&cfg.key, &cfg.account_terms);

        Ok(())
    }

    /// Load a configuration from a JSON string
    pub fn import_config_from_str(&mut self, json: &str) -> Result<(), ParseError> {
        self.db
            .register_from_str(json)
            .map_err(|e| ParseError::ConfigError(e))?;

        let cfg = config_json_file_to_config::from_json_str(json)
            .map_err(|e| ParseError::ConfigError(e))?;
        self.typer.add_account_terms(&cfg.key, &cfg.account_terms);

        Ok(())
    }

    /// Register a configuration without updating the typer (for batch configs)
    pub fn register_config_from_str(&mut self, json: &str) -> Result<(), ParseError> {
        self.db
            .register_from_str(json)
            .map_err(|e| ParseError::ConfigError(e))
    }

    /// Get the list of config keys applicable to extracted text items
    pub fn get_applicable_config_keys(&self, text_items: &[TextItem]) -> Vec<String> {
        self.typer.identify(&text_items.to_vec())
    }

    /// Parse a PDF file and return statement data
    ///
    /// # Arguments
    ///
    /// * `pdf_path` - Path to the PDF file
    /// * `config_keys` - Optional config keys to use. If None, applicable configs are auto-detected.
    ///
    /// # Returns
    ///
    /// The first error-free StatementData found, or an error if none could be parsed.
    pub fn parse_pdf(
        &self,
        pdf_path: &str,
        config_keys: Option<Vec<String>>,
    ) -> Result<StatementData, ParseError> {
        let text_items = pdf_extractor::extract_text_items_from_file(pdf_path)?;
        self.parse_text_items(&text_items, config_keys)
    }

    /// Parse PDF bytes and return statement data
    pub fn parse_pdf_bytes(
        &self,
        pdf_bytes: &[u8],
        config_keys: Option<Vec<String>>,
    ) -> Result<StatementData, ParseError> {
        let text_items = pdf_extractor::extract_text_items_from_bytes(pdf_bytes)?;
        self.parse_text_items(&text_items, config_keys)
    }

    /// Parse pre-extracted text items
    pub fn parse_text_items(
        &self,
        text_items: &[TextItem],
        config_keys: Option<Vec<String>>,
    ) -> Result<StatementData, ParseError> {
        let keys = if let Some(keys) = config_keys {
            keys
        } else {
            self.get_applicable_config_keys(text_items)
        };

        if keys.is_empty() {
            return Err(ParseError::ParsingError(
                "No applicable configurations found for this statement".to_string(),
            ));
        }

        let mut configs = Vec::new();
        for key in &keys {
            let cfg = self
                .db
                .get_config(key)
                .map_err(|e| ParseError::ConfigError(format!("Config '{}': {}", key, e)))?;
            configs.push(cfg);
        }

        let results = text_items_to_statement_datas(&text_items.to_vec(), &configs)
            .map_err(|e| ParseError::ParsingError(e))?;

        // Return first error-free result
        for data in results {
            if data.errors.is_empty() {
                return Ok(data);
            }
        }

        Err(ParseError::NoValidStatement(keys.join(", ")))
    }

    /// Get debug information for text items (useful for troubleshooting)
    pub fn get_debug_info(
        &self,
        text_items: &[TextItem],
        config_keys: Option<Vec<String>>,
    ) -> Result<String, ParseError> {
        let keys = if let Some(keys) = config_keys {
            keys
        } else {
            self.get_applicable_config_keys(text_items)
        };

        if keys.is_empty() {
            return Err(ParseError::ParsingError(
                "No applicable configurations found".to_string(),
            ));
        }

        let mut configs = Vec::new();
        for key in &keys {
            let cfg = self
                .db
                .get_config(key)
                .map_err(|e| ParseError::ConfigError(format!("Config '{}': {}", key, e)))?;
            configs.push(cfg);
        }

        text_items_to_debug(&text_items.to_vec(), &configs)
            .map_err(|e| ParseError::ParsingError(e))
    }

    /// Get layout visualization for text items
    pub fn get_layout_info(
        &self,
        text_items: &[TextItem],
        y_bin: f32,
        x_gap: f32,
    ) -> Result<String, ParseError> {
        text_items_to_layout(&text_items.to_vec(), y_bin, x_gap)
            .map_err(|e| ParseError::ParsingError(e))
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function: Parse a PDF file with default settings
///
/// Requires at least one config to be loaded via environment or default configs.
pub fn parse_pdf(pdf_path: &str) -> Result<StatementData, ParseError> {
    let parser = Parser::new();
    parser.parse_pdf(pdf_path, None)
}

/// Convenience function: Parse PDF bytes with default settings
pub fn parse_pdf_bytes(pdf_bytes: &[u8]) -> Result<StatementData, ParseError> {
    let parser = Parser::new();
    parser.parse_pdf_bytes(pdf_bytes, None)
}
