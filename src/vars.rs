// Global constants and thread-safe mutable state
use lazy_static::lazy_static;
use std::sync::Mutex;
lazy_static! {
    pub static ref INITIALIZED: Mutex<bool> = Mutex::new(false);
}

pub const APP_NAME: &str = "fy";
pub const APP_VERSION: &str = "1.0.1";
pub const APP_DESCRIPTION: &str = "Translate text to target language.";
pub const APP_USAGE: &str = "fy [target language: zh/en/..] [optional: text, or from clipboard]";

pub const GOOGLE_API_URL: &str = "https://translate.googleapis.com/translate_a/single";
pub const SUPPORTED_LANGUAGES: &[&str] = &["zh", "en", "ja", "fr", "es", "ru", "la", "ko", "tw"];

// Language list: (code, native_name, english_name)
pub const LANGUAGE_MAP: &[(&str, &str, &str)] = &[
    ("zh", "中文", "Chinese"),
    ("en", "英文", "English"),
    ("ja", "日文", "Japanese"),
    ("fr", "法文", "French"),
    ("es", "西班牙文", "Spanish"),
    ("ru", "俄语", "Russian"),
    ("la", "拉丁文", "Latin"),
    ("ko", "韩文", "Korean"),
    ("tw", "繁体中文", "Traditional Chinese"),
];
