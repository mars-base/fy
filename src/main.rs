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
    // convert zh to zh-CN, tw to zh-TW
    let mut to_language = target_language.to_string();
    if target_language == "zh" {
        to_language = "zh-CN".to_string();
    } else if target_language == "tw" {
        to_language = "zh-TW".to_string();
    }

    let encoded_text = urlencoding::encode(text);
    let url = format!("{}?client=gtx&sl=auto&tl={}&dt=t&q={}", vars::GOOGLE_API_URL, to_language, encoded_text);
    // println!("Request URL: {}", url);
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
        // debug response status and body
        println!("Response: {} {}", resp_status, resp_body);
        return "Translation failed".to_string();
    }

    // parse resp_body as json
    let (result, json_value) = utils::json::parseJson(&resp_body.as_str());
    if !result {
        // debug resp_body
        println!("Response body: {}", resp_body);
        return "Translation failed".to_string();
    }
    // debug json value string to console
    if !json_value[0].is_array() {
        println!("Response JSON: {}", utils::json::dumpJsonPretty(&json_value));
        return "Translation failed".to_string();
    }

    let translated_text_list = json_value[0].as_array().unwrap();
    let mut translated_text = String::new();
    for item in translated_text_list {
        // Note: item[0] is the translated text, item[1] is the original text
        // println!("{}", item[0].as_str().unwrap());
        // println!("{}", item[1].as_str().unwrap());
        translated_text.push_str(item[0].as_str().unwrap());
    }
    translated_text
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
