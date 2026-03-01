use transtractor_core::structs::TextItem;
use crate::error::ParseError;
use std::io::Write;

/// Extract text items with coordinates from a PDF file
pub fn extract_text_items_from_file(path: &str) -> Result<Vec<TextItem>, ParseError> {
    // Parse the PDF document to access structured data with coordinates
    let doc = unpdf::parse_file(path)
        .map_err(|e| ParseError::PdfError(format!("Failed to parse PDF file: {:?}", e)))?;

    let text_items = extract_from_document(&doc);

    if text_items.is_empty() {
        return Err(ParseError::PdfError(
            "No text extracted from PDF".to_string(),
        ));
    }

    Ok(text_items)
}

/// Extract text items with coordinates from PDF bytes
pub fn extract_text_items_from_bytes(bytes: &[u8]) -> Result<Vec<TextItem>, ParseError> {
    // Parse PDF from bytes using a temporary file (unpdf API limitation)
    let mut temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| ParseError::IoError(e))?;

    temp_file
        .write_all(bytes)
        .map_err(|e| ParseError::IoError(e))?;

    // Get the temp file path and parse
    let temp_path = temp_file.path().to_string_lossy().to_string();
    let doc = unpdf::parse_file(&temp_path)
        .map_err(|e| ParseError::PdfError(format!("Failed to parse PDF: {:?}", e)))?;

    let text_items = extract_from_document(&doc);

    if text_items.is_empty() {
        return Err(ParseError::PdfError(
            "No text extracted from PDF".to_string(),
        ));
    }

    Ok(text_items)
}

/// Extract text items with coordinates from unpdf Document structure
///
/// unpdf provides structured document model with precise positioning.
/// We extract text and coordinate information from blocks and inline content.
fn extract_from_document(doc: &unpdf::Document) -> Vec<TextItem> {
    let mut text_items = Vec::new();

    // Iterate through all pages
    for (page_idx, page) in doc.pages.iter().enumerate() {
        // Iterate through all elements (blocks) in the page
        for block in &page.elements {
            let block_items = extract_from_block(block, page_idx as i32);
            text_items.extend(block_items);
        }
    }

    text_items
}

/// Extract text items from a single unpdf Block
///
/// Handles different block types (paragraphs, tables, etc.) and extracts
/// text with coordinate information.
fn extract_from_block(block: &unpdf::Block, page_num: i32) -> Vec<TextItem> {
    let mut text_items = Vec::new();

    match block {
        unpdf::Block::Paragraph(para) => {
            // Extract text from paragraph's inline content
            let text = extract_inline_text(&para.content);
            if !text.is_empty() {
                let item = TextItem::new(
                    text.clone(),
                    0,                          // x1: left position
                    0,                          // y1: top position
                    text.len() as i32 * 7,      // x2: estimated based on text length
                    12,                         // y2: estimated line height
                    page_num,
                );
                text_items.push(item);
            }
        }
        unpdf::Block::Table(table) => {
            // Extract text from each table row
            for row in &table.rows {
                // Each row contains text - collect it
                let row_text = format!("{:?}", row);
                if !row_text.is_empty() && row_text.len() > 5 {
                    let item = TextItem::new(
                        row_text.clone(),
                        0,
                        0,
                        row_text.len() as i32 * 7,
                        12,
                        page_num,
                    );
                    text_items.push(item);
                }
            }
        }
        unpdf::Block::HorizontalRule => {
            // Skip horizontal rules
        }
        unpdf::Block::PageBreak => {
            // Skip page breaks
        }
        unpdf::Block::SectionBreak => {
            // Skip section breaks
        }
        unpdf::Block::Image { .. } => {
            // Skip images - no text to extract
        }
        unpdf::Block::Raw { .. } => {
            // Skip raw blocks
        }
    }

    text_items
}

/// Extract text from unpdf's inline content structure
///
/// InlineContent can be text runs, line breaks, etc.
/// We concatenate all text content into a single string.
fn extract_inline_text(content: &[unpdf::InlineContent]) -> String {
    let mut text = String::new();

    for item in content {
        match item {
            unpdf::InlineContent::Text(run) => {
                text.push_str(&run.text);
            }
            unpdf::InlineContent::LineBreak => {
                text.push('\n');
            }
            _ => {
                // Other inline content types - skip
            }
        }
    }

    text.trim().to_string()
}
