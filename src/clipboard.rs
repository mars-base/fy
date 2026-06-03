use arboard::Clipboard;

pub fn get_clipboard_text() -> String {
    let mut clipboard = match Clipboard::new() {
        Ok(clipboard) => clipboard,
        Err(_) => {
            println!("Error: Failed to initialize clipboard");
            return String::new();
        }
    };
    match clipboard.get_text() {
        Ok(text) => text,
        Err(_) => {
            println!("Error: Failed to get clipboard text");
            String::new()
        }
    }
}
