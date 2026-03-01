use pdfium_render::prelude::*;
use transtractor_core::structs::TextItem;
use crate::error::ParseError;
use std::path::PathBuf;

/// Extract text items with coordinates from a PDF file
pub fn extract_text_items_from_file(path: &str) -> Result<Vec<TextItem>, ParseError> {
    let bindings = Pdfium::bind_to_system_library()
        .or_else(|_| {
            Pdfium::bind_to_library(
                PathBuf::from("/usr/lib/x86_64-linux-gnu/libpdfium.so")
            )
        })
        .or_else(|_| {
            Pdfium::bind_to_library(
                PathBuf::from("/usr/local/lib/libpdfium.so")
            )
        })
        .map_err(|e| ParseError::PdfError(format!("Failed to bind PDFium: {}", e)))?;

    let pdfium = Pdfium::new(bindings);
    let document = pdfium
        .load_pdf_from_file(path, None)
        .map_err(|e| ParseError::PdfError(format!("Failed to load PDF: {}", e)))?;

    extract_text_items_from_document(&document)
}

/// Extract text items with coordinates from PDF bytes
pub fn extract_text_items_from_bytes(bytes: &[u8]) -> Result<Vec<TextItem>, ParseError> {
    let bindings = Pdfium::bind_to_system_library()
        .or_else(|_| {
            Pdfium::bind_to_library(
                PathBuf::from("/usr/lib/x86_64-linux-gnu/libpdfium.so")
            )
        })
        .or_else(|_| {
            Pdfium::bind_to_library(
                PathBuf::from("/usr/local/lib/libpdfium.so")
            )
        })
        .map_err(|e| ParseError::PdfError(format!("Failed to bind PDFium: {}", e)))?;

    let pdfium = Pdfium::new(bindings);
    let document = pdfium
        .load_pdf_from_byte_slice(bytes, None)
        .map_err(|e| ParseError::PdfError(format!("Failed to load PDF from bytes: {}", e)))?;

    extract_text_items_from_document(&document)
}

fn extract_text_items_from_document(
    document: &PdfDocument,
) -> Result<Vec<TextItem>, ParseError> {
    let mut text_items = Vec::new();

    for page_index in 0..document.pages().len() {
        let page = document
            .pages()
            .get(page_index)
            .map_err(|e| ParseError::PdfError(format!("Failed to get page {}: {}", page_index, e)))?;

        // Extract all text from page
        let page_text = page
            .text()
            .map_err(|e| ParseError::PdfError(format!("Failed to extract text: {}", e)))?;

        // Simple approach: extract raw text and estimate coordinates
        // For more precise coordinates, you'd need additional PDF processing
        let raw_text = page_text.all();

        // Split into lines and create text items
        for line in raw_text.lines() {
            if !line.trim().is_empty() {
                // Estimate coordinates (0,0 as top-left, height 12pt each)
                let y_coord = text_items.len() as i32 * 12;
                let item = TextItem::new(
                    line.to_string(),
                    0,
                    y_coord,
                    (line.len() as i32 * 7), // Rough estimate: ~7px per character
                    y_coord + 12,
                    page_index as i32,
                );
                text_items.push(item);
            }
        }
    }

    if text_items.is_empty() {
        return Err(ParseError::PdfError(
            "No text extracted from PDF".to_string(),
        ));
    }

    Ok(text_items)
}
