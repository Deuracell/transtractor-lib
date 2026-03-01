use transtractor_core::structs::TextItem;
use crate::error::ParseError;
use lopdf::Document;
use std::io::Write;

/// Extract text items with accurate coordinates from a PDF file
pub fn extract_text_items_from_file(path: &str) -> Result<Vec<TextItem>, ParseError> {
    let doc = Document::load(path)
        .map_err(|e| ParseError::PdfError(format!("Failed to parse PDF: {:?}", e)))?;

    let text_items = extract_from_document(&doc)?;

    if text_items.is_empty() {
        return Err(ParseError::PdfError(
            "No text extracted from PDF".to_string(),
        ));
    }

    Ok(text_items)
}

/// Extract text items with accurate coordinates from PDF bytes
pub fn extract_text_items_from_bytes(bytes: &[u8]) -> Result<Vec<TextItem>, ParseError> {
    // Write bytes to temp file (lopdf requires file path)
    let mut temp_file = tempfile::NamedTempFile::new()
        .map_err(|e| ParseError::IoError(e))?;

    temp_file
        .write_all(bytes)
        .map_err(|e| ParseError::IoError(e))?;

    let temp_path = temp_file.path().to_string_lossy().to_string();
    let doc = Document::load(&temp_path)
        .map_err(|e| ParseError::PdfError(format!("Failed to parse PDF: {:?}", e)))?;

    let text_items = extract_from_document(&doc)?;

    if text_items.is_empty() {
        return Err(ParseError::PdfError(
            "No text extracted from PDF".to_string(),
        ));
    }

    Ok(text_items)
}

/// Internal struct to track text state while parsing content streams
#[derive(Debug, Clone)]
struct TextState {
    x: f32,
    y: f32,
    x_line_start: f32,
    y_line_start: f32,
    font_size: f32,
    font_name: Vec<u8>,
    leading: f32,
}

impl Default for TextState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            x_line_start: 0.0,
            y_line_start: 0.0,
            font_size: 12.0,
            font_name: Vec::new(),
            leading: 0.0,
        }
    }
}

/// Intermediate text span with position information
#[derive(Debug, Clone)]
struct TextSpan {
    text: String,
    x: f32,
    y: f32,
    width: f32,
    font_size: f32,
    page: i32,
}

/// Extract text items from lopdf Document
fn extract_from_document(doc: &Document) -> Result<Vec<TextItem>, ParseError> {
    let mut text_items = Vec::new();

    // Get all page IDs
    let pages = doc.get_pages();

    // Sort pages by page number
    let mut page_ids: Vec<_> = pages.iter().map(|(num, id)| (*num, *id)).collect();
    page_ids.sort_by_key(|&(num, _)| num);

    for (page_num, page_id) in page_ids {
        let page_text_items = extract_page_text(doc, page_id, page_num)?;
        text_items.extend(page_text_items);
    }

    Ok(text_items)
}

/// Extract text items from a single page
fn extract_page_text(
    doc: &Document,
    page_id: (u32, u16),
    page_num: u32,
) -> Result<Vec<TextItem>, ParseError> {
    // Get page dimensions from MediaBox
    let page_dict = doc.get_object(page_id)
        .map_err(|e| ParseError::PdfError(format!("Failed to get page: {:?}", e)))?;

    let page_dict_ref = match page_dict {
        lopdf::Object::Dictionary(d) => d,
        _ => {
            return Err(ParseError::PdfError("Page is not a dict".to_string()));
        }
    };

    let media_box_result = page_dict_ref.get(b"MediaBox");
    let media_box_obj = match media_box_result {
        Ok(obj) => obj,
        Err(_) => {
            return Err(ParseError::PdfError("No MediaBox found".to_string()));
        }
    };

    let media_box = match media_box_obj.as_array() {
        Ok(arr) => arr,
        Err(_) => {
            return Err(ParseError::PdfError("MediaBox is not an array".to_string()));
        }
    };

    let page_height = if media_box.len() >= 4 {
        match media_box[3].as_float() {
            Ok(v) => v as f32,
            Err(_) => 842.0,
        }
    } else {
        842.0
    };

    // Get page content stream operations
    let content = doc.get_page_content(page_id)
        .and_then(|bytes| lopdf::content::Content::decode(&bytes))
        .map_err(|e| ParseError::PdfError(format!("Failed to decode page content: {:?}", e)))?;

    // Extract text spans from content operations
    let spans = extract_text_spans_from_content(&content, page_num as i32)?;

    // Group spans into words and convert to TextItems
    let text_items = group_spans_to_items(spans, page_height);

    Ok(text_items)
}

/// Extract text spans from page content operations
fn extract_text_spans_from_content(
    content: &lopdf::content::Content,
    page_num: i32,
) -> Result<Vec<TextSpan>, ParseError> {
    let mut spans = Vec::new();
    let mut state = TextState::default();

    for op in &content.operations {
        match op.operator.as_ref() {
            "BT" => {
                state = TextState::default();
            }
            "ET" => {
                // End text block
            }
            "Tm" => {
                // Set text matrix [a b c d x y]
                if op.operands.len() >= 6 {
                    if let (Ok(x), Ok(y)) =
                        (op.operands[4].as_float(), op.operands[5].as_float())
                    {
                        state.x = x as f32;
                        state.y = y as f32;
                        state.x_line_start = state.x;
                        state.y_line_start = state.y;
                    }
                }
            }
            "Td" => {
                // Move text position [tx ty]
                if op.operands.len() >= 2 {
                    if let (Ok(tx), Ok(ty)) =
                        (op.operands[0].as_float(), op.operands[1].as_float())
                    {
                        state.x += tx as f32;
                        state.y += ty as f32;
                    }
                }
            }
            "TD" => {
                // Move text position and set leading [tx ty]
                if op.operands.len() >= 2 {
                    if let (Ok(tx), Ok(ty)) =
                        (op.operands[0].as_float(), op.operands[1].as_float())
                    {
                        state.x += tx as f32;
                        state.y += ty as f32;
                        state.leading = -(ty as f32);
                    }
                }
            }
            "T*" => {
                // Move to next line
                state.x = state.x_line_start;
                state.y -= state.leading;
            }
            "TL" => {
                // Set text leading [tl]
                if op.operands.len() >= 1 {
                    if let Ok(tl) = op.operands[0].as_float() {
                        state.leading = tl as f32;
                    }
                }
            }
            "Tf" => {
                // Set font [name size]
                if op.operands.len() >= 2 {
                    if let Ok(size) = op.operands[1].as_float() {
                        state.font_size = size as f32;
                    }
                    if let Ok(name) = op.operands[0].as_name() {
                        state.font_name = name.to_vec();
                    }
                }
            }
            "Tj" => {
                // Show text [string]
                if op.operands.len() >= 1 {
                    if let Ok(text) = decode_text_operand(&op.operands[0]) {
                        let width = estimate_text_width(&text, state.font_size);
                        spans.push(TextSpan {
                            text,
                            x: state.x,
                            y: state.y,
                            width,
                            font_size: state.font_size,
                            page: page_num,
                        });
                        state.x += width;
                    }
                }
            }
            "TJ" => {
                // Show text with individual glyph spacing [(string|offset) ...]
                if op.operands.len() >= 1 {
                    if let Ok(array) = op.operands[0].as_array() {
                        for item in array {
                            if let Ok(text) = decode_text_operand(item) {
                                let width = estimate_text_width(&text, state.font_size);
                                spans.push(TextSpan {
                                    text,
                                    x: state.x,
                                    y: state.y,
                                    width,
                                    font_size: state.font_size,
                                    page: page_num,
                                });
                                state.x += width;
                            } else if let Ok(offset) = item.as_float() {
                                // Kerning adjustment in text space
                                state.x -= (offset as f32) * state.font_size / 1000.0;
                            } else if let Ok(offset) = item.as_i64() {
                                // Kerning as integer
                                state.x -= (offset as f32) * state.font_size / 1000.0;
                            }
                        }
                    }
                }
            }
            "'" => {
                // Move to next line and show text
                state.x = state.x_line_start;
                state.y -= state.leading;
                if op.operands.len() >= 1 {
                    if let Ok(text) = decode_text_operand(&op.operands[0]) {
                        let width = estimate_text_width(&text, state.font_size);
                        spans.push(TextSpan {
                            text,
                            x: state.x,
                            y: state.y,
                            width,
                            font_size: state.font_size,
                            page: page_num,
                        });
                        state.x += width;
                    }
                }
            }
            "\"" => {
                // Set word/char spacing, move to next line, show text
                if op.operands.len() >= 3 {
                    state.x = state.x_line_start;
                    state.y -= state.leading;
                    if let Ok(text) = decode_text_operand(&op.operands[2]) {
                        let width = estimate_text_width(&text, state.font_size);
                        spans.push(TextSpan {
                            text,
                            x: state.x,
                            y: state.y,
                            width,
                            font_size: state.font_size,
                            page: page_num,
                        });
                        state.x += width;
                    }
                }
            }
            _ => {}
        }
    }

    Ok(spans)
}

/// Decode a text operand (string or name) to String
fn decode_text_operand(obj: &lopdf::Object) -> Result<String, String> {
    match obj {
        lopdf::Object::String(bytes, _) => {
            // Try UTF-8 first, fall back to Latin-1
            String::from_utf8(bytes.clone())
                .or_else(|_| {
                    // Fallback: interpret as Latin-1
                    Ok(bytes.iter().map(|&b| b as char).collect())
                })
        }
        _ => Err("Not a string".to_string()),
    }
}

/// Estimate text width (characters × average char width)
fn estimate_text_width(text: &str, font_size: f32) -> f32 {
    // Assume average character width is 0.6 × font_size
    text.len() as f32 * font_size * 0.6
}

/// Group text spans into words and convert to TextItems
fn group_spans_to_items(spans: Vec<TextSpan>, page_height: f32) -> Vec<TextItem> {
    if spans.is_empty() {
        return Vec::new();
    }

    // Sort spans by y (rounded to 2pt bins), then by x
    let mut sorted = spans;
    sorted.sort_by(|a, b| {
        let y_a = (a.y / 2.0).round();
        let y_b = (b.y / 2.0).round();
        if y_a == y_b {
            a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            y_a.partial_cmp(&y_b).unwrap_or(std::cmp::Ordering::Equal)
        }
    });

    let mut items = Vec::new();
    let mut current_group = vec![sorted[0].clone()];
    let x_tolerance = 2.0;
    let y_tolerance = 2.0;

    for span in sorted.iter().skip(1) {
        let last = &current_group[current_group.len() - 1];

        // Check if span should be merged with current group
        let y_diff = (span.y - last.y).abs();
        let x_gap = span.x - (last.x + last.width);

        if y_diff <= y_tolerance && x_gap <= x_tolerance {
            current_group.push(span.clone());
        } else {
            // Flush current group as a TextItem
            let item = group_to_item(current_group, page_height);
            items.push(item);
            current_group = vec![span.clone()];
        }
    }

    // Flush final group
    if !current_group.is_empty() {
        let item = group_to_item(current_group, page_height);
        items.push(item);
    }

    items
}

/// Convert a group of spans into a TextItem
fn group_to_item(group: Vec<TextSpan>, page_height: f32) -> TextItem {
    // Concatenate text
    let text = group
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<_>>()
        .join("");

    // Calculate bounding box
    let x1 = group[0].x;
    let x2 = group[group.len() - 1].x + group[group.len() - 1].width;
    let font_size = group[0].font_size; // Use first span's font size
    let page = group[0].page;

    // Convert PDF coords (bottom-left origin) to pdfplumber coords (top-left origin)
    // baseline y in PDF → distance from top of page in pdfplumber coords
    let y_baseline_pdf = group[0].y;
    let y1_pdfplumber = (page_height - y_baseline_pdf) as i32;
    let y2_pdfplumber = (page_height - y_baseline_pdf - font_size * 0.8) as i32;

    TextItem::new(
        text,
        x1 as i32,
        y1_pdfplumber,
        x2 as i32,
        y2_pdfplumber,
        page,
    )
}
