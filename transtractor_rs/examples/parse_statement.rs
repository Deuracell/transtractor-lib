use transtractor_rs::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new parser instance
    let mut parser = Parser::new();

    // Load a configuration file
    // parser.import_config_from_file("path/to/config.json")?;

    // Or load from JSON string
    let sample_config = r#"{
        "key": "sample_bank",
        "account_number_patterns": ["[0-9]{10,12}"],
        "transaction_description_exclude": [],
        "account_terms": ["Sample Bank"],
        ...
    }"#;

    // parser.import_config_from_str(sample_config)?;

    // Parse a PDF file
    // let statement = parser.parse_pdf("statement.pdf", None)?;

    // Or parse with specific configs
    // let statement = parser.parse_pdf("statement.pdf", Some(vec!["sample_bank".to_string()]))?;

    // Access results
    // println!("Account: {}", statement.account_number.unwrap());
    // println!("Transactions: {}", statement.proto_transactions.len());

    Ok(())
}
