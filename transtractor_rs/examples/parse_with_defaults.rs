use transtractor_rs::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating parser with bundled default configurations...");

    // Create a parser pre-loaded with all bundled configurations
    // This is equivalent to Python's Parser() constructor behavior
    let _parser = Parser::with_defaults()?;

    println!("✓ Parser created successfully with all bundled configs");
    println!();
    println!("The parser is now ready to:");
    println!("  • Auto-detect applicable configs for PDFs");
    println!("  • Parse PDF files: parser.parse_pdf(\"statement.pdf\", None)?");
    println!("  • Parse PDF bytes: parser.parse_pdf_bytes(pdf_bytes, None)?");
    println!("  • Get debug info: parser.get_debug_info(&text_items, None)?");
    println!();
    println!("Bundled configurations available:");
    println!("  ✓ au__cba__credit_card__1   - Commonwealth Bank (Credit Card)");
    println!("  ✓ au__cba__debit__1         - Commonwealth Bank (Debit/Savings)");
    println!("  ✓ au__cba__loan__1          - Commonwealth Bank (Loan)");
    println!("  ✓ au__nab__classic_banking__1 - National Australia Bank (Classic Banking)");

    Ok(())
}
