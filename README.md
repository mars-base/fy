# fy

A command-line translation tool.

## Installation

```bash
make build
sudo make install
```

This installs `fy` to `/usr/local/bin/`.

## Usage

```
fy <target_language> [text]
```

If no text argument is given, `fy` reads from the system clipboard.

### Examples

```bash
# Translate to English
fy en "你好世界"        # Hello World

# Translate to Chinese
fy zh "Hello, how are you today?"   # 你好，今天怎么样？

# Translate to Japanese
fy ja "你好"            # こんにちは

# Translate clipboard content to Chinese
fy zh

# Show help
fy -h
```

### Supported Languages

| Code | Language           |
|------|--------------------|
| zh   | Chinese            |
| en   | English            |
| ja   | Japanese           |
| fr   | French             |
| es   | Spanish            |
| ru   | Russian            |
| la   | Latin              |
| ko   | Korean             |
| tw   | Traditional Chinese|

## Development

```bash
make build          # Build release binary
make check          # Run integration tests
make test           # Run unit tests
make clean          # Clean build artifacts
```

## Architecture

1. Parse CLI args: target language code + optional text
2. If no text arg, read from system clipboard
3. Validate input — rejects control chars and binary content
4. Translate via Google Translate API
5. Print translated result

### Module Structure

- **`src/main.rs`** — Entry point, CLI parsing, translation logic, clipboard handling
- **`src/vars.rs`** — Global constants: supported languages, API URL, app metadata
- **`src/clipboard.rs`** — System clipboard read via `arboard`
- **`src/utils/`** — Utility library:
  - `url.rs` — HTTP client wrapper (async via `reqwest`)
  - `json.rs` — JSON parsing helpers
  - `tools.rs` — Process exit, random strings, UUID generation
- **`src/tests/`** — Test modules

## License

MIT
