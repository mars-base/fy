use arboard::Clipboard;
use std::sync::Mutex;

// Clipboard is a system-wide shared resource; concurrent reads/writes interfere.
// Use a mutex to ensure serial execution across tests.
static CLIPBOARD_LOCK: Mutex<()> = Mutex::new(());

// Helper: write text to clipboard and run an assertion closure.
// On Linux/X11 the clipboard content is owned by the owning process and
// disappears when the Clipboard instance is dropped, so writes and reads
// must happen within the same Clipboard instance's lifetime.
fn with_clipboard_text(text: &str, f: impl FnOnce()) {
    let _guard = CLIPBOARD_LOCK.lock().unwrap();
    let mut cb = match Clipboard::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Clipboard unavailable ({}), skipping test", e);
            return;
        }
    };
    if cb.set_text(text).is_err() {
        eprintln!("Failed to write to clipboard, skipping test");
        return;
    }
    // cb still alive (holds X11 ownership), get_clipboard_text can read back correctly
    f();
}

#[cfg(test)]
mod tests {
    use super::{with_clipboard_text, Clipboard, CLIPBOARD_LOCK};
    use crate::clipboard::get_clipboard_text;

    /// Basic round-trip: ASCII text should be read back unchanged
    #[test]
    fn test_get_clipboard_text_ascii() {
        let text = "Hello, clipboard!";
        with_clipboard_text(text, || {
            assert_eq!(get_clipboard_text(), text);
        });
    }

    /// Chinese text: UTF-8 multi-byte characters should not be lost or garbled
    #[test]
    fn test_get_clipboard_text_chinese() {
        let text = "你好，这是剪切板测试！";
        with_clipboard_text(text, || {
            assert_eq!(get_clipboard_text(), text);
        });
    }

    /// Mixed languages: Chinese, English, Japanese, Korean
    #[test]
    fn test_get_clipboard_text_mixed_languages() {
        let text = "Hello 你好 こんにちは 안녕하세요";
        with_clipboard_text(text, || {
            assert_eq!(get_clipboard_text(), text);
        });
    }

    /// Multi-line text: newlines should be preserved
    #[test]
    fn test_get_clipboard_text_multiline() {
        let text = "第一行\n第二行\n第三行";
        with_clipboard_text(text, || {
            assert_eq!(get_clipboard_text(), text);
        });
    }

    /// Special characters: ASCII punctuation should not be escaped
    #[test]
    fn test_get_clipboard_text_special_chars() {
        let text = r#"!@#$%^&*()_+-=[]{}|;':",./<>?"#;
        with_clipboard_text(text, || {
            assert_eq!(get_clipboard_text(), text);
        });
    }

    /// Overwrite: second write should replace the first
    #[test]
    fn test_get_clipboard_text_overwrite() {
        let _guard = CLIPBOARD_LOCK.lock().unwrap();
        let mut cb = match Clipboard::new() {
            Ok(c) => c,
            Err(e) => { eprintln!("Clipboard unavailable ({}), skipping test", e); return; }
        };
        if cb.set_text("first").is_err() { return; }
        if cb.set_text("second").is_err() { return; }
        assert_eq!(get_clipboard_text(), "second");
    }

    /// Safety net: get_clipboard_text should never panic under any condition.
    /// macOS headless CI has no NSPasteboard server — arboard can throw ObjC
    /// exceptions that Rust cannot catch, so this test is skipped on macOS.
    #[test]
    #[cfg_attr(target_os = "macos", ignore)]
    fn test_get_clipboard_text_never_panics() {
        let _result = get_clipboard_text();
    }
}
