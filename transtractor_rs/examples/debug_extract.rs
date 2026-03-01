use transtractor_rs::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pdf_path = "tests/fixtures/test1.pdf";

    println!("Extracting text items from: {}", pdf_path);
    println!();

    let parser = Parser::new();
    // We can't directly access extract_text_items_from_file, but we can parse the PDF
    // through the public API

    // For now, let's just test that the parser initializes and can process a PDF
    match parser.parse_pdf(pdf_path, None) {
        Ok(statement) => {
            println!("Successfully parsed statement!");
            println!("  Account: {:?}", statement.account_number);
            println!("  Opening balance: {:?}", statement.opening_balance);
            println!("  Closing balance: {:?}", statement.closing_balance);
            println!("  Transactions: {}", statement.proto_transactions.len());
        }
        Err(e) => {
            println!("Parse error: {}", e);

            // Let's try with defaults
            println!();
            println!("Trying with Parser::with_defaults()...");
            let parser = Parser::with_defaults()?;
            match parser.parse_pdf(pdf_path, None) {
                Ok(statement) => {
                    println!("Successfully parsed with defaults!");
                    println!("  Account: {:?}", statement.account_number);
                    println!("  Opening balance: {:?}", statement.opening_balance);
                    println!("  Closing balance: {:?}", statement.closing_balance);
                    println!("  Transactions: {}", statement.proto_transactions.len());
                }
                Err(e2) => {
                    println!("Also failed with defaults: {}", e2);
                }
            }
        }
    }

    Ok(())
}
