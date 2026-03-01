use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("PDF error: {0}")]
    PdfError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("No error-free statement data found. Applicable configs: {0}")]
    NoValidStatement(String),
}
