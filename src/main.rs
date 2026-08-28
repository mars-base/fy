#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

mod utils;
mod vars;
mod tests;
mod clipboard;

use std::collections::HashMap;

fn print_help() {
    println!("{} {} {}", vars::APP_NAME, vars::APP_VERSION, vars::APP_DESCRIPTION);
    println!("Usage: {} [-h | --help]", vars::APP_USAGE);
    println!("Supported languages:");
    for (code, name, english_name) in vars::LANGUAGE_MAP {
        println!("  {}: {} ({})", code, name, english_name);
    }
}

async fn translate_async(text: &str, target_language: &str) -> String {
    // If source and target are the same language, return original text
    let detected_sl = detect_source_language(text);
    let detected_sl_mapped = get_mymemory_lang_code(detected_sl);
    let target_mapped = get_mymemory_lang_code(target_language);
    if detected_sl_mapped == target_mapped {
        return text.to_string();
    }

    // Try Google Translate first
    let result = google_translate(text, target_language).await;
    if result != "Translation failed" {
        return result;
    }

    // Fallback to MyMemory
    println!("[Warn] Google Translate failed, falling back to MyMemory...");
    mymemory_translate(text, target_language).await
}

async fn google_translate(text: &str, target_language: &str) -> String {
    // convert zh to zh-CN, tw to zh-TW
    let mut to_language = target_language.to_string();
    if target_language == "zh" {
        to_language = "zh-CN".to_string();
    } else if target_language == "tw" {
        to_language = "zh-TW".to_string();
    }

    let encoded_text = urlencoding::encode(text);
    let url = format!("{}?client=at&sl=auto&tl={}&dt=t&q={}", vars::GOOGLE_API_URL, to_language, encoded_text);
    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (compatible; translate-script/1.0)".to_string());

    let url_client = utils::url::Url::new_with_timeout(
        &url,
        "GET",
        headers,
        "",
        std::time::Duration::from_secs(5));

    let (resp_status, resp_headers, resp_body) = url_client.send().await;

    if resp_status != 200 || resp_body.is_empty() {
        return "Translation failed".to_string();
    }

    // parse resp_body as json
    let (result, json_value) = utils::json::parseJson(&resp_body.as_str());
    if !result {
        return "Translation failed".to_string();
    }
    if !json_value[0].is_array() {
        return "Translation failed".to_string();
    }

    let translated_text_list = json_value[0].as_array().unwrap();
    let mut translated_text = String::new();
    for item in translated_text_list {
        translated_text.push_str(item[0].as_str().unwrap());
    }
    translated_text
}

fn detect_source_language(text: &str) -> &str {
    for c in text.chars() {
        // CJK ranges: Chinese, Japanese Kanji, Korean Hanja
        if ('\u{4E00}'..='\u{9FFF}').contains(&c) ||     // CJK Unified Ideographs
           ('\u{3400}'..='\u{4DBF}').contains(&c) ||     // CJK Extension A
           ('\u{F900}'..='\u{FAFF}').contains(&c) {      // CJK Compatibility Ideographs
            return "zh-CN";
        }
        // Hiragana / Katakana → Japanese
        if ('\u{3040}'..='\u{30FF}').contains(&c) {
            return "ja";
        }
        // Hangul → Korean
        if ('\u{AC00}'..='\u{D7AF}').contains(&c) ||
           ('\u{1100}'..='\u{11FF}').contains(&c) ||
           ('\u{3130}'..='\u{318F}').contains(&c) {
            return "ko";
        }
        // Cyrillic → Russian
        if ('\u{0400}'..='\u{04FF}').contains(&c) {
            return "ru";
        }
    }
    "en"
}

fn get_mymemory_lang_code(code: &str) -> &str {
    for (fy_code, mm_code) in vars::MYMEMORY_LANG_MAP {
        if *fy_code == code {
            return mm_code;
        }
    }
    code
}

async fn mymemory_translate(text: &str, target_language: &str) -> String {
    let encoded_text = urlencoding::encode(text);
    let sl = detect_source_language(text);
    let tl = get_mymemory_lang_code(target_language);
    let url = format!(
        "{}?q={}&langpair={}|{}",
        vars::MYMEMORY_API_URL, encoded_text, sl, tl
    );

    let mut headers = HashMap::new();
    headers.insert("User-Agent".to_string(), "Mozilla/5.0 (compatible; fy-translate/1.0)".to_string());

    let url_client = utils::url::Url::new_with_timeout(
        &url,
        "GET",
        headers,
        "",
        std::time::Duration::from_secs(10),
    );

    let (resp_status, _resp_headers, resp_body) = url_client.send().await;

    if resp_status != 200 || resp_body.is_empty() {
        println!("MyMemory response: {} {}", resp_status, resp_body);
        return "Translation failed".to_string();
    }

    let (result, json_value) = utils::json::parseJson(&resp_body.as_str());
    if !result {
        println!("MyMemory JSON parse failed: {}", resp_body);
        return "Translation failed".to_string();
    }

    // Extract translatedText from responseData
    if let Some(response_data) = json_value.get("responseData") {
        if let Some(translated) = response_data.get("translatedText").and_then(|v| v.as_str()) {
            if !translated.is_empty() {
                // Verify translatedText has a reliable match (quality >= 74 and not from Public Web)
                if let Some(matches) = json_value.get("matches").and_then(|v| v.as_array()) {
                    let text_lower = text.to_lowercase();
                    for m in matches {
                        let seg = m.get("segment").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                        let trans = m.get("translation").and_then(|v| v.as_str()).unwrap_or("");
                        let creator = m.get("created-by").and_then(|v| v.as_str()).unwrap_or("");
                        let quality = m.get("quality").and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))).unwrap_or(0);
                        if trans == translated && seg == text_lower && quality >= 74 && !creator.contains("Public") {
                            return translated.to_string();
                        }
                    }
                }
                // No reliable match found for translatedText, try matches array
            }
        }
    }

    // Fallback: pick best match from matches array, filtering by segment similarity
    // and preferring professional sources over Public Web
    if let Some(matches) = json_value.get("matches").and_then(|v| v.as_array()) {
        let text_lower = text.to_lowercase();
        let mut best_match = "";
        let mut best_score: f64 = -1.0;
        for m in matches {
            let translation = m.get("translation").and_then(|v| v.as_str()).unwrap_or("");
            let segment = m.get("segment").and_then(|v| v.as_str()).unwrap_or("");
            if translation.is_empty() || segment.is_empty() {
                continue;
            }
            // Only accept matches whose segment is similar to the input text
            let seg_lower = segment.to_lowercase();
            let len_diff = (seg_lower.len() as i64 - text_lower.len() as i64).unsigned_abs() as usize;
            if len_diff > text_lower.len().max(2) {
                continue;
            }
            let match_score = m.get("match")
                .and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok())))
                .unwrap_or(0.0);
            let creator = m.get("created-by").and_then(|v| v.as_str()).unwrap_or("");
            // Prefer professional sources: exact match bonus, Public Web penalty
            let bonus = if seg_lower == text_lower { 0.1 } else { 0.0 };
            let penalty = if creator.contains("Public") { -1.0 } else { 0.0 };
            let score = match_score + bonus + penalty;
            if score > best_score {
                best_score = score;
                best_match = translation;
            }
        }
        if !best_match.is_empty() {
            return best_match.to_string();
        }
    }

    "Translation failed".to_string()
}

#[tokio::main]
async fn main() {
    // fetch command line arguments
    let args: Vec<String> = std::env::args().collect();
    // add -h and --help support
    if args.contains(&"-h".to_string()) || args.contains(&"--help".to_string()) {
        print_help();
        utils::tools::exit(0);
    }

    // set target language to "zh" if not exists
    let mut target_language = &String::from("zh");
    let text: String;

    if args.len() >= 2 {
        target_language = &args[1];
    } else {
        println!("No target language specified, default to zh. -h or --help for usage.");
    }

    if args.len() >= 3 {
        text = args[2].clone();
    } else {
        text = clipboard::get_clipboard_text();
        if !text.is_empty() {
            println!("Clipboard text: {}", text);
        }
        if !is_valid_text(&text) {
            println!("Error: clipboard text contains invalid characters");
            print_help();
            utils::tools::exit(1);
        }
    }

    // check target language is valid
    if !vars::SUPPORTED_LANGUAGES.contains(&target_language.as_str()) {
        println!("Error: {} is not a supported language", target_language);
        print_help();
        utils::tools::exit(1);
    }

    // check text is empty
    if text.is_empty() {
        println!("Error: text is empty");
        print_help();
        utils::tools::exit(1);
    }

    // invoke async translate function
    let translated_text = translate_async(text.as_str(), target_language).await;
    println!("{}", translated_text);

    utils::tools::exit(0);
}

fn is_valid_text(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }

    // reject NULL bytes
    if text.contains('\0') {
        return false;
    }

    let mut total = 0;
    let mut printable = 0;

    for c in text.chars() {
        total += 1;

        if c.is_control() {
            // allow common text control characters
            if c == '\n' || c == '\r' || c == '\t' {
                printable += 1;
                continue;
            } else {
                return false;
            }
        }

        // allow all Unicode printable characters (including CJK)
        if !c.is_whitespace() {
            printable += 1;
        }
    }

    // reject binary-looking content (less than 70% printable)
    if printable * 100 / total < 70 {
        return false;
    }

    true
}
