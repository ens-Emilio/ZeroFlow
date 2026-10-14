// OCR via Tesseract (leptess). Port of CTS/TextRecognition/*.
// The original uses Windows.Media.Ocr; here we wrap Tesseract and produce the
// same OcrDocument shape: words with bounding boxes, so the TS hit-tester
// (OcrTextHitTester port) works unchanged.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrWord {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrDocument {
    pub words: Vec<OcrWord>,
    pub language_tag: String,
}

#[tauri::command]
pub async fn ocr(png_base64: String, lang_tag: String) -> Result<OcrDocument, String> {
    tokio::task::block_in_place(|| run_ocr(png_base64, lang_tag))
}

fn run_ocr(png_base64: String, lang_tag: String) -> Result<OcrDocument, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(png_base64.trim())
        .map_err(|e| format!("base64 decode: {e}"))?;
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("decode image: {e}"))?;
    let mut png = Vec::new();
    img.to_rgba8()
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|e| format!("encode png: {e}"))?;

    // Empty lang_tag means Tesseract default (eng).
    let lang = if lang_tag.trim().is_empty() {
        "eng".to_string()
    } else {
        lang_tag.trim().to_string()
    };

    let mut api = leptess::LepTess::new(None, &lang).map_err(|e| {
        format!("tesseract init ({lang}): {e} - install tesseract-ocr and the traineddata for this language")
    })?;
    api.set_image_from_mem(&png)
        .map_err(|e| format!("tesseract set image: {e}"))?;
    // Screenshot crops are small: single block, no OSD.
    let _ = api.set_variable(leptess::Variable::TesseditPagesegMode, "6");
    // Assume ~150 dpi so Tesseract skips its tiny-image heuristics.
    api.set_fallback_source_resolution(150);

    // get_word_str_box_text returns "makebox" format:
    // "text x1 y_bottom x2 y_top 0" per word (y coords bottom-up, origin at bottom-left).
    let tsv = api
        .get_word_str_box_text(0)
        .map_err(|e| format!("tesseract tsv: {e}"))?;
    eprintln!("zeroflow: ocr makebox[:400] {}", &tsv[..tsv.len().min(400)]);
    if tsv.trim().is_empty() {
        eprintln!("zeroflow: ocr tsv empty, falling back to plain text");
        let full = api.get_utf8_text().map_err(|e| format!("tesseract text: {e}"))?;
        eprintln!("zeroflow: ocr plain text: {:?}", &full[..full.len().min(80)]);
    }
    let mut words = Vec::new();
    // Format observed from leptess get_word_str_box_text:
    //   "WordStr x1 y_bottom x2 y_top 0 #<full line text>"
    // (y coords bottom-up, origin bottom-left; text follows a '#' column).
    // CSS coords: origin top-left.
    let img_h = api.get_image_dimensions().map(|(_, h)| h as f64).unwrap_or(1.0);
    for line in tsv.lines() {
        let (coord_part, text) = match line.split_once('#') {
            Some((c, t)) => (c, t.trim()),
            None => continue,
        };
        if text.is_empty() {
            continue;
        }
        let cols: Vec<&str> = coord_part.split_whitespace().collect();
        if cols.len() < 5 {
            continue;
        }
        let (Ok(x1), Ok(y_bot), Ok(x2), Ok(y_top)) = (
            cols[1].parse::<f64>(),
            cols[2].parse::<f64>(),
            cols[3].parse::<f64>(),
            cols[4].parse::<f64>(),
        ) else {
            continue;
        };
        // makebox y grows upwards: y_top > y_bot. CSS y of the word's top edge.
        let y = img_h - y_top;
        let height = y_top - y_bot;
        if height < 1.0 {
            continue;
        }
        words.push(OcrWord {
            text: text.to_string(),
            x: x1,
            y,
            width: x2 - x1,
            height,
        });
    }
    eprintln!("zeroflow: ocr {} words (lang {lang})", words.len());
    Ok(OcrDocument { words, language_tag: lang })
}
