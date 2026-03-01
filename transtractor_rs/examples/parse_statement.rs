use transtractor_rs::Parser;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Paths to test fixtures
    let pdf_path = "tests/fixtures/test1.pdf";
    let config_path = "tests/fixtures/test1_config.json";

    // Check if test files exist
    if !Path::new(pdf_path).exists() {
        eprintln!("Error: Test PDF not found at {}", pdf_path);
        eprintln!("Make sure you're running from the transtractor-lib directory");
        std::process::exit(1);
    }

    // Create a new parser instance
    let mut parser = Parser::new();

    println!("Loading configuration from: {}", config_path);
    parser.import_config_from_file(config_path)?;

    println!("Parsing PDF from: {}", pdf_path);
    println!();

    // Parse the PDF with auto-detected config
    let statement = parser.parse_pdf(pdf_path, None)?;

    println!("✓ Successfully parsed PDF!");
    println!();
    println!("=== Statement Results ===");
    println!("Config Key:     {}", statement.key.as_ref().unwrap_or(&"N/A".to_string()));
    println!("Account Number: {}", statement.account_number.as_ref().unwrap_or(&"N/A".to_string()));
    println!("Opening Balance: {:?}", statement.opening_balance);
    println!("Closing Balance: {:?}", statement.closing_balance);
    println!("Start Date:      {:?}", statement.start_date);
    println!();
    println!("Transactions: {}", statement.proto_transactions.len());
    println!("Errors:       {}", statement.errors.len());

    if !statement.errors.is_empty() {
        println!();
        println!("=== Errors ===");
        for (i, error) in statement.errors.iter().enumerate() {
            println!("{}. {}", i + 1, error);
        }
    }

    if !statement.proto_transactions.is_empty() {
        println!();
        println!("=== Sample Transactions ===");
        for (i, tx) in statement.proto_transactions.iter().take(3).enumerate() {
            println!("Transaction {}:", i + 1);
            println!("  Date:        {:?}", tx.date);
            println!("  Description: {}", tx.description);
            println!("  Amount:      {:?}", tx.amount);
            println!("  Balance:     {:?}", tx.balance);
        }
        if statement.proto_transactions.len() > 3 {
            println!("... and {} more", statement.proto_transactions.len() - 3);
        }
    }

    Ok(())
}
